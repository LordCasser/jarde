# ecab deferred binding 静态收尾审查

本次只审读当前 ecab 工作树中的 `crates/jarde-java/src/build.rs`、
`crates/jarde-java/src/init.rs`、`crates/jarde-java/src/names.rs` 以及现有
`jarde_reader::budget::Budget`/`crate::stop` 契约。没有运行 Cargo，也没有改生产代码、
Rust 测试、任务勾选、census 或 fingerprint。这里的 `collect_value_dependency_bcis`
在当前源码中的实际名字是 `Builder::collect_dependency_bcis`。

## 结论

ecab 已经把 deferred binding 的证明失败与安全内联区分开，并把成功声明和后续引用闭合：

* `terminal_consumer` 的直接 reader、嵌套表达式 reader、未知/合并值和深度截断都不能
  生成成功计划；`collect_dependency_bcis` 的不完整结果会向
  `has_independent_boundary` 传播为 rejection。
* `Site` 的 `owned` 仍是固定的 allocation/`dup`/constructor 三个 BCI。constructor 才是
  可绑定的完成值，allocation 或 `dup` 不会被误当成已初始化对象的别名。
* `instruction` 先消费 `binding_rejections` 和 `binding_plans`，所以已进入这两张表的
  producer 不会再走普通 `Invoke` 的旧 deferred 分支。声明失败后
  `binding_refused` 先于 SSA 重算检查，后续 reader 不会悄悄回到旧的 inline `render_value`。
* 证明遍历、候选和 rejection 条目都有现有 `poll`/`IrItems` 计费；两个依赖递归使用
  `MAX_VALUE_DEPTH`。`seen` 命中是已完成 shared-DAG 去重，不能视为不完整证明。

发现两处新增工作与预算/取消检查之间的窄缺口。它们不会把失败声明变成未声明变量，也
没有把已知安全 inline 误报成错误；影响是冲突名称或拒绝来源很多时，工作可能在下一次
checkpoint 才被观察到。

## 1. preparation、终端 consumer 和依赖 proof

`prepare_deferred_bindings`（`build.rs:2172-2320`）对每个 SSA value 先
`poll` 并收取一个 `IrItems`（2175-2188）。支持的 producer 还必须真正写出该 value
（2189-2201），且只能有一个 use；多 use 会写入 `BindingRejection`（2202-2212）。

`terminal_consumer`（2423-2458）每次递归先检查 `MAX_VALUE_DEPTH`，再检查唯一 use、
具体 BCI，并对读取的 instruction 轮询和计费。透明的表达式 producer 只沿自己的唯一
stack output 继续；local store、merge、缺失 BCI 或不支持的 operation 会返回 `None`，
回到 preparation 的 rejection（2214-2223）。这保证它寻找的是最终呈现语句，而不是把
直接 arithmetic reader 当作求值位置。

`has_independent_boundary`（2363-2417）把 producer 依赖和最终 consumer 依赖分别收集，
然后仅扫描同一 canonical block 的中间区间；每个区间 instruction 都有 `poll`/`IrItems`
（2402-2415）。

实际的 `collect_dependency_bcis`（2493-2534）有三个安全出口：超过深度返回 `false`
（2500-2502），Phi/Caught/非 Entry 的非 instruction 定义返回不完整（2506-2511），
未知 operation 或缺失 instruction 也返回 `false`（2519-2523）。递归结果用
`complete &= ...` 汇总，因此某个 operand 不完整时不会被当成已证明；调用方把 `false`
转换为 `None`，最终写 rejection（2370-2371、2392-2400、2244-2255）。同一 SSA value
再次遇到时返回 `true`（2503-2505），这是共享 DAG 的有限去重，不是失败证据。

因此，当前新增 proof 的深度和 cancellation/IR 计费闭环成立。`Some(false)` 只表示区间
没有独立边界，保留原有 inline 是有序的；它不表示未知 proof 被放行。对不支持的
producer，`binding_producer` 本身返回 `None`，继续使用该 operation 原有路径，这属于本项
既定范围外，不能和“保存失败后回落”混为一谈。

## 2. Site owned 与 new alias

`binding_producer`（2331-2357）对 construction site 只接受
`producer == site.constructor`（2331-2339）。`init.rs:422-430` 的 `Site::owned` 仍由
`[head, dup, constructor]` 这三个已验证身份组成；constructor 参数的 producer BCI 仅保留
在 `arguments` 中用于参数顺序和来源，不被加入 uninitialized site alias 集合。

依赖 proof 在遇到 site 时把这个固定三元素集合加入 `bcis`（2516-2517），这是常数大小的
来源扩展，且该 value 的 instruction 已先计费；不会随着整张 SSA 图无界扩展。
`new_expr`（5073-5111）在 constructor 位置呈现一次参数列表，并把 site owned anchors
放进 expression origin。故 declaration 的 initializer 不会在 consumer 处重新构造对象，
对 Site 可达的 render-error 拒绝，`bind_value` 与 `reject_binding` 都会把 site owned 加入
quote（2567-2578、2581-2595）。仅缺少 `presented` 类型的通用分支使用
`quoted_bcis` 自己的 producer/operand 来源（2598-2608）；已验证 Site 的 `new_expr` 要么
产生 reference type，要么在不可拼写 class 时先走 render-error 分支，因此不会漏掉 Site
owned。

