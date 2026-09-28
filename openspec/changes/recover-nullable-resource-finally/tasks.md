## 1. 固定语义和有效反例

- [ ] 1.1 扩展固定 Test9 probe，记录资源存在/缺失、赋值后 Scanner 构造或读取抛错、正常及异常清理 `close()` 抛错时的内容、调用次数与异常身份；以原 class、原 Java 8 转写及 JADX DX 的 `java -Xverify:all` 逐路径对照验收，同时保留 Java-input 漏关负向记录。
- [ ] 1.2 冻结至少四类 verifier 有效近邻：两份清理的判断/接收者/目标不同、返回值或 Throwable 改写、自保护行扩围、额外入口/出口；以固定 SHA、`java -Xverify:all` 及当前安全拒绝记录验收。

## 2. 可空资源证书

- [ ] 2.1 在现有 Guard/Plan 内有界证明双行同一 catch-all、handler 绑定自保护、两份非空清理、全部正常/异常边和物理块所有权；以固定目标命中、1.2 行/CFG 近邻拒绝及旧 finally 证书回归验收。
- [ ] 2.2 以 SSA 核 local 1 的 null 初始化、资源赋值及两份条件关闭的同一到达值，local 3 保存/返回及 local 4 原 Throwable 重抛；以目标、值/接收者/目标改写近邻和预算/取消测试验收，不能只因槽号或 `close` 名称相同而合并。

## 3. 完整源码和原子来源

- [ ] 3.1 复用现有 Try/If AST 与局部声明输出一份 `try/finally`，使资源赋值、Scanner 操作和字符串返回保留次序，finally 中仅一处 `if (input != null) input.close()`；以完整类 `javac --release 8`、目标所有 BCI 来源、无 fallback 和行为对照验收。
- [ ] 3.2 对局部绑定、块/边、来源、预算或取消的不完整证明原子回退；以定向拒绝及停止测试确认没有半个 try、静默漏关或残缺来源映射。

## 4. 四方重放与主线验收

- [ ] 4.1 用 fresh CLI 重放固定原 class、原 Java 8 转写、JADX DX 与 Jarde 完整源码，全部有效路径 `javac --release 8`、`java -Xverify:all` 的内容/清理次数/异常身份一致；另外固定 Java-input 输出在有效资源上 `close=0`，不把它算正例。
- [ ] 4.2 重放所有 verifier 有效近邻和既有单行、TestEmptyFinally、Test3/4/11/13/16 等 finally 切片；运行 `cargo test -p jarde-java --tests --locked`、workspace check、fmt、OpenSpec strict、diff check，并清理专用 Cargo target。
- [ ] 4.3 root 独立复核双行异常表、资源 SSA/条件关闭、CFG/来源和完整源码行为，更新 CF-16 账本；只标记固定 Test9 JVM 切片，不外推 JDBC/TWR 或 JADX Java-input 的错误输出。
