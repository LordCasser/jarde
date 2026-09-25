# Root 独立验收（2026-09-24）

字段 `Signature` 恢复的已验收边界是顶层普通类、已发布类变量作用域、完整 descriptor 擦除一致、类型可拼写、无 type-use 注解路径，且常量池不存在指向本类同名同 descriptor 字段的 `Fieldref`。reader 沿已有 `FieldSignature` 树和第一边界擦除证明；类源码从结构化类型重新拼字段声明，成功/拒绝都只改变该字段的候选和标记，`FieldItem` 的物理身份未改。常量池缺席门保证本类字节码没有对该字段的指令访问；它也会保守拒绝未使用的同名 Fieldref，后续可用同轮 AST/SSA 扩展，当前没有新增泛型 pass。

Root 用独立 `/tmp/jarde-field-root-accept-target` 重建并运行：`field_generic_projection` 3/3、`class_source` 47/47、`generic_method_budget` 4/4、`generic_method_projection` 9/9、`ordinary_generic_projection` 14/14、reader 的 Signature 过滤结果 15/15（其中 14 项为 Signature 单测）、query lib 7/7。Root 在审读时清理了字段声明拼写重复、无效的 annotation stop 二次判断及报告注释，再重跑字段 3/3。`cargo fmt --all -- --check`、`git diff --check`、OpenSpec strict 均通过。

独立命令行重放 [字段证据](../../evidence/java-syntax-2026-09-24/field-generic-signatures/analysis.md)的 `StandaloneFieldBoundary<T>`：原 class、JADX、Jarde 三份 Java 源码在 `-g`、`-g:none` 下均通过 `javac --release 8`；同一个强类型调用方及字段泛型反射 runner 在 `java -Xverify:all` 下的 stdout 与原 class 逐字相同。Jarde 两份完整类源码 SHA-256 均为 `80b4e4025404480f04879c31e86e19de17b506f1fea1fcf24831afd131c76e13`，字段保留 `List<String>`、`List<? extends Number>`、`T`、`T[]`、`List<? super T>`；物理常量值仍可见。

Root 再独立把 `FieldSignatureConflict.items` 的字段 `Signature` 从 `List<Object>` 改为 `List<String>`，不改 descriptor 或 Code。原变体在 `-Xverify:all` 下仍输出 `value=42,java.lang.Integer`；JADX 输出 `List<String>` 和 `items.add((String) 42)`，Java 8 编译报 int 不能转 String。Jarde 明确报告同类 Fieldref 拒绝，仍输出 raw `List`，完整类及同一 runner 编译、运行成功，值与原变体一致。擦除不符、未绑定变量和 type-use 注解的字段级回退由定向测试与可重放 mutator 覆盖。

严格 `cargo clippy -p jarde --test field_generic_projection -- -D warnings` 首先遇到其它在途模块的 `useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref` 和 `collapsible_if`，均非本变更引入；只屏蔽这五类既有 lint 后同一目标通过。未通过全仓门禁、完整依赖闭包、含本类字段访问的兼容正文或泛型 `<clinit>` 不在本次成功声明范围内。[`throws E` 三方证据](../../evidence/java-syntax-2026-09-24/generic-throws-signatures/analysis.md)已单独冻结，未混入本变更。
