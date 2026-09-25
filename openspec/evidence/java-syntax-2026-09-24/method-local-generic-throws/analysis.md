# 方法自有泛型异常的三方对照（后续独立候选）

Java 8 顶层抽象类 `MethodThrowsBoundary` 的无正文方法声明 `public abstract <X extends Exception> void raise() throws X`。[fixture](fixture/MethodThrowsBoundary.java)与[强类型调用方](fixture/MethodThrowsCaller.java)只依赖标准 JDK。调用方以显式 `<RuntimeException>` 类型实参调用，并提供保持泛型方法签名的覆写类。原源码以 `javac --release 8` 的 `-g` 和 `-g:none` 均成功；`java -Xverify:all` 输出 `raised` 和 `throws=X`。`javap -v` 显示物理 `Exceptions: java.lang.Exception`，方法 `Signature: <X:Ljava/lang/Exception;>()V^TX;`。两份原类 SHA-256 分别为 `0805596ea53747e10010e7f72ddb17955159bc12eeb644c00da81fb89dc4d685` 和 `a406c2ce3c309b131d04d9af4918d9e1ef9efcdf4993cd1fd6a8e6dbe22cace7`。

JADX 1.5.6 两种变体均输出 `<X extends Exception> void raise() throws Exception`，源码 SHA-256 同为 `b839386929a62da115774591138b5539fa51b5cef4b1d2c0a757115bda62ff60`。单独类和只用反射的 runner 可重编，但反射变为 `throws=java.lang.Exception`；原调用方在显式 `<RuntimeException>` 调用处报未处理的 `Exception`，退出码 1。JADX 本地 `SignatureProcessor.parseMethodSignature` 只消费类型参数、参数和返回，`MethodNode.getThrows()` 从物理异常属性取类型；泛型异常尾部未进入方法声明。

当前 Jarde 两种变体都输出 `public abstract void raise() throws java.lang.Exception`，源码 SHA-256 同为 `974b8be950b74675059de4ff11b875d30c1dbf85ac3e8d707509f1a19ccf0a43`。单独类可重编，反射也变成 `throws=java.lang.Exception`；调用方在覆写处就因方法类型参数消失而产生三个编译错误。源码接缝 `project_method_signature` 对带方法自有类型参数的 `NoBody` 直接跳过，`generic_method_declaration` 只接受带已恢复正文的静态返回形状。因此此缺口同时涉及方法级 `<X>` 与 `throws X`，不能靠类级 `throws E` 改动顺手解决。

合理的后续切片是在已有 reader 的方法局部作用域与逐位置擦除证明之后，给无正文声明增加方法级类型参数的结构化拼写；同一个候选里写出参数、返回与异常变量，并要求异常变量第一界可证明属于异常家族。沿用既有无正文标志、注解门、同类调用门、来源和预算，不引入另一套 parser 或泛型 pass。有正文泛型方法的异常控制流与调用绑定另行证明。JADX 的类型参数投影有参考价值，但这里的异常输出不是可沿用的正确性基线。

[负例重放](negative/replay.py)保留物理 descriptor/`Exceptions`，只把方法 `Signature` 改成未绑定的 `^TY;` 或把 `X` 的界改为 `Throwable`、造成与物理 `Exception` 不同的擦除；两份 class 仍在 `java -Xverify:all` 下调用成功。另一个正常编译、也通过 verifier 的抽象类继承了外部 `GenericParent`，其无正文泛型方法需要父类覆写契约才能安全投影；首片按缺失继承证明拒绝。当前 Jarde 对三者都因方法自有变量早退而输出物理声明，尚无针对签名矛盾的局部拒绝；后续实现需用 reader proof 和类层次门区分这三种原因。

[replay.sh](replay.sh) 在本地完整重放退出码 0，并清理了私有 Cargo target 和全部临时产物。工具：OpenJDK 23.0.1（目标 Java 8）、JADX 1.5.6、Cargo 1.98.1。fixture SHA-256：边界类 `21d6ff7e12f89334b21c614fdf7103ced2859b40038e7efc7b7517f3c8713045`，调用方 `af5ae1137fb0fea02a870ebdd8f247c278b93d3a892627d9fa08a47405861248`，独立反射 runner `d4acf55f29aaa7819ddce15d9bd919a1307a28344b90274e17e4712655986a4f`。
