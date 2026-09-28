## Context

[固定 Test5 证据](../../evidence/java-syntax-2026-09-28/cf16-test5-multi-return/README.md)将受保护范围钉为 `[21,34)→93`、`[44,83)→93`、handler 绑定自保护 `[93,95)→93`。第一段把 `null` 存入 local 5 后在 BCI 36 清理并于 43 返回；第二段含正常可达回边 `76→53`，把构造好的 list 从 local 5 存入 local 6 后在 BCI 85 清理并于 92 返回；handler 93 保存 Throwable、97 清理、104 重抛。BCI 0–19 的早退和资源求值在 finally 之外。Jarde 当前在 handler Guard/Region 和跨 fallback 的 local 4 声明上拒绝。

现有 `Shape::Finally` 只含一处受保护段与正常/异常两份副本；`SharedFinally::SavedReturns` 虽有两处保存返回，却绑定一处具名 catch 与其 catch-all，并假定两个正常副本对应 try/catch 完成。Test5 没有具名 catch，拥有两个互斥的正文段、三份副本和 handler 自保护行。Test11 的异常 handler SCC 也不是本方法的正常可达正文循环。这里应复用已有物理证据、Region、AST、来源和预算链，不把上述不等价形态强塞进旧证书。

## Goals / Non-Goals

**Goals:** 对固定三行布局证明两处保存返回值及第三处异常完成；恢复 `try/finally` 内的提前返回和 do-while，保持 `close()` 抛错覆盖、求值顺序和每个物理 BCI 的来源；对 verifier 有效近邻及预算/取消安全拒绝。

**Non-Goals:** 任意编译器 lowering、DEX 输入、Test2 的 void 三循环布局、Test9 的可空清理，或把同名 `close()` 直接视为等价副本。不修改调用方/CLI 协议和现有已验收证书。

## Decisions

1. **独立的窄物理证书，复用现有恢复层。** 在 Guard 中为两段正文、三个清理副本及一条自保护行增加有界完成布局，而不新增 pass 或通用 IR/AST 节点。两段必须互斥且同到 handler；第三行只保护 handler `astore`，不能覆盖清理。遍历所有 canonical normal/exception/call/return 边，拒绝额外入口、出口和不完整块所有权。复用已有 `shared_cleanup_copies` 一类的调用目标/SSA 比较思路；`Shape::Finally` 与 `SharedFinally` 的现有字段不应被改成含大量无效选项的联合体。另一种改造是把所有 finally 形态合成一个高度参数化的通用计划，但当前证据不足以定义正确的不变量。
2. **在清理前后证明三个独立的完成值。** 第一段的 `aconst_null→astore 5→aload 5→areturn`、第二段的 list 构造及循环累加 `→astore 6→aload 6→areturn`、handler 的 `astore 7→aload 7→athrow` 均按 SSA 生产/消费、局部重用与跨循环活性核验。三次 `D.close:()V` 必须是同一调用目标，接收者指向同一 local 4 值且无额外可观察消费者。不能把 `List.add`/`toNext` 的副作用或循环回边吸收进清理。`p(a)`/`b.f(c)` 与 `c == null` 留在 finally 外；清理自己的异常覆盖 pending return/Throwable。
3. **Region 与 Builder 在完整所有权后一次提交。** 沿现有普通 loop 和 protected-body 结构把两个 return 写在一份 `try` 内，finally 只写一次；源映射将两个正常副本、handler 绑定/重抛和三行异常表归到真实 `try/finally` 来源。局部声明、循环区域、所有物理块和输出预算不能同时成立时，回滚 visited/局部规划并原子保留解释性 fallback。参考固定 JADX 对相同清理副本的寻找顺序，但不沿用其按指令相似性消隐的判定。
4. **物理和行为验收分层。** 固定 Java 11 class 为语义基准；原转写/JADX/Jarde Java 源用 `javac --release 8` 重编，再以 `java -Xverify:all` 对照七条既有行为路径，并扩充 `first`/`toNext` 抛错。private `p(A)` 在 Java 11/8 的 `invokevirtual`/`invokespecial` 差异只按已记录的编译器 profile 解释，不要求字节级同一 class。有效负例逐一 verifier 验证并保持不发布 finally。

## Risks / Trade-offs

- **两段保护与循环被误并成一个范围** → 固定三行逐 BCI 覆盖、正文回边、互斥完成和全部 CFG 边；范围扩大/改入口的近邻必须拒绝。
- **局部 5 同时是早退值与 list，local 4 跨清理副本** → 以 SSA 与词法声明规划同时核验，禁止按局部槽号直接合并值；失败原子回退。
- **`close()` 抛错覆盖原异常** → 运行清理失败覆盖返回值、body 错误的路径，并验证 Throwable 身份和调用次数。
- **窄证书只覆盖一个 lowering 家族** → 在 CF-16 账本标记固定 Test5 切片，Test2/9 及其他 profile 仍作为独立任务。