`init::sites` 的 outside-reader 扫描是已有 construction-site 计划，不是 ecab 新增的全图
cache/pass；本审查没有把它没有独立 `Budget` 参数误报成 deferred proof 的漏洞。ecab 新增
的 `site_of`/owned 使用只读取这个既定的三 BCI 事实。

## 3. Declare commit、拒绝来源和旧 render 路径

提交顺序在 `bind_value`（2581-2633）是闭合的：

1. 先在 anchor 位置 `render_value` 初始化表达式；失败就 `refuse_binding`，再以
   `quoted_bcis(anchor)` 加上 site owned 做 fallback。
2. 没有 `presented` 类型时同样先拒绝再 fallback。当前已验证 site 的 `new_expr` 会从
   可拼写 class 产生 reference type；不可拼写 class 在 `new_expr` 内先报 render error，
   因此不会绕过上面的 source quote。
3. `push(Declare)` 成功后还检查 `stmts.last()` 确实是同名 `Declare`（2610-2623）。若
   `undeclared_statement` 把它变成 fallback，则标记 refused；只有这个检查通过后才写入
   `bindings`（2625-2633）。`push` 若因 StopReason 失败，整个 build 直接停止，不会发布
   部分 Program，也不存在后续旧 render。
4. `reject_binding`（2567-2579）和所有 `bind_value` 失败路径都调用
   `mark_binding_refused`。`render_value` 在查 SSA definition 前先检查
   `binding_refused`（4088-4103），所以消费者看到的是明确错误并进入自己的 fallback，
   而不是再次 inline producer。

`instruction` 在任何普通形状前先检查 rejection，再检查 plan（2637-2644）。因此，对于
已经被 preparation 识别为 supported producer 的值，rejection/Declare failure 不会继续
进入普通 `Invoke` 分支中 `produced_value_reaches_a_reader` 的旧延期逻辑
（2800-2817）。失败时加入的 `deferred` 只供 `quoted_bcis` 的来源传播使用，不会解除
`binding_refused`；`quoted_bcis` 会递归保留 deferred producer、field/array/cast 等实际
来源（5466-5481、5526-5613）。这条路径没有发现“声明失败后旧 render 重新计算”的泄漏。

## 4. 找到的预算/取消缺口：free_name 冲突循环

候选计划每项只在名称工作前收取一次 `IrItems`（`build.rs:2279-2286`）。随后
`build.rs:2288-2296` 循环调用 `NameTable::free_name`，再和 `synthetic_names`、
`lambda_params` 比较并追加下划线。`NameTable::free_name`（`names.rs:491-503`）每次都
重新建立完整 reserved/local 名字集合（492-497），并在 499-501 扫描冲突候选；这些扫描
和每轮冲突都没有 `poll` 或 `charge`。

循环本身是有限的：候选字符串每轮增长，冲突来源来自有限的 local/reserved/lambda/已发布
synthetic 名集合。因此这是计费粒度和取消响应缺口，不是无限循环或递归溢出。但在极端名称
冲突时，`Budget::charge`/`stop::poll`（`jarde-reader/src/budget.rs:499-549`、
`jarde-java/src/stop.rs:115-150`）不会在该段工作内部执行；只有下一候选、下一条 statement
或之后的阶段才会看到取消/截止时间。该缺口正对应 tasks 2.5 的“名称冲突循环共享既有预算”
要求。

## 5. 来源 quote 的次级计费缺口

ecab 的 rejection 和 failed `bind_value` 新增调用 `quoted_bcis`。其
`deferred_producers` 已有 `MAX_VALUE_DEPTH`（`build.rs:5526-5529`），因此深度上有界；
但它逐层走 stack operands 时没有 `poll`/`IrItems`，也没有像
`collect_dependency_bcis` 那样的 SSA `seen` 集合（5558-5561、5607-5613）。`into.contains`
只去重最终 BCI，不阻止同一个未标记为 deferred 的共享子树再次递归。

这条 quote walk 旧有 fallback 也会用到，但 ecab 把它接到了新的
`reject_binding`/Declare-failure 路径，所以新增拒绝越多，未计费且不可取消的来源工作越
多。`MAX_VALUE_DEPTH` 仍防止无界递归，故这是低优先级的 accounting/cancellation debt，
不是当前发现的正文错序或来源丢失。后续若补预算，应复用现有 proof 的 poll/charge，并以
ValueId 级去重；不应改变 quote 的来源集合语义。

## 静态审查边界

本文没有运行新 CLI 或 Cargo，不能替代 root 的完整类行为验收。没有把缺少新字段当作
作用域错误，也没有把 `collect_dependency_bcis` 的 shared-DAG 去重当成失败；跨 region、
guard、switch 和实际异常顺序仍以 root 的独立运行证据为准。

## root 对结论的时序限定

此静态审查的“消费者不会旧inline”只对已进入instruction并标记binding_refused后的消费成立。root实际switch-core/ecab证明switch_join会在arm前预先render返回，此时已计划拒绝尚未提交，旧调用仍被生成并错序。该运行证据优先，不得将本文局部提交顺序结论扩大到所有呈现入口。Site固定3项的事实经根审读采纳，不要求扩充新来源机制。命名内部计费继续本项补齐，旧deferred_producers工作预算债务记录并保持最小范围。
