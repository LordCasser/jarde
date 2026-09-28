## 1. 固定行为与有效反例

- [x] 1.1 扩展固定 TestEmptyFinally 行为探针，记录成功、`IOException` 和同一运行时异常对象的关闭次数/终局；重编原 class 转写与固定 JADX，逐条通过 `javac --release 8`、`java -Xverify:all` 与固定物理 class 对照验收。
- [x] 1.2 构造并冻结至少三类 verifier 有效近邻：handler 有 effect 或更换 Throwable、异常行范围/顺序变化、额外入口或出口；逐个以固定 class SHA、`java -Xverify:all` 和当前安全拒绝验收，不用 verifier 无效变异充数。

## 2. 透明行证书

- [x] 2.1 在既有 catch 判定内有界证明两行同范围、具名优先、透明 `astore; aload; athrow` 的同一 Throwable SSA，以及全部异常/正常边和块所有权；以固定正例命中和 1.2 全部近邻拒绝的定向测试验收。
- [x] 2.2 将已证透明 handler 的物理 BCI 与异常行作为普通 catch 形态的显式附属证据，且不放宽已有 `finally_copy` 或单行 catch-all 规则；以普通 catch、既有 finally 切片、预算/取消测试验收。

## 3. 普通 try/catch 结构与来源

- [x] 3.1 复用 Region 与 Try AST 输出受保护的 `close()` 和唯一 `catch(IOException)`，只在全部相关块可呈现时消费透明 handler；以固定目标无 fallback、无 `finally`、完整类 Java 8 重编及目标全部 BCI 来源验收。
- [x] 3.2 对正文/handler 构建失败、来源缺口、预算耗尽与取消实行原子回退；以定向测试证明不会交付半个 try、静默删去 `close()` 或把未证的 Throwable handler 当成源码 catch。

## 4. 三方重放与主线验收

- [x] 4.1 用 fresh CLI 重放固定原 class、原 Java 8 源码、固定 JADX 与 Jarde 完整源码；三条路径的关闭次数、异常身份和 `java -Xverify:all` 行为一致，Jarde 全类 `javac --release 8` 通过且没有 recovery/fallback 标记。
- [x] 4.2 重放 1.2 近邻、普通 catch/单行 catch-all、已有两至五行 finally 切片；运行 `cargo test -p jarde-java --tests --locked`、`cargo check --workspace --locked`、fmt、OpenSpec strict、diff check，并清理专用 Cargo target，记录结果。
- [ ] 4.3 root 独立复核两行异常表、SSA、边闭合、来源和三方运行，更新 CF-16 账本；只标记固定 TestEmptyFinally 切片，不外推其他空 finally lowering。
