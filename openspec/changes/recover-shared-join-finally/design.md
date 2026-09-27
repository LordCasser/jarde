## Context

固定基线为本地 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally.java`。原 Java 8 class SHA-256 为 `24a23e9f96826d8a7ffe104ba31bfc0abb005141572920604f82e13b4b082807`。`test(Object)` 的异常行是 `[5,10)→18 Exception`、`[5,10)→31 any`、`[18,23)→31 any`。BCI 10–12、23–25、32–34 各写一次相同 `this.f = true`；BCI 15/28 的 goto 均至 39，BCI 37–38 重抛原异常，39–43 只在两条正常完成路径运行 `return this.f`。BCI 0–4 的 `this.f = false` 位于 try 之前。固定测试默认断言清理源码出现一次，关闭 finally extraction 时断言三次；两种固定 JADX 输出均可重编、验证和运行。

现有 `guard::prove_shared_finally` 已证明三行 catch-all、三个清理副本、两个有界子正文与原异常重抛，但要求每份正常清理前有 saved return、清理后直接返回；`shared_cleanup_span` 只接受无参静态调用或 static `int` 增量。`region::shared_finally_body` 和 `build::SharedFinallyBuild` 同样按两个 saved return 设置有界恢复。这里没有这两个值，不能放松旧证明到一个无意义的 BCI。`Shape::SharedFinally` 的 `Plan::join` 已是现有续接概念，但该路径当前固定返回 `None`。已有 `Region::Try`/`StmtKind::Try`、实例字段写入、来源映射、预算和 checkpoint 可以复用。参考 JADX `MarkFinallyVisitor` 的候选清理匹配，但以原 class 的效果和异常表判定，不能单凭三个片段字节相似消重。

## Goals / Non-Goals

**Goals:** 让固定 `TestTryCatchFinally.TestCls` 全类无 bytecode fallback、Java 8 重编、`-Xverify:all` 和正常/异常 `check()` 与原类一致；已验收的共享调用型、静态字段型保持通过。

**Non-Goals:** 任意清理语句、复杂 finally body、保存返回形态的扩容、多个具名 catch、`FinallyOnce.escaping`、通用异常 CFG/SSA 变换、对 JADX 文本做后处理。`--no-finally` 只是固定 JADX 测试的对照模式，不作为 Jarde 新选项。

## Decisions

1. **一个共享 finally 路径，两个互斥的完成合同。** 在私有共享证书中明确区分旧的“双 saved return”和新的“双 goto 同一 join”。只用已有 `Plan::join` 表示正常续接；不要把 BCI 39 伪装成任何子正文的 return，也不要在两个分支之间共用未证明的值。若用一个私有判别字段/类型才能避免非法状态，可局限在现有 `Shape::SharedFinally` 与 Builder 状态内，不增公开机制。
2. **清理效果和异常边逐 BCI 证明。** 三份副本均须是连续的 `aload_0; iconst_1; putfield`，写同一当前实例的同一 `Z` 字段，SSA 必须证明三次 receiver 都是该方法的 `this`，常量生产值只被各自写入消费。第一份位于 `[5,10)` 之后、第二份位于 `[18,23)` 之后、第三份位于共用 handler 的 throwable 保存之后；副本及 goto 均不再被上述异常行保护。具名 catch 的异常参数只供 `printStackTrace()` 的已恢复正文使用，catch-all 的原 throwable 经 BCI 37–38 完整重抛。两条正常清理之后只能经单一 goto 至同一个 BCI 39；不得有额外入口、退出或副本中途跳入。
3. **有界正文与汇合点分别认领。** try 只恢复 `[5,10)`，catch 只恢复 `[18,23)`；前置 `[0,5)` 保持在 try 之前。共享证书认领三份清理、两个 goto 和 catch-all handler 的来源，不把 join 39–43 放进证书所有权。Region 经 `Plan::join` 继续写 `return this.f`，且证明 BCI 39 的字段读取由两条正常完成流共享；Builder 在原有 checkpoint 中输出唯一 `finally` 赋值，任何正文、声明、来源或语法闭合失败都整体回滚。来源图须覆盖所有物理指令与三条异常行，不能靠代码文本中“只有一次赋值”替代来源校验。
4. **首片边界保持窄。** 只接纳当前实例的布尔字段常量写入和固定三行/单具名 catch/单 join 布局；其它实例字段、值、receiver、额外效果或控制流需独立证据。verifier 有效的 BCI 24 `iconst_0` 变体必须拒绝折叠；再构造 verifier 有效的异常范围或 join 变体以检验边界。另以不改 `test(Object)` 布局的 verifier 有效 `Error` 变体实际进入 catch-all，核清理一次及原异常重抛。既有双返回调用/字段型测试与单出口 finally 均复跑。

## Risks / Trade-offs

- [把 goto 前的赋值留在 try/catch 又放进 finally，执行两次] → 子 Region 物理范围截至清理前，三份副本仅由一份 finally 语句和派生来源认领，并运行正常/异常路径。
- [误把异常 handler 的赋值当作正常路径汇合] → 分别核三条异常行、handler 的 throwable SSA 和唯一重抛；join 只接受两条正常 goto。
- [把后续 `return this.f` 丢掉或错误放入 finally] → `Plan::join` 保留 BCI 39 单独续接，完整类重编与两路 `check()` 验收。
- [新形态破坏旧双返回证书] → 完成合同互斥，旧调用型/静态字段型的证书、负例、来源与三方运行复测。
