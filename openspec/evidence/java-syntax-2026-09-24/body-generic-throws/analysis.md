# 有正文 `throws E`：Jarde 超越 JADX 的最小空正文切片

Java 8 顶层类 [`BodyThrows`](fixture/BodyThrows.java) 声明 `E extends Exception`，其有正文方法为 `void run() throws E { }`。`-g:none` 的 `javap -v` 证明：`run` 物理 descriptor 为 `()V`、`Exceptions` 为 `java.lang.Exception`、方法 `Signature` 为 `()V^TE;`；`javap -c` 的完整正文仅有 BCI 0 的 `return`。原 class 的[强类型调用方](fixture/BodyThrowsCaller.java)在 `BodyThrows<RuntimeException>` 上无检查异常地调用 `run()`，通过 `javac --release 8`，`java -Xverify:all` 输出 `throws=E`。

本地 JADX 1.5.6 将同一方法写为 `void run() throws Exception`；输出类可 Java 8 重编，但强类型调用方在 `value.run()` 报未捕获的 `Exception`、`javac` 退出 1。源码原因可直接定位到 `SignatureProcessor.parseMethodSignature`：它依次消费方法类型参数、参数和返回后，没有消费 `Signature` 的泛型异常后缀；`MethodThrowsVisitor` 扫描正文并把物理 `ExceptionsAttr` 并入 `MethodThrowsAttr`，`MethodNode.getThrows()` 再取该列表或物理异常。这条路径从未恢复 `E`，而 `MethodThrowsAttr` 的 `Set<String>` 也不是方法 Signature 的逐位置来源。实现前 Jarde 对有正文泛型异常亦保守保留物理类型；[`generic_throws_projection`](../../../../tests/generic_throws_projection.rs) 的相邻非空正文测试仍断言 `body() throws java.lang.Exception`，不把 `throws E` 盲目写进未知正文。

此场景不同于已完成的无正文 `throws E`：正文存在，虽然样本仅有一条 `return`，声明投影仍须有同轮完整正文证明，确认没有异常处理器、参数/局部效果或改变编译绑定的同类调用；reader 已有 Signature/擦除 proof，但它本身不证明源码正文兼容。泛型构造器的严格空正文 AST/SSA 候选可供比较其证明形状，但方法异常声明有自己的覆写与调用约束，需要单独界定。

[replay.py](replay.py) 已独立完成 `-g` 和 `-g:none` 两次三方重放，退出码 0；临时 class/source 和私有 Cargo target 自动清理。两种变体中，原 class 均以 Java 8 目标编译、JVM 严格验证、强类型调用方重编与运行成功，反射返回 `throws=E`。JADX 输出类可重编，但反射退化成 `throws=java.lang.Exception`，相同调用方均因必须捕获 `Exception` 而编译失败。实现前 Jarde 也这样退化，其局部拒绝理由是 `generic throws variables are supported only for no-body methods`。实现后 Jarde 在两种变体均写出 `public void run() throws E`；完整类与强类型调用方 Java 8 重编、`-Xverify:all` 运行成功，反射返回 `throws=E`。这项行为超过本地 JADX，而不依赖调试表。

| 变体 | 原 class SHA-256 | JADX 源 SHA-256 | Jarde 实现前 / 后源 SHA-256 |
| --- | --- | --- | --- |
| `-g` | `5623480bd30f5fa6609c4f1ec22fc6e1a1923f28afebce4bcd01decdfc6dedee` | `10f0ed2551267f65b27fa393f71274c0d7ab5e16b242b9ca853e520e2ca48e89` | `56a5cb4fc2cd926e3f112a5bfd0b89b0b0830b38e39abdbb9462c4ac934cf4a6` / `59ce417557143ccd5b852d6e599239b67b9622c40387f0771781b0373a6c0a9b` |
| `-g:none` | `0b1ef03f017ba81e0cf3cc14ac0f241a946b5ab63ef87ab22869a493d2e9d65e` | `10f0ed2551267f65b27fa393f71274c0d7ab5e16b242b9ca853e520e2ca48e89` | `56a5cb4fc2cd926e3f112a5bfd0b89b0b0830b38e39abdbb9462c4ac934cf4a6` / `59ce417557143ccd5b852d6e599239b67b9622c40387f0771781b0373a6c0a9b` |

环境：OpenJDK 23.0.1（目标 Java 8）、JADX 1.5.6、Cargo 1.98.1。源码样本的 SHA-256 均由重放脚本输出；该正例只证明严格空正文值得实现，不能自行推及非空正文或 verifier-valid 伪造 Signature。[独立 OpenSpec](../../../changes/recover-proved-body-generic-throws/design.md)因此复用 reader 的异常位置擦除证明和现有声明投影，增加最小同轮空正文证明；先保守限制顶层类、无覆写契约与无同类方法调用，不引入通用泛型异常推断。

[负例材料](analysis-negative.md)随后补齐了两份严格 JVM 可验证的未绑定/擦除矛盾 Signature，以及真实 `throw`、异常处理器、本类 Methodref 与继承调用。前三类正文在当前 Jarde 均保留物理 `throws Exception` 且完整类可 Java 8 重编；继承例在更早的父类 generic scope 门拒绝，不能误算为正文门的验证。相应 [negative/replay.py](negative/replay.py) 使用私有临时编译目录并逐例断言结果。`Object.finalize()` 同名覆写门另由 `generic_body_throws_projection` 定向 Rust 测试覆盖。
