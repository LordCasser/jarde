# Bound receiver 创建时机：root 独立反例

`generate.py` 只用标准库生成合法major52 class，工厂是标准LambdaMetafactory，捕获Thread，implementation为REF_invokeVirtual Thread.start()V，无显式nullcheck。NoCheck把函数作为Thread构造实参；NoStand直接返回函数，证明缺口早于本轮构造参数扩展。

OpenJDK23.0.1与Corretto1.8.0_432均以-Xverify:all运行外部Driver/StandDriver，原class输出：

```
creation=ok
invocation=NPE
```

构造候选e47e7940的NoCheck完整输出(exposed.text、recompiled/NoCheck.java)及主线生产行为ddfd05f8的NoStand完整输出(v8*/baseline-NoStand.text)都产生arg0::start；完整重编执行却输出 `creation=NPE`。Java引用的隐式check不能反证原工厂本来已有check。不能把“method reference保留检查”说成安全，因为这里新增了原bytecode没有的创建时失败。

正式修复`preserve-bound-reference-creation-timing`是构造片前置；没有验证前不合并。默认拒绝无同轮nonnull证据的直接绑定引用，不引入猜测的check或capture移动算法。JADX此非javac字节码变体尚未对照，不声称其覆盖或缺陷。未来接手可直接重放原class与冻结错误输出，无须Rust builder。

原始fixture只有NoCheck.class/NoStand.class，Driver及recompiled class为输出不入库。源语义相当于捕获参数后的 `() -> arg0.start()`，但冻结字节码直接用虚拟handle，不假称由javac直接生成。

`positive/` 保存由 Corretto 8 编译的 `KnownBound` 源码、原class和Driver。`entryThis()` 的 `this::value` 与 `constantString()` 的 `"value"::length` 是本片验证的安全正例。`constantClass()` 的 `KnownBound.class::getName` 在javac8 class中是 `ldc; dup; invokevirtual Object.getClass; pop; invokedynamic`；当前builder保守拒绝这条dup形状，不把这个待处理的duplicate表达式所有权债务并入本片。
