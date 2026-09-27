# DT-26：捕获局部值和 `this` 的 lambda

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `java8/TestLambdaExtVar.java`、`TestLambdaExtVar2.java` 与 `TestLambdaInstance.java` 检查 lambda 体内捕获值且不出现 `lambda$`。本次先隔离两个 Java 8 降低形态：[CaptureCases.java](CaptureCases.java) 的 `x -> x + base` 捕获一个 `int` 参数，`() -> this.number() + delta` 同时捕获 `this` 和一个 `int` 参数；[Runner.java](Runner.java) 验证二者结果。

[replay.py](replay.py) 固定以 `javac --release 8 -g:none` 编译原源码，再由固定 JADX 与 Jarde 各自反编译；完整源码分别重编，能运行的产物再用 `java -Xverify:all` 执行。原始与 JADX 都通过并逐字输出 `7:-1`。Jarde 虽然从 `LambdaMetafactory` 写出了正确 arity 和捕获实参，却保留 `lambda$add$0`、`lambda$bound$1` 的显式 helper 调用和同名物理方法。javac 重编这些源码时，为 lambda 再生成同名方法，与显式方法发生 `compiler-synthesized` 符号冲突，因此 Jarde 的完整源码编译失败。这比单纯的 lambda 文案差异更严重。固定源码、`javap`、三方文本、字节哈希和编译诊断在 [outputs](outputs/)；连续两次重放的文件哈希相同。

当前 `crates/jarde-java/src/lambda.rs` 已从站点自身的 bootstrap、SAM/实现描述符与 handle 验证 lambda 形态和捕获顺序；`build.rs` 把捕获表达式传进 lambda AST，但 body 仍是对合成 helper 的调用。JADX 的 `CustomLambdaCall.buildMethodCall` 会识别同类 synthetic 实现方法，`InsnGen.makeInlinedLambdaMethod` 把其正文写进 lambda，并将 helper 标为不生成。这个处理顺序值得复用，不能仅凭 `lambda$` 名称删方法：先证明精确同类 bootstrap handle、唯一物理 helper、完整 body、所有调用点及捕获/SAM 形参映射，再一次性提交内联体和 helper 省略。捕获值是在函数对象创建时求值，内联不能把有副作用的捕获表达式移到每次调用时重新求值；首片可限定 `this` 和稳定参数值，保留现有站点来源与 helper 原始方法报告。预算停止、额外 helper 调用或不完整方法体应原子拒绝。

DT-25 的无捕获 lambda 首片和本项可共用内联/省略接缝，但本项需要额外的捕获求值时机证明；DT-27 的 `::` 方法引用保持独立。本次没有把 JADX 原测试的泛型 `List.removeIf`、字符串局部值与复杂短路体视为已经测过，它们应在无泛型干扰的捕获首片修复后逐个扩验。
