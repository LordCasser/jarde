# 静态直接返回方法的局部泛型异常

`fixture/probe/StaticThrows.java` 是 Java 8 合法源码：`<T, X extends Exception> T echo(T value) throws X` 的正文只返回参数。`Caller` 显式指定 `String, RuntimeException`，无需捕获 checked `Exception`；`Reflect` 核对两个方法类型参数、`T` 返回和 `X` 异常。`replay.py --mode baseline` 在 `-g`/`-g:none` 下逐一编译、`-Xverify:all` 运行、反编译、完整类重编、反射和强类型调用重编，已退出 0。

| 来源 | `-g` / `-g:none` 共同结果 |
| --- | --- |
| 原 class | `Caller` 输出 `ok`；反射 `params=2, return=T, throws=X`；方法 `Signature` 是 `<T:Ljava/lang/Object;X:Ljava/lang/Exception;>(TT;)TT;^TX;`，物理 `Exceptions` 是 `java.lang.Exception`。class SHA256 分别为 `bb7f91c90811399d52954d9e9b2d923d3f1c38581bc9a2bce78f728903089757` 和 `cba80ff646dc5f9e301b48bb99a26a630c7ea96639316c58130e6fcc316bef5a`。 |
| JADX 1.5.6 | 保留 `<T, X extends Exception> T echo(T ...)`，但输出 `throws Exception`；类能重编，反射异常变成 `java.lang.Exception`，原强类型调用方重编失败。 |
| Jarde 基线 | 因 `generic_source_shape_unproved` 整体退回 `Object echo(Object) throws Exception`；类能重编，但返回/异常反射均丢失，原强类型调用方重编失败。 |

基线有调试表时 JADX/Jarde 源 SHA256 为 `324a6411ccdfeb4cd752022d4741d4ede91dc22d56ab50d7e50db01a7e7887a4` / `fdb8a94dd468374717bf08d98b8c8ceeb5b407a29d91089da0facc957778d79a`；无调试表时为 `b8bb57159433d6e47116c552e46bb064778a76e2637a7b9a92300bc588050c91` / `5725ae01011496c75a4739b873cae03163b71022708d1d0c30653db49eeac62c`。源码 fixture 的 SHA 和当前工具版本由脚本输出。`--mode recovered` 验收 Jarde，保留 JADX 的失败对照。

实现后 root 独立运行 `--mode recovered` 退出 0：Jarde 在 `-g` 与 `-g:none` 下均输出 `<T extends java.lang.Object, X extends java.lang.Exception> T echo(T ...) throws X`，完整类重编、反射 `params=2, return=T, throws=X`、原强类型调用方重编并运行 `ok`；JADX 两种变体的调用方仍失败。Jarde 两份源码 SHA256 分别为 `39a4bf6e4e925cee09ce9d4bd3978cb6961699dca0c936faf39f4e4af56901b6` 和 `17d77eaf4a557d772e4846e942d8444bd0134f45f71a14abd22362329bf9f30b`。负例的未绑定变量与异常擦除冲突由 reader 分别以 `jvm_signature_scope_unproved`、`jvm_signature_erasure_mismatch` 先拒绝；详见 [负例](negative/analysis-negative.md)。

JADX 源码的 `SignatureProcessor.parseMethodSignature` 读取方法类型参数、参数和返回值并应用它们，但没有消费 `^TX;` 异常后缀；`MethodThrowsVisitor` 则从 `Exceptions` 属性及方法体收集物理异常，并以集合合并。这解释了 `throws Exception` 的观察，也解释了另一份 [异常顺序反例](../throws-order/analysis.md)；Jarde 不应复制这条物理异常覆盖算法。

Jarde 的 reader 已经完整解析方法 Signature、证明类型变量作用域、逐位置擦除与物理 `Exceptions` 顺序；class-source 已有同轮 AST/SSA 的静态直接参数返回候选，以及无正文/空正文的局部 `throws X` 拼写方法。当前仅 `generic_method_declaration` 拒绝 `SignatureType::TypeVariable` 异常。先在静态直接返回子集复用这些证据，要求 `X` 的唯一 class first bound 是已知 JDK Throwable 根、原始类为顶层非泛型 Object 直接子类且无接口/本类同名调用；将完整方法头一次发布。复杂正文、自定义异常界、继承/隐藏与跨方法调用另列，不为此新增 Signature parser 或通用类型推断机制。
