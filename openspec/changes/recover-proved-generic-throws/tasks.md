## 1. 冻结三方证据

- [x] 1.1 固定 `-g`/`-g:none` 的 Java 8 `throws E` 原源码、JADX、Jarde 对照，核对 `javap`、原类 `-Xverify:all`、类与同一强类型调用方重编、反射和哈希；以 `generic-throws-signatures/replay.sh` 退出码 0 及 `analysis.md` 验证。
- [x] 1.2 构造至少一个保持物理 descriptor/`Exceptions` 不变且 JVM 可验证的签名矛盾负例，记录 reader 拒绝原因、Jarde 物理回退及 `java -Xverify:all` 结果；用可重放命令或定向测试验收。

## 2. 恢复已证明的无正文泛型异常声明

- [x] 2.1 在现有普通方法候选里将类级已证、异常上界在确定 JDK 根集合中的 `throws` 变量按签名位置拼写；其它异常位置维持已证 Class 类型，缺少后缀时保留物理属性，有正文或未证类型局部拒绝；以定向 Rust 测试及 `cargo fmt --all -- --check` 验收。
- [x] 2.2 定向测试覆盖正例两种调试变体、完整类与强类型调用方 `javac --release 8`、反射及行为、异常变量未绑定/擦除不符、无后缀、预算/取消与 essential/all；以测试通过和原/JADX/Jarde 三方结果可复核验收。

## 3. 独立验收

- [x] 3.1 root 独立审读作用域、异常合法性、位置拼写和原子发布，重放正反例，运行 reader/类源码/泛型定向回归、格式、适当 Clippy、`git diff --check` 与 `openspec validate recover-proved-generic-throws --strict`，记录结果、后续自定义异常与有正文边界，并清理私有 Cargo target。证据：[verification-root.md](verification-root.md)。
