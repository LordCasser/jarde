# 2.3c 验收：同名数组构造 helper 的类级原子投影

`ArrayCtorSubject.arrayCtor()` 的 Java 8 字节码以 `invokedynamic` 引用同类合成方法 `lambda$arrayCtor$0(I)[I`。方法级恢复保留该物理调用；类级恢复只有在同次方法表、Code、常量池和 BootstrapMethods 证明该 helper 的全部可达用途都属于已证站点后，才从同次 typed AST 重发 `int[]::new`，并同时省略类源码里的 helper 声明。方法报告及物理方法表不变。这里没有照搬 JADX 1.5.6 解析到 helper 后立即隐藏声明的做法。

本次准入形状刻意限定为一个直接 `return` 的 Lambda 站点，唯一 SAM 参数须是 helper 调用的唯一实参，实际已拼写的非泛型函数式返回类型须足够精确。若站点、同类其它用途或后续类级投影不能同时证明，则保留原始 lambda 调用与 helper。多个站点、赋值和嵌套表达式留给 2.3d。

主分支独立复跑：

- `cargo test -p jarde --test p3_immediate_functional_receivers --test class_source --locked`：6 + 47 项通过。正例用 `javac --release 8` 重编完整恢复类，原 class 与恢复类经 `java -Xverify:all` 执行都输出 `7`。冻结 `BoxedSamProbe` 的 raw `Function`、捕获长度的 `Supplier<int[]>` 均不投影；预算少一个 `IrItems` 时普通调用和物理 helper 同时保留。
- `cargo test -p jarde-java --lib --locked`：192 项通过，包含数组 helper 的精确 Code/适配证明。
- `cargo fmt --all -- --check` 与 `git show --check` 通过。类级用途普查的直接 invoke/LDC MethodHandle 身份负例包含于库测试。严格 Clippy 的全 targets 门仍被仓库既有 lint 和无关测试符号错误阻断；本项不据此声称 3.2 完成。

2.3c 的类级正例不替代 3.1 要求的边界 Integer、null 拆箱和负数组长度执行矩阵；3.1 与 3.2 继续开放。
