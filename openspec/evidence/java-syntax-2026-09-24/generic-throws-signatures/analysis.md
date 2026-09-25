# 泛型 `throws` 的三方对照（下一候选）

Java 8 顶层抽象类 `GenericThrowsBoundary<E extends Exception>` 的无正文 `invoke()` 声明 `throws E`。[固定 fixture](fixture/GenericThrowsBoundary.java)与[强类型调用方](fixture/GenericThrowsCaller.java)只需标准 Java classpath。调用方把实例静态看作 `GenericThrowsBoundary<RuntimeException>`，在未声明检查异常的 `narrowed` 方法中调用 `invoke()`；原源码 `javac --release 8` 成功，`java -Xverify:all` 输出 `invoked`、`throws=E`。`-g` 和 `-g:none` 原 class 的 SHA-256 分别为 `1a92a314944a0c6a960a4acd26c70dddaadb7a44d08cb6caf7ff221afc2d7a15` 与 `5c932669ca2619bb70d81bfc5e7adbf31aa00bd8b8d4c891e91ba28ba8ec2164`。

`javap -v` 显示 class `Signature` 声明 `E:Exception`，方法的物理 `Exceptions` 为 `java.lang.Exception`，方法 `Signature` 为 `()V^TE;`。这是泛型异常变量与第一边界擦除的正常分工，不能只根据 `Exceptions` 属性写出源码。

安装的 JADX 1.5.6 在 `-g`、`-g:none` 两份样本上均输出 `GenericThrowsBoundary<E extends Exception>`，但无正文方法是 `invoke() throws Exception`。该类单独通过 Java 8 重编，同一 `GenericThrowsCaller` 的 `narrowed` 则在 `value.invoke()` 处因未捕获的检查异常无法编译；重编的无泛型方法异常位置也会使反射从 `E` 变为 `java.lang.Exception`。本地 JADX `SignatureProcessor.parseMethodSignature` 处理类型参数、参数和返回类型，未消费泛型 `throws` 后缀；`MethodNode`/代码生成走物理 `MethodThrowsAttr`。因此它在此处不是正确性基线。两份 JADX 输出 SHA-256 均为 `9e858190ad5772ed6aaba7102da584ef08e7edae830a1fd299197b7ac9c7aeca`。

本次 Jarde 在两种调试变体的类源码中也写出类头 `<E extends java.lang.Exception>`，但 `invoke() throws java.lang.Exception`，并附 `ordinary_generic_source_unproved: generic throws variable has no preserved source position` 的局部拒绝。类单独可重编，调用方在同一行编译失败；两份 Jarde 源码 SHA-256 均为 `9602f283c06cf6178531060c8d380f422147bd2edec461e9a51e261fdb523033`。[replay.sh](replay.sh) 在字段实现验收后完整重放退出码 0，私有 Cargo target 与中间产物由脚本清理。

架构上 reader 的 `prove_method_signature_erasure_with_class_scope` 已能把 `throws E` 与物理 `Exceptions` 逐位置比对，类头的 `E` 也已证明并发布。当前缺口集中在 `ordinary_parameterized_declaration`：它显式拒绝 `SignatureType::TypeVariable` 异常，最后仍以物理异常列表拼接方法头。下一独立 OpenSpec 可以只扩展已证类变量、无正文方法的 `throws` 拼写；同时保留 `Exceptions` 逐位置擦除和 checked-exception 类型约束，不需要新的 parser 或全局求解机制。方法自身 `<X extends Exception> ... throws X` 与有正文异常边另作测试和后续扩展，不混入字段 Signature 改动。

环境：OpenJDK 23.0.1（编译目标 Java 8）、JADX 1.5.6、Cargo 1.98.1。fixture SHA-256：`GenericThrowsBoundary.java` 为 `c9f3ecdd17a87d17db7b7c57df0326bb3fda74292ba4343a8cf83f80cad38c6a`，`GenericThrowsCaller.java` 为 `1643e21dbfb6f89ae024a573081357e0e40b753058155ff305e2668c38c840db`。原/JADX/Jarde 两个调试变体的类单独重编和同一调用方均已由完整脚本核对。

负例 `negative/replay-verifier-valid-erasure-mismatch.sh` 只把 class `Signature` 的 `E` 上界从 `Exception` 改为等长的 `Throwable`；方法 descriptor 与物理 `Exceptions: java.lang.Exception` 保持原样，脚本逐项比较原/改 class 的 `Exceptions` 输出。改后 class 在同一调用方下通过 `java -Xverify:all`。Jarde reader 报 `jvm_signature_erasure_mismatch`（Signature 擦除与 `Exceptions` 不一致），源码局部回退为 `invoke() throws java.lang.Exception;`。该脚本退出码为 0。
