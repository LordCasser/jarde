# Root 独立验收（2026-09-24）

`recover-proved-generic-throws` 的首片已通过：Java 8 顶层类已发布的 `E extends Exception`，其无正文方法 `Signature` 为 `()V^TE;` 且物理 `Exceptions` 为 `Exception` 时，Jarde 完整类写出 `invoke() throws E`。`-g`/`-g:none` 由[正例重放](../../evidence/java-syntax-2026-09-24/generic-throws-signatures/replay.sh)独立再跑均退出 0：原/Jarde 类与同一强类型调用方 `javac --release 8` 成功，`java -Xverify:all` 都输出 `invoked`、`throws=E`；JADX 1.5.6 仍写 `throws Exception`，调用方编译退出 1。Jarde 两份源码 SHA-256 均为 `6708a11644032c0a2574ed9832b529ea5ae54ae3076dd336968f2afa563414f4`。

[负例重放](../../evidence/java-syntax-2026-09-24/generic-throws-signatures/negative/replay-verifier-valid-erasure-mismatch.sh)也独立退出 0。仅把类 `Signature` 的 `E:Exception` 改为 `E:Throwable`，不改方法 descriptor 或物理 `Exceptions`；脚本比较了原/补丁 `javap` 的 `Exceptions` 行，JVM 在 `-Xverify:all` 下仍运行。reader 的异常位置擦除证明报 `jvm_signature_erasure_mismatch`，Jarde 保留 `invoke() throws java.lang.Exception` 并附局部拒绝。定向测试还验证未绑定变量、混合 `IOException, E` 的位置顺序、删除 Signature 异常后缀时物理两项异常仍保留，以及有正文方法继续拒绝。

代码审读：`project_method_signature` 先用同轮已发布类作用域和 reader 的完整方法擦除证明核对参数、返回及非空异常列表；`ordinary_parameterized_declaration` 只对无正文方法接受已发布类变量，并要求其已证第一界擦除精确为四个已知 `Throwable` 根之一。它从解析树按顺序构造整个异常列表；后缀缺席时使用物理 `Exceptions`。`project_generic` 在来源说明、声明和输出预算均准备完成后才替换成员文本；证明失败仅给该成员留下物理声明和拒绝。预算测试经 root 加强后要求正常的 bounded outcome，并核对任何已发布的部分正文都没有半个 `throws E`；预取消返回 `Incomplete`。

Root 验证通过：`cargo test -p jarde --locked` 指定的 `class_source` 47/47、`field_generic_projection` 3/3、`generic_method_projection` 9/9、`generic_throws_projection` 5/5、`ordinary_generic_projection` 14/14；`cargo test -p jarde-reader --locked --lib signature` 15/15；`cargo test -p jarde-query --locked --lib` 7/7；`cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-generic-throws --strict` 均成功。定向 `cargo clippy -p jarde --test generic_throws_projection --locked -- -D warnings` 在沿用仓库既存五项 lint 允许项（`useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref`、`collapsible_if`）后成功；未据此声称整仓无警告。

覆盖边界：自定义异常界缺少受控继承证明时保持物理声明；有正文方法的异常控制/调用绑定仍保守拒绝；方法自有 `<X> throws X` 与无正文参数/返回泛型按[下一独立 OpenSpec](../recover-no-body-generic-method-signatures/tasks.md)处理。本变更未新增 parser、pass、crate 或外部依赖。
