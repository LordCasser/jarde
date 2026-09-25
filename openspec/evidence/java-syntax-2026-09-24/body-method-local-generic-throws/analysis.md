# 有正文方法自有 `<X> throws X`：空正文首片

Java 8 顶层类 [`MethodBodyThrows`](fixture/MethodBodyThrows.java)声明空正文 `public <X extends Exception> void run() throws X { }`。方法物理 descriptor 为 `()V`、`Exceptions` 为 `Exception`，方法 `Signature` 为 `<X:Ljava/lang/Exception;>()V^TX;`。原 class 的 `-g`/`-g:none` 均通过 `javac --release 8` 和 `java -Xverify:all`；不声明检查异常的[强类型调用方](fixture/MethodBodyThrowsCaller.java)以显式 `<RuntimeException>` 调用并打印 `called`。原 class 的方法反射有 1 个类型参数，泛型异常为 `X`。

[replay.py](replay.py) 实现后两次三方重放退出 0。JADX 1.5.6 写出 `<X extends Exception>`，但将 `throws X` 退化为 `throws Exception`；完整类可 Java 8 重编，方法类型参数反射仍为 1，但异常反射变成 `Exception`，强类型调用方因未捕获 `Exception` 而编译失败。Jarde 写出 `<X extends java.lang.Exception> ... throws X`；完整类和强类型调用方均可 Java 8 重编，`-Xverify:all` 运行打印 `called`，反射保留 1 个方法类型参数及异常变量 `X`。两种调试变体结果相同。

| 变体 | 原 class SHA-256 | JADX 源 SHA-256 | Jarde 源 SHA-256 |
| --- | --- | --- | --- |
| `-g` | `e39cdbd98d0bec7846ae05caeaed2fb3e1221ed2de5ec0adf83aa696fd8ad034` | `22a70ab7cdedd37e9a5bc6cb0572a696d213514c4a3385a21520aa7df49c2d66` | `03a56795e61d917ebc52714bddb3b225bbb41c7f2c459ecaf9269b41dfe337e3` |
| `-g:none` | `35e392e69ff715a66977b7f47a70f9cc542e5d454f29785df1ecd96f58ccfa39` | `22a70ab7cdedd37e9a5bc6cb0572a696d213514c4a3385a21520aa7df49c2d66` | `03a56795e61d917ebc52714bddb3b225bbb41c7f2c459ecaf9269b41dfe337e3` |

JADX 缺口与类级 `E` 共用 `SignatureProcessor.parseMethodSignature` 未消费 `^TX;` 的原因；这里更需区别**方法作用域**而非已发布类作用域。Jarde 复用了 reader 对局部 `<X>` 的解析和逐位置异常擦除证明，并把无正文泛型方法的结构化方法头拼写抽成共享步骤；有正文分支只在同轮 `EmptyVoid` 候选、单一 Throwable 根界、`throws X`、确无类 `Signature` 的 Object 直接子类和无本类调用风险都通过后一次发布完整声明。验证覆盖无正文方法与类级 `throws E` 回归；[独立负例](analysis-negative.md)重放 verifier-valid 伪签名、非空正文与本类调用，定向 Rust 回归另验证 handler 和 Object 同名方法的物理回退。

环境：OpenJDK 23.0.1（目标 Java 8）、JADX 1.5.6。脚本使用私有 Cargo target 和临时 class/source，运行结束自动清理；三份 fixture 的 SHA-256 由脚本逐次输出。
