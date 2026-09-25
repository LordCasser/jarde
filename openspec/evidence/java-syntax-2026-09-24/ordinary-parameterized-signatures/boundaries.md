# 1.2 拒绝边界：Signature、正文、类型上下文与调用绑定

在仓库根运行 `openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/reproduce_boundaries.sh` 可重建本记录的样本、JADX/Jarde 输出和命令结果。脚本使用独立临时 `CARGO_TARGET_DIR`，退出时移除；class、`javap` 和反编译文本留在 `/tmp` 目录，不入库。环境为 OpenJDK/Javac 23.0.1、JADX 1.5.6、Cargo 1.98.1。

## verifier 接受但 Signature 与正文冲突

`RawBodyMismatch.java` 是可由 Java 8 编译的 raw 方法：descriptor 为 `(Ljava/util/List;)Ljava/util/List;`，正文调用 `List.add(Integer.valueOf(42))`，raw runner 输出 `[42]`。`add_method_signature.py` 只在该 classfile 新增 Utf8 常量与 method `Signature` 属性，将它声明成 `(List<String>)List<String>`；不改 Code、descriptor 或其余成员。`javap -v` 读回该 Signature，`java -Xverify:all` 执行原 raw runner仍输出 `[42]`，读取泛型反射得到 `List<String>`。

`GenericLieRunner.java` 分别对原始 raw class 与带 Signature 的 class 做 Java 8 编译：前者编译成功但有 unchecked conversion 警告，后者无该警告；两次运行都会在读取 `result.get(0)` 时抛 `ClassCastException: Integer cannot be cast to String`。因此 CCE 不是新增 Signature 后才出现，字节码 Code 没变；证据说明 JVM 不验证 Signature 与正文的 Java 源类型一致，且泛型调用方会将不一致转成运行时检查。

对带 Signature 的 class，JADX `1.5.6` 输出 `List<String> body(List<String>)` 和 `values.add(42)`；这份源码的 Java 8 编译失败，诊断是 `int cannot be converted to String`。Jarde 输出 descriptor 的 raw `List`，并拒绝理由 `ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under parameterized types`；该 raw 单类源码可编译。用它重编 `GenericLieRunner` 会恢复 unchecked warning，运行行为与 raw 原类相同。结论是 body compatibility gate 真正区分可写声明；验证器通过并不足以批准泛型头。

## 未呈现的 class-level 类型变量

`ClassVariableBoundary<U>` 的 javac class Signature 为 `<U:Object>Object`，方法 `identity(U):U` 的 Signature 为 `(TU;)TU;`，descriptor 是 `(Object)Object`。原 class 和泛型 runner 通过 `java -Xverify:all`，反射返回 `U`。JADX 保留类头 `<U>` 及方法 `U identity(U)`。Jarde 的 single-class class-source 类头未呈现 `<U>`，方法头回退 Object，并报告 `jvm_signature_scope_unproved: type variable U is not declared by this method`；它单类可编译，但把 `ClassVariableRunner` 编译到 Jarde 单类 classpath 会因 `ClassVariableBoundary` 不是泛型类而失败。这是 source context 不完整的拒绝，不是合法 classfile 的 verifier 问题。

## 内类路径和 TYPE_USE 注解

`AmbiguousInnerBoundary` 定义两个不同 owner 下同名 `Entry` 内类，并分别声明 `List<Left.Entry>` / `List<Right.Entry>` identity 方法。原 class 的 `javap -v` 显示参数化 Signature 中两个二进制路径 `AmbiguousInnerBoundary$Left$Entry` 与 `...$Right$Entry`，原 reflection 也分别报告这两个类型；`-Xverify:all` 通过。JADX 从包含外层及嵌套 class 的 jar 重建为 `List<Left.Entry>` / `List<Right.Entry>`。Jarde 当前 class-source 对两个方法均报告 `generic_source_shape_unproved: class name has no unambiguous Java source spelling`，并回退到 raw `List`，不选择任一 `Entry`。

