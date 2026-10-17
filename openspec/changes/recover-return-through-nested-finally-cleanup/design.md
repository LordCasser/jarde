# 设计

## Evidence gate

首先对封存的 `EX.class` 中 `nestedFin(I)I` 做只读探针，沿用现有 `analyze_method_ir` 测试路径。记录 canonical 完整性、handler ordinal/range、normal/exception edge、throw site、block、SSA 定义/使用/phi、逐指令 effect 和 handler 集合。尤其证明 BCI 4 保存的返回值就是 BCI 35 读取的值；每个 catch 绑定的 throwable 都只到达对应 reload/rethrow；确认哪些 println 调用可能抛错及各自受哪条 row 保护。若证据不完整或不符合此有界形状，则保留 fallback。

在此探针前不选择或新增 certificate variant。取证后，先检查现有 `Shape::Finally`、`FinallyCompletion::SavedReturn`、Region tree 与 Builder 能否在保持物理 ownership/origin 的同时表达正常清理及其嵌套异常清理。现有 `NestedCleanupFinally` 是固定的 Test4 IOException-catch certificate，不是通用嵌套 finally 机制。普通 Finally Builder 当前将正常 cleanup 作为扁平 `body_range` 读取；只有探针和具体 Region walk 证明这是最小缺口后，才增加私有 structured cleanup tree。

## Behavioral invariants

在清理前只计算并保存一次 `10 / n`。正常完成时按 `println("in")`、`println("out")` 顺序各执行一次；只有两者都正常完成才返回保存的整数。除零时，两个清理层各执行一次；若清理成功，保留并重新抛出原始算术异常。运行时检查 ArithmeticException 类型和消息；异常对象身份只能由静态字节码/SSA 值链证明。

若 `in` 抛出已知 `RuntimeException` 或 `Error`，仍执行一次 `out`；若 `out` 正常完成，则逃逸的是 `in` 异常。若 `out` 抛错，其确切对象覆盖待返回值、算术异常或更早的 `in` 异常。由单独的记录型 `PrintStream` 在 println 调用处注入异常；目标类本身不增加 hook/helper method。运行完整矩阵：`n ∈ {2,0}`、`in ∈ {none, RuntimeException, Error}`、`out ∈ {none, RuntimeException, Error}`，并断言 `IO` trace 恰好出现一次。

这些规则来自 Java finally 完成语义，但本例的字节码实现仍须由实际异常 edge、effects 和 SSA identity 支撑。动态 driver 只补充静态证书，不能替代它。

## Presentation and ownership

尽可能复用现有 Try/finally AST 与 SavedReturn completion。每条物理指令必须恰好归属一次，带正确 direct/derived origin。正常 cleanup 与异常 cleanup copy 必须具有可证明的嵌套结构及匹配操作；每个 handler 都须保留绑定异常身份与覆盖顺序。只有 body 和 cleanup 均无 fallback、未声明 local 或 ownership 缺口后，Builder 才能原子 claim/publish。任一 edge 不匹配、额外前驱、未证明 throw site、finally 非局部完成、预算停止或取消，均保留原拒绝。

## Validation and handoff boundary

以封存的完整 EX class 和独立完整类 fixture 验证；后者分别由 javac 8 与 javac 23 `--release 8` 编译。两 compiler leg 都保留完整类源码及所有方法。生成的完整源码原样重编，只运行重建产物；对照返回值/异常类型与消息、注入对象身份和精确打印 trace 与原 fixture oracle。覆盖 compiler × runtime JVM (8,23) × profile (`default`,`all`) 的 2×2×2 矩阵、固定 JADX 对照、静态 source/BCI 与 full-source 检查；保留真实 JADX 输出和失败。

所有 Rust 执行保留至少 5 GiB 磁盘余量、仓库 target 不超过 1 GiB、每秒检查并中止超限进程组、默认 Rust stack，并在 finally 路径执行 exact cargo clean。Java cleanup 必须恰好一次并保持顺序。精确固定实现 product commit，并只附该 product 自身 CI 收据。覆盖 early/late budget、cancellation、deadline stop、guard/resource regression、CLI/public entry 与项目要求的本地验证。shared OpenSpec change 已由 root 创建；root 可现在验证并派发 implementation，集成仍须等待 `finThrow` 稳定。parent-wide final gates 在集成后运行。

## Root review: 已执行探针与最小机制选择

`results/initial-facts-root-v1/capture.tar.gz` 保存 e575 上实际执行的完整 Facts/SSA/effects 与 Region 输出、探针源码及资源/清理收据。图为 Complete，六个 canonical block；四条 catch-all 的范围、throw site 归属以及 return/caught 对象的 SSA store→reload→terminal 链已核对。BCI57 自保护行仅覆盖不可抛 store，不允许以此推断其它 cleanup 不可抛。当前拒绝发生于 guard，Builder 尚未消费该形；首轮 helper 的 generic-copy None 只对应第一行，不冒充所有行的遍历结果。

canonical block0 同时包含外层 body `[0,5)`、inner body `[5,13)`、normal inner finally `[13,21)` 与 goto21→35。它们不能拆成分别认领同一 block 的普通 Region 子节点。`FinallyFallthrough` 表示第二份条件 cleanup，也不能拿来代替指令级 nested cleanup。

决定保留外层公开 `Shape::Finally` 与 `SavedReturn`，先在既有私有 `Plan` 数据上增加只供 crate 内消费的可选有界 nested-cleanup 描述，供 Builder 构造现有嵌套 `StmtKind::Try`。描述只保存呈现必要的 inner-body/inner-finally 指令范围；异常副本、四行、goto/return、throw-site/effect、SSA identity、所有权与无额外边必须在私有 proof 中完整证明，保存在原 facts/origins 中。外层 Plan 一次认领全图，禁止重复 block owner；不创建通用递归 cleanup tree、公开 AST/Region/Shape variant 或新的 solver。新增遍历/分配须沿已有收费与停止通路。非结构化路径也必须建立完整 Builder checkpoint，任意 body/cleanup/return 失败原子恢复。

本选择只允许有界两层落穿 cleanup；finally 自身 return/throw、exception-only、额外正常入口/出口、不匹配副本或缺失值证明均保留拒绝。实际实现若需要扩展此边界，先回到 root 审查，不得暗中放宽。
