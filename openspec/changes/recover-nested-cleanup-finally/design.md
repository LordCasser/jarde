## Context

固定 class SHA-256 `2bf1b8932e521aade29fa1d562269903f9b8e9942f86981eb458d4f552d67060`，方法 BCI 0–57。四行按物理顺序是 `[22,31)→34 IOException`、`[17,22)→38 any`、`[40,49)→52 IOException`、`[38,40)→38 any`。正文写调用在 19；正常清理 `close` 23、`delete` 27 后跳 57，named catch 34–35 也跳 57；异常副本先保存原 Throwable 于 38，再 `close` 41、`delete` 45，named catch 52 直接汇入 54，最后加载并重抛原 Throwable。头部 `File.createTempFile` 与 `new FileOutputStream` 在异常保护区外。Java 8 转写的 BCI/opcode 与四行表逐项吻合。

现有 `shared_finally_candidate` 对四行直接调用 `prove_two_catch_return_finally`；该证书要求两个同范围 named catch、四个单调用清理及 return，与 Test4 的两个**分别覆盖清理副本**的 named catch 不同。普通 `Shape::Finally` 的 `normal_cleanup` 由 `body_range` 当直线语句输出，无法把其中的 named catch 附着于 `finally` 内部。普通 named-catch region 单独先进入时又看不到两个清理副本的联合所有权。因此需要一个有界的四行 Guard 形状；AST 和 Java writer 已能表达目标结构，不需要新语法节点或通用异常图合并。

## Decisions

1. **四行一起证明，先于普通 catch 选择。** 在现有 `shared_finally_candidate` 的四行分支先尝试 Test4 证书，未命中仍调用原 `prove_two_catch_return_finally`，保持其它形态原判。核四行的顺序、catch 类型确为 `java/io/IOException`、精确覆盖范围与 handler，特别是 `[38,40)→38` 只保护原异常绑定，不能把异常副本的清理调用误认为再次进入 finally。逐 BCI 检查其它覆盖行不存在。
2. **两份清理按 SSA/符号事实等价。** 在两个副本中各证一条 `OutputStream.close():void` 后的 `File.delete():boolean`，返回值只 `pop`；正常和异常副本的 `OutputStream`/`File` 接收者各指向头部同一已绑定物理值，调用目标、顺序和次数一致。正文的 `write(1)` 唯一且在 catch-all 保护范围内。`IOException` handler 的局部只作绑定而无读取；正常两个出口都汇入 return，异常两个出口都汇入原 Throwable 的 load/throw，按 SSA 核原 Throwable 未改写。每个 CFG 正常/异常边和 Plan 所有 block 必须有唯一归属，不能仅靠 BCI 位置或文字相同删副本。
3. **整组构建并原子发布。** Guard 证书持有正文、两份清理和两个 named catch 的物理范围；Region 先闭合所有权，再把正文及清理内层 catch 交给既有 `Region::Guard`/`Region::Try` 表示。Build 复用 `StmtKind::Try` 构造外层 `finally_body` 中的内层 `try/catch`，只在正文、接收者表达式、catch 头/空体和来源全可呈现时省略异常副本。失败、预算耗尽或取消均回滚，物理 class-source 保留明确拒绝。所有目标 BCI（包括 `pop`、goto、原异常保存/重抛和两份清理）有 source-map 锚点。
4. **证据分层。** 固定 Test4 目标原类、JADX、Jarde 完整源码均 Java 8 重编、`-Xverify:all`，但只能运行正常路径。独立 `Control` 注入输出流和文件，在四类路径比较三方事件日志与终局异常；它的字节码布局不同，不充当 Test4 物理形状的证明。变异负例应优先保留 verifier 有效并证明调用或异常表变化确实改变效果，不能用无效 class 拒绝冒充健壮性。

## Risks / Trade-offs

- `IOException` catch 中的 `delete()` 只在 `close()` 成功时执行；若把两调用拆成 finally 后续语句，异常路径会多删文件。证书必须保持整个内层 `try` 的覆盖范围。
- 清理 `RuntimeException` 应覆盖正文原异常；若异常副本的 self row 被误当清理重试，会重复调用。异常边和原 Throwable 的 SSA 身份是发布门。
- 固定目标无法触发清理异常；control 提供源级语义对照，物理等价仍靠 classfile/SSA 证明。验收报告须分别陈述这两种证据。
- Test11 也修改 FINALLY 分派；并行分支集成后须重跑两套固定样例与既有二、三、四、五行证书，按语义合并分派顺序，不靠文本冲突解决作为验收。