`AnnotationPathBoundary` 的合法 Java 声明为 `List<@Mark String>`。`javap -v` 记录普通参数化 Signature 加 `RuntimeVisibleTypeAnnotations`，type_path 指向类型参数。原 class 由 `-Xverify:all` 运行，反射观察到 `@AnnotationPathBoundary.Mark`。JADX 的反编译 class tree Java 8 编译、运行正常，但反射得到 `List<String>` 且类型实参注解数组为空，说明此输出丢了 annotation metadata。Jarde 为此方法拒绝普通参数化 projection，报告 `ordinary_generic_source_unproved`，并逐处记录 `target=0x14` / `0x16`、`TypePathEntry { kind: 3, index: 0 }` 无法拼写；声明回退为 raw `List`。不要把 annotation 丢失当成成功投影。

## 正文消费者与同类调用重载

`Boundaries.bodyPositive(List<String>)` 与 `identity(List<String>)` 直接返回原参数，是可成功投影并能编译/运行的正文正例。`bodyOverload` 被 `overloadedCall` 调用，后者对返回元素调用 `choose(CharSequence)`，类中另有 `choose(Object)`；javac 的 `javap -c -v` 显示调用目标严格为 `Boundaries.choose:(Ljava/lang/CharSequence;)Ljava/lang/String;`。原类与完整 JADX 重编类均经 `java -Xverify:all` 输出 `overload=char-sequence`。JADX 明写 `(CharSequence) bodyOverload(values).get(0)`，保留该绑定。Jarde 对同类 Methodref 涉及的 `bodyOverload` 与 `overloadedCall` 拒绝泛型声明，reason 为 `generic_call_binding_unproved`，但恢复正文仍保留显式 CharSequence cast；这条证据是“保守拒绝、调用仍指向原 descriptor”的负例，不主张观测到 Jarde 改绑。

独立的 `OverloadBinding.java` 把同类内层依赖拿掉，避免整类无法编译掩盖调用绑定证据。脚本输出原/JADX/Jarde 都是 `char-sequence`。原类与 JADX/Jarde 单类源码、调用 runner 均以 `javac --release 8` 编译，并以 `java -Xverify:all` 输出 `char-sequence`。`javap -c -v` 可核实原调用 Methodref 指向 `OverloadBinding.choose:(Ljava/lang/CharSequence;)Ljava/lang/String;`；JADX 正文显式保留 `(CharSequence)` cast。Jarde 因 identity 被本类 Methodref 使用而以 `generic_call_binding_unproved` 拒绝其泛型头，恢复正文仍有显式 cast，重编调用结果一致。脚本同时验证混合 `Boundaries` 类因独立 `$` 内类拼写问题无法整类重编，所以此最小 fixture 才是原/JADX/Jarde `-Xverify:all` 的调用绑定证据，不把混合类编译失败算成 2.4 正例。

Jarde 完整 `Boundaries` 文本另因物理 descriptor 中的 `Boundaries$Outer$Inner` / `$Left$Item` / `$Right$Item` 被写成 Java 不存在的二进制路径而未通过 javac；这一失败与上述泛型 Methodref 拒绝分开记录。独立 `AmbiguousInnerBoundary` 则验证它会对无法无歧义拼写的泛型内类路径拒绝并回退 raw 类型。

## 递归类名与擦除证明的边界

`reproduce.sh` 将所有 method Signature 的 `java/util/List` 改成等长 `java/util/Tree`，保留 descriptor，随后执行 verifier、反射和两份 decompiler source 的 Java 8 编译。`java -Xverify:all` 接受 class；直接参数/返回 `numbers`、`arrays` 的 Jarde 候选因 `jvm_signature_erasure_mismatch` 拒绝，但 `nested(Map<String,List<Integer>>)` 的最外层擦除仍为 Map，当前 Jarde 把内部不存在的 Tree 写进泛型声明，导致完整类 `javac --release 8` 失败。专用 `TreeSignatureReflectionRunner` 反射读取该 nested 参数时得到 `Map<String,java.util.Tree<Integer>>`；另一个 reflection runner 先读取 numbers，所以在它处抛 `TypeNotPresentException`。JADX 也投影 Tree，bad source 同样编译失败。

这说明 erasure proof 是直接签名位置的门，不是对任意嵌套引用的 class/source resolution。此样本 verifier 有效，但泛型反射在不存在类处失败、decompiler 源码非法；Jarde 对直接擦除冲突的拒绝成立，递归嵌套类引用校验仍待解决。不要把这条结果概括成“Jarde 拒绝所有错误 Signature”或“全面优于 JADX”。

## 哈希（SHA-256）

