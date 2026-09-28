## 1. 具名 catch 行为与有效近邻

- [ ] 1.1 扩充固定 Test7 debug/no-debug probe，保留 `test(Object)` 的 32 条指令/四行表，另让 `exc` 在独立 probe 中抛 `Exception` 以覆盖具名 catch；以正常、具名 catch、AssertionError 三类路径的原 class/原源码/JADX `javac --release 8`、`java -Xverify:all` 的返回值、`f` 增量和异常身份一致验收。
- [ ] 1.2 冻结至少五个 verifier 有效近邻，覆盖错字段/接收者/增量、返回值改写、Throwable 改写、自保护行扩围及额外入口；逐个记录 SHA、`java -Xverify:all` 与当前 Jarde 安全拒绝，不用不可验证字节码充数。

## 2. 四行字段清理与合流值证明

- [ ] 2.1 沿既有 `SharedFinally` 的四行路径核具名行、两段 catch-all、自保护 store、三份 `f++` 的完整异常/正常边及块所有权；以固定目标命中、1.2 的 CFG/行近邻拒绝和 Test3 等旧证书回归验收。
- [ ] 2.2 按 SSA 证明三份 `getfield; iconst_1; iadd; putfield` 是同一入口接收者/字段的等价增量，两个 local 2 定义在尾部合流并由一次 return 消费，handler 重抛原 Throwable；以错字段/值/原异常近邻、预算/取消测试验收。

## 3. 方法与完整类交付

- [ ] 3.1 复用普通 Try/Catch/Finally/Return AST 输出一次 `try/catch/finally`、一次 `f++` 和一次合流返回；以固定 debug/no-debug 目标无 fallback、全部 BCI 来源和 Jarde 完整类 `javac --release 8` 验收。
- [ ] 3.2 当声明绑定、块归属、异常边、来源、输出预算或取消不完整时原子回退；以定向负例/停止测试证明不会丢清理、副本重复或交付半个 try。

## 4. 四方与主线验收

- [ ] 4.1 用 fresh CLI 对固定物理 class、原源码、固定 JADX Java-input、Jarde 完整源码重编运行 1.1 全部路径；`java -Xverify:all` 结果/字段增量/异常身份逐路径一致，并单列默认 DX 集成测试通过记录。
- [ ] 4.2 重放所有 verifier 有效近邻及既有 Test3、4、11、13、16、TestEmptyFinally 等 finally 切片；运行 `cargo test -p jarde-java --tests --locked`、workspace check、CI 精确 Clippy、fmt、OpenSpec strict、diff check，并清理专用 Cargo target。
- [ ] 4.3 root 独立复核四行表、三份字段读写 SSA、合流返回/Throwable、全部 BCI 来源和行为矩阵，更新 CF-16 账本；只标记固定 Test7 Java-input 切片，不外推默认 DX lowering。
