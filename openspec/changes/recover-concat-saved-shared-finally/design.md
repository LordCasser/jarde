## Context

见 `proposal.md`。固定 `FinallyOnce.handled(Z)String` 原 class SHA-256 为 `3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2`，异常行依次为 `[4,21)→31 IllegalArgumentException`、`[4,21)→65 any`、`[31,55)→65 any`。三份静态 `cleanupCount++` 是 BCI 21–26、55–60、66–71；正常返回 `ldc` 18 → store 20 → load/return 29/30；catch 返回的 `StringBuilder` 32–51 → store 54 → load/return 63/64。原异常于 BCI 65 保存，经 BCI 74/75 原值重抛。

现有 `prove_shared_finally` 已检查三行、三份清理、两份保存返回、原异常、边与区域所有权，`shared_cleanup_copies` 已接受静态整数增量。已定位的一处 Guard 门槛是保存返回生产者 `Operation::Push` 白名单：catch 的直接生产者 BCI 51 是 `StringBuilder.toString()`。通过此门槛后仍须核 Region/Builder 的完整闭合。`concat::Plan` 已在 Region 恢复前由同一 SSA/Operations 计算，并在 Builder 中负责把受证链写成一条拼接表达式；重复实现拼接识别会产生相互矛盾的证明。固定 JADX 的正常路径输出 `normal:2`，原 class 为 `normal:1`，故原 class 是行为依据。

## Goals / Non-Goals

**Goals:** 在现有共享 finally 证书内证明这个保存返回值的受证拼接生产者和异常保护时机，继续由原有 Try/Builder 写两个返回及一份 finally；三方完整类验收原运行行为。

**Non-Goals:** 放宽所有 `Invoke` 作为返回生产者、修复一般字符串拼接、改变现有异常行或清理副本证明、处理 `escaping()` 的单行 catch-all、Test14 分支清理、JADX 自身语义错误。

## Decisions

1. **使用同一轮的拼接计划，不另建识别器。** 将 `report` 已计算的只读 `concat::Plan` 送到 Region 的共享候选入口/Guard；只在第二保存返回的直接生产者 BCI 51 命中 `value_at`，其 `tail` 与生产者相同、`owned` 属于 `[31,55)` 且不跨过清理起点 BCI 55 时接受。正常路径仍用现有字面量证明。相比直接允许任意 `Invoke`，此边界保留 `concat` 模块的转换、顺序、别名与 statement-free 检验。
2. **保存值仍按 SSA 和物理位置证明。** 拼接尾部只有一个 stack 结果，直接流入 store BCI 54，且该结果在方法内无其他 stack 消费者；store/load/return 已有定义-使用与唯一终端检验。拼接所有可能抛错指令仍处于具名 catch `[31,55)` 的 catch-all 保护内，生成的 return 表达式在 catch 中、finally 前求值。检查整条链物理 BCI，而非只检查尾部，防止提前/延后求值或异常优先级改变。证明失败保留原整体拒绝。
3. **复用 Region/Builder。** 已有 `Shape::SharedFinally`、`shared_finally_body` 与 `shared_saved_return` 均保留；若 Builder 的现有值渲染不能在保存点原子写出拼接，局部修复该接缝而不新增 AST 或通用返回重排机制。独立核检查物理来源、三份清理只出现一次和全类无未闭合引用。
4. **完整类运行优先于 JADX 文本。** 冻结原/JADX/Jarde 三方完整 Java 8 输出；同布局最小类替换不相关方法后逐 BCI 比对目标，分别重编并 `java -Xverify:all`。正常、具名 catch 与异常的可观察值以原 class 为真值；若原固定类无法制造某异常路径，则以保持目标方法的受控 fixture/有效近邻验证，不把编译成功当行为正确。

## Risks / Trade-offs

- [将可能抛错的拼接移出 catch 范围] → 整链 BCI 受保护、输出求值点、异常路径运行三重核验。
- [同一物理拼接既被单独写语句又被返回读取] → 复用 `concat::Plan::owned` 与单一保存消费者，核 source map 和完整方法输出。
- [测试只看计数掩盖返回值或异常对象变化] → 分别核返回文本、计数、类型/消息/对象身份及清理抛错优先级。
- [与 exception-only catch-all 分支触碰同一核心文件] → 后者先合入；本 change 基于合入后的主线实施并独立回归。
