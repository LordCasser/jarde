## Context

见 proposal.md。`reuse::plan` 已按 typed、array、reference 路径尝试局部身份拆分；引用路径要求 `records.is_empty()`。有两个同名 LVT row 时，该路径拒绝，后续 unique-name shortcut 得到 `Whole("value")`。当前 debug 完整类因此有 String→Integer 编译错误；none 模式的两个身份没有这项错误，但两模式都因独立 `handled()` 拒绝而无法完整类运行。

新基线各25真实命令、8份 Jarde 11方法报告、四条原类运行各17场景。路径为 `/private/tmp/jarde-em20-raw-alias-escape-baseline-{debug,none}-current-root-v1`。这是观察而非实现验收。JADX 1.5.6 的两种 rename profile 都把 raw alias 写入折成泛型接收者调用，完整编译失败。其 SSA/debug attach/CodeVar/ProcessVariables 的分层可参考，错误泛型折叠不能照搬。

## Goals / Non-Goals

**Goals:** 在已经证明的引用生命周期上附加调试范围和名称。定义、值流、CFG、来源仍是同一次真实 IR；仅扩大现有私有引用拆分的可接受证据，最小闭环落在 reuse。

**Non-Goals:** 不新增 solver、pass、公共 carrier 或变量命名空间；不放宽泛型投影，不删除 raw alias，不承诺 sibling scopes 使用相同文本名称。外层 try/foreach 属于独立 region composition 问题，需同次 Walker 跟踪后单独设计。

## Decisions

1. **先证明身份，再匹配调试记录。** 保留 exact reference type、`reference_values_stay_before_boundaries`、`owned_definition_split` 与 canonical normal-flow 条件。私有 helper 返回与 typed split 相同的 names-plus-Split 形状；无记录时仍是 unnamed identities。有记录时将每个读取的 LVT interval 匹配到已证明的 `Split.by_bci` 身份。所有读取各有唯一 row，每个 row 与身份一一对应、非空互不重叠，且范围顺序与定义顺序一致。每个身份必须有读取证据。初始化 store 的 slot5 BCI30/72 在 row [32,41)/[74,82) 之前是正常编译产物，归属继续由 SSA 决定。

2. **名称在完整证明后发布。** 匹配失败返回现有 refusal/fallback，不发布部分 split。参数、resource、未知/等同类型、缺失访问、克隆/不可达块、未支持边、逆向跨生命周期流、歧义 phi 和 stale reference 的门不变。新增遍历仍用既有 budget poll/charge；不依赖 optional evidence 选择。可复用现有有界匹配，不为查表引入通用框架。

3. **复用现有声明和命名。** 两个身份进入现有 declaration placement，各自落在对应循环作用域。NameTable 全局 taken-set 允许第二个 `value` 变成 `value_2`；这足够解决类型冲突。直接按名称拆分会误信 debug table；仅移除 records guard 会丢失调试名称；改用 LVT-only `split()` 的同块 store_producing 无法替代现有跨块所有权证明，均不采用。

4. **不扩展 reader。** `DebugLocal.descriptor()` 只在唯一 generic signature 匹配时保证存在，普通 Integer/String LVT 不能据此当成无类型。确切引用类型沿现有 SSA/decoded operations 求得。若有额外 descriptor，它只能佐证身份，不代替值流。

5. **完整类验收保留两个阶段。** 本项先证明 debug 类型冲突消失、none 正文不回退、其他方法和 handled fallback 不被掩盖。完整类仍有拒绝时保留 compile fail/no-runtime，不能关闭整体运行门。独立外层 try 修复后，必须在两项实际最终代码上重跑未手修完整类，双 JDK、两 compiler modes、default/all、全部17 oracle；当前仅局部实现不得代替这个联合里程碑。

## Risks / Trade-offs

- 调试表看似合法却与实际读取不符 → 一一匹配失败时拒绝；不按名称、最近 BCI 或 cast 外观猜测。
- 旧值通过别名留在栈上 → 保留现有 alias survival 与 CFG 证明，并用真实反例验证。
- 名称对了但声明跨作用域 → 永久测试检查两种真实类型与 declaration owners；完整类编译承担实际合法性门。
- 有 debug 与 none 的新增分析计费不同 → 各自核新增预算/取消，无部分计划；P5若有真实变化只更新经独立测量的基线。
- 另一方法仍失败 → 保留原文件、raw stderr 与未运行分类，不删方法，不修补生成 Java，不增加完成计数。

## Migration Plan

root 固化新基线和此设计，Luna 仅在私有副本准备实现 patch 与永久反例；root 审查后独占共享源码和工具链。运行前至少5GiB可用、target至多1GiB、1秒进程组守卫，默认 Rust stack，finally exact cargo clean。门禁通过后提交推送确切产品，接受自身 CI，再更新交接；失败保留证据并修真实原因。