完整源码、原 class、变造 class、JADX source 和 Jarde source 的摘要由脚本末尾 `shasum -a 256` 逐次生成并列出。当前 OpenJDK 23.0.1 / JADX 1.5.6 的摘要如下：

- Sources: `OverloadBinding.java` `67f3f4d006a15db3050dfbc4b67c549ef7d963e7bebc9aa86c37c4461e432cbd`; `OverloadBindingRunner.java` `7f3e229c61cfb2c9fbe70ac89736f507e8b0f925d739c639a267b476c73517b3`; `Boundaries.java` `dde40c055a79de92b2b1a2641becba2fd5586db6e134215b47ac6fb8862962aa`; `BoundaryReflection.java` `33871d347fabc6ae184bd200c3f42213a51a09ef16cc3a05da705057c6993c04`; `ClassVariableBoundary.java` `13450e8528e1ea6851ca37b7d5c3eea97ef6df288c6f532c14f2cbcae6bb807f`; `AnnotationPathBoundary.java` `d130c457aca4530bd10d7455a0994cb5c253df0203449097a57bafe8e8570002`; `AmbiguousInnerBoundary.java` `f75ca629baa4eb563ff83fee5bec25cfc82a02e6c0e910a89fbe012d0a389375`; `RawBodyMismatch.java` `e20b18f1935a0a9598d09da34158a30a1d2a234e4279dccb45f3cbf4cc9e0718`; `RawBodyOriginalRunner.java` `72076b6d0957cb0fbcda292e14e8affd4cee3d1198b64382ede1173b5c050a0b`; `GenericLieRunner.java` `a75fc11136efd77f809d9c00f2d77077bf7e0024f0d4d56007aa1851247f8529`; `GenericSignatureReflectionRunner.java` `3e9780f35875f2bc2149f303bcdceacd57c0526aeecc54fd57452a21ddebb7f0b9`; `ClassVariableRunner.java` `622bd43c07034c93189208270ed07418cbac2cb19b7ac2c061c001deedea6143`.
- Compiled classes: `OverloadBinding.class` `6de0c5d6cac210519eccdb9bacffa9cb4fe8883e3a6fcce93b8acc46f5c1a56c`; JADX `OverloadBinding.java` `bf5698c38d47e21f47dbaab50817bae922528d67db6a552309049efa5e5b71b8`; Jarde `OverloadBinding.java` `bdaf1cac3299bdfdbb385a77a3ae1c6de8c64a3585414d34eba48adfccd2f866`; `Boundaries.class` `af23cdb331b87f8d9510ff808abd9c553824d97c4eb9003edfdc7e0432d1626c`; `ClassVariableBoundary.class` `c7e5ac474e76260f34640d904477a3a4a1ae6e883428f6ee805965deec116c31`; `AnnotationPathBoundary.class` `974eddb73ba68a76a15f642865f0bf3ef8aedb8e4929c8208209279d859a811d`; `AmbiguousInnerBoundary.class` `216dcae377f61676d858610ca62154a3d67fb0fe064282ccbcdc255a67019e12`; patched body class `dd8e508888cc73043a771cb201fffc7cd16d1b011d7ed4f525dfb4ff7fa2aaf7`.
- Decompiler outputs: JADX `Boundaries.java` `3fa053d641625493e770689c68af63c974ac70ae1db221cf8b26be3d1e34a066`; JADX patched `RawBodyMismatch.java` `640d4a41e2d8ca8e0afc32f6ac8399121870197b2e01fba7aff1dd64841d7a76`; Jarde `Boundaries.java` `f4605a49862df2479d48186e086e5e680c6b3e9315c5e2c927d66929baff68a6`; Jarde `ClassVariableBoundary.java` `3e807ef8f658ede1bfae3bd4c2a4d9b8b74c4be7278a7f11b48568ab9c7323cb`; Jarde `AnnotationPathBoundary.java` `b87c0453a1d2fc11cd3cf5129fd4183e6a5f95c8b9abeed7e6fd4ad694db7af4`; Jarde `AmbiguousInnerBoundary.java` `703d36817b7a2d310c56cc7c12b24c81d6a2de243905bc181ddbf2095bb1bbb1`; Jarde `RawBodyMismatch.java` `011539cb9713ae38bae1d0007872ecc8dd5e71cbb050e45a5f2e587eec53b723`.
