## 1. 固定语义与有效反例

- [x] 1.1 扩展固定 Test3 证据的 stand-in 和 runner，保持原 `test` BCI/opcode/四行表不变，重放正常、`load`/visitor/logger/`unload` 抛错及异常覆盖；以原 class 和固定 JADX Java 8 完整源码 `-Xverify:all` 的逐路径事件/终局异常对照验收。
- [x] 1.2 冻结至少错清理接收者/目标、错自保护范围、改写原 Throwable、额外 loop/catch 入口等 verifier 有效近邻；以固定 SHA、`java -Xverify:all` 和当前安全拒绝记录验收，语义不同的 control 不冒充目标字节码。

## 2. 四行共享 finally 证明

- [x] 2.1 在现有 `SharedFinally` joined 路径有界证明三条正文/catch catch-all 行和第四条只保护 handler 绑定的自保护行，核完整异常/正常边及所有权；以固定目标命中和 1.2 的行/入口变异拒绝测试验收，不放宽旧三行、Test4 或 Test17 证书。
- [x] 2.2 证明三份 `ClassNode.unload:()V` 是同一入口对象上的同一实例调用，并核正文循环、具名 catch 日志、原 Throwable 的 SSA 生产消费与每个可抛效果的异常覆盖；以目标正例及错接收者/目标/重抛近邻和预算/取消测试验收。

## 3. 方法级结构输出

- [x] 3.1 复用现有 Region、`SharedFinally` joined completion 与 Try AST 输出一次 `try/catch/finally`，包含正文迭代和 catch 日志；以目标 `test` 无 fallback、独一 finally、方法级 Java 8 重编和全部 BCI source map 验收。
- [x] 3.2 对声明绑定失败、物理块未归属、输出预算与取消实行原子回退；以定向 AST/来源/停止测试证明没有半个 try 或被静默删去的清理，并在报告中保留独立 `<clinit>` 的 fallback 质量。

## 4. 三方与主线验收

- [x] 4.1 用 fresh CLI 把恢复的 `test` 放入明确标注的原类声明/stand-in 方法级 harness，重编并以 `java -Xverify:all` 对照 1.1 全部路径；另记录原/JADX 完整类可编译和未经补齐的 Jarde class-source 因 `<clinit>` 独立缺口仍不能整类重编，不混淆证据等级。
- [x] 4.2 重放 1.2 全部有效近邻以及旧三行 shared finally、Test4 四行嵌套清理、Test11 双行循环和普通 catch/loop；运行 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、fmt、OpenSpec strict、diff check，并清理专用 Cargo target，记录结果。
- [x] 4.3 root 独立核四行异常/CFG/SSA/来源和方法级三方行为，确认 `<clinit>` 债务仍独立，更新 CF-16 账本；只标记 Test3 方法切片，不宣称完整文件或整类源码已追平。
