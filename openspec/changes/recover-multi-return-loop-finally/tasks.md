## 1. 固定形态与有效反例

- [ ] 1.1 在固定 Test5 class 上扩充 probe：分别记录 `c == null`、`first == false`、单/多次循环、`first`/`load`/`toNext`/`close` 抛错的返回值、操作顺序、清理次数与异常身份；以原 class、原 Java 8 源码和固定 JADX 的 `java -Xverify:all` 逐路径一致验收。
- [ ] 1.2 冻结至少五个 verifier 有效近邻，覆盖清理接收者/目标不同、保存值改写、自保护范围扩大、Throwable 改写、循环新增出口或入口；逐一记录 class SHA、`java -Xverify:all` 和 Jarde 基线拒绝，不以不可验证字节码充数。

## 2. 三行双返回证明

- [ ] 2.1 在现有 Guard/Plan 中实现固定两段、三份副本和 handler 自保护的有界证书，逐行核异常覆盖、循环正常边、全部块与进入/退出边；以目标命中和 1.2 的行/CFG 近邻拒绝测试验收，旧 finally 证书保持原约束。
- [ ] 2.2 用 SSA 证明同一清理调用目标及接收者、`null` 与 list 的分别保存/重载/返回、handler 原 Throwable 重抛，以及正文循环内 list 的同一活值；以目标及接收者/目标/值改写近邻、预算/取消测试验收。

## 3. 原子源码交付

- [ ] 3.1 复用普通 Loop、Try AST 与来源映射输出一份 `try/finally`，在 try 内保留提前返回和 do-while/list 返回，finally 中只写一次 `close()`；以无 fallback、完整类 `javac --release 8`、全部目标 BCI 来源和固定路径运行对照验收。
- [ ] 3.2 当循环/局部声明、块所有权、输出预算或取消不能完整证明时回滚并保留物理字节码与异常行；以定向负例、停止和源映射测试证明不会交付半个 try 或静默删除副作用。

## 4. 三方与主线验收

- [ ] 4.1 用 fresh CLI 生成完整 Jarde class-source，和固定原 class、原源码、JADX 对照 1.1 全部路径；`javac --release 8`、`java -Xverify:all` 成功且逐路径操作/结果/异常身份一致，记录工具 SHA 和输出。
- [ ] 4.2 重放 1.2 的所有有效近邻及 Test3、Test4、Test11、Test13、TestEmptyFinally 等已验收 finally 切片；运行 `cargo test -p jarde-java --tests --locked`、workspace check、fmt、OpenSpec strict、diff check，记录结果并清理专用 Cargo target。
- [ ] 4.3 root 独立复核三行表、正常循环/SSA/异常边、全部来源和三方行为，更新 CF-16 账本；只标记固定 Test5 切片，不推断 Test2/9 或所有编译 profile 已恢复。
