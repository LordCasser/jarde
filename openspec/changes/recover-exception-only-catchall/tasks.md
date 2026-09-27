## 1. 冻结输入与可验证近邻

- [x] 1.1 为固定 `FinallyOnce.escaping()V` 记录原 class、异常行和 pinned JADX 输出，并制作只替换无关 `handled()` 的同 BCI/opcode/异常行完整 Java 8 验收类；脚本每次重编并比较目标方法，原/JADX 两份类 `java -Xverify:all` 均输出 `state:1`。
- [x] 1.2 制作 verifier 有效的正常出口、异常范围扩/缩、竞争行、外部 handler 入口和异常值改变近邻；逐个冻结原 class 的路径行为或验证状态，使后续拒绝断言有实际输入。

## 2. 单行证书与普通 catch 所有权

- [x] 2.1 在现有 catch 候选入口增加私有单行 catch-all 证明，核仅异常完成、逐 BCI 覆盖、无竞争边、handler 原值重抛及清理效果位于保护外；用固定正例和 1.2 的有效近邻定向测试核拒绝边界。
- [x] 2.2 对已证 catch-all 用显式类型事实向现有 CatchSite/CatchClause 表示 `java.lang.Throwable`，不依赖不存在的 CP index；用具名 catch/multi-catch 原测试与本例 header/source origin 核不混淆类型。
- [x] 2.3 复用现有 Region Try/Catch 与 Builder 构造完整正文、唯一块 owner、异常 handler 和物理来源；用固定方法 BCI/异常行、清理一次与预算/取消/输出预算失败测试核原子性。

## 3. 三方验收与回归

- [x] 3.1 用 fresh CLI 对原验收类、pinned JADX 和 Jarde 完整 Java 8 源码分别重编，`java -Xverify:all` 对照异常类、消息、对象身份和计数；所有有效错误近邻不得生成错误 catch/finally，固定 `handled()` 仍单列。
- [ ] 3.2 跑普通具名 catch、multi-catch、TWR/monitor、Test12/Test13/共享 finally 回归和 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、格式、OpenSpec strict、diff check；清理专用 Cargo target。
- [ ] 3.3 root 独立审阅 Guard 证书、类型表示、Region/Builder 所有权、负例和三方运行，写主线验收记录并更新 CF-16 清单；不把整个 CF-16 标为追平。
