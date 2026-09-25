# 有正文 `throws E` 投影门的 negative 边界

本组反例限定下一步若支持类级类型变量 `throws E`，必须先证明**同轮正文**对源码声明兼容。它们不否定父目录正例中 `return` 空效果正文可作为候选，只证明不能仅凭类级 Signature 的变量边界证明就普遍删除物理 `Exceptions` 擦除类型。

## 可重放证据

从仓库根目录执行：

```sh
python3 openspec/evidence/java-syntax-2026-09-24/body-generic-throws/negative/replay.py
```

脚本以 `javac --release 8 -g:none` 编译 fixture；先用 `java -Xverify:all` 运行含有调用、handler 和真实 throw 的原 class，再分别用 Jarde 的 `class-source --policy single-class --release 8 --format text` 检查类。前三个直接类与 inherited case 的 Jarde 完整 class-source 都另以 Java 8 `javac` 重编译成功。脚本还按等长改写 classfile 的 `CONSTANT_Utf8` Signature 内容，构造 verifier-valid 的未绑定 throws 变量与异常擦除矛盾变体；两变体都再次经严格 JVM 验证，并送入 Jarde。断言要求方法声明不出现 `throws E`、仍呈现物理 `throws java.lang.Exception`，正常直接类还须出现 `run` 的局部泛型投影拒绝，不绑定具体解释文字。临时 class、源码输出以及私有 `CARGO_TARGET_DIR` 都由 Python `TemporaryDirectory` 清理。

## 边界事实

fixture 的三个目标都由完整 Java 8 源编译，没有手改正文：

| Case | 正文证据 | 限制了什么 |
| --- | --- | --- |
| `HasCall.run() throws E` | 调用同类 `callee() throws E`；`javap -v` 的方法 Signature 为 `()V^TE;`，运行体有 `invokevirtual` | 必须考虑本类 Methodref 及调用绑定，不能仅看直接异常 opcode |
| `InheritedCall.run() throws E` | 继承 `GenericParent<E>` 并调用其 `inherited() throws E` | 继承 throws 契约确实存在；但当前 single-class Jarde 更早因缺失父类 generic scope 拒绝，不能把它误记为 body 门已验证 |
| `HasHandler.run() throws E` | `try` 调用真实 `risky() throws IOException`，catch `IOException`；class 有 Exception table | 有处理器/捕获类型与局部变量时，不能按空正文放行 |
| `HasThrow.run(E) throws E` | 直接 `throw value`，方法 Signature `(TE;)V^TE;` | 真正的异常逃逸效果与擦除 `Exception` 不能忽略 |

Java 8 编译后 runner 在 `-Xverify:all` 下输出 `verified`。Jarde 对前三个方法都局部拒绝泛型异常投影，并维持物理 throws 类型；因此当前物理回退与完整类编译结果一致，也确定候选门须排除含 throw、异常处理器及本类调用的方法。继承样本同样通过 javac 与 JVM verification，Jarde 保留物理异常类型；它因缺少父类 generic scope 在更早阶段拒绝，没有覆盖到 body 专属准入逻辑。基于此样本，可确认继承 throws 契约是需检查的边界，但尚未证明现有 body 门本身如何处理已解析的继承关系。Object 实例方法同名覆写边界未单独构造。

另外两个输入说明 JVM verifier 不验证泛型 Signature 的变量作用域或擦除一致性：

| 变体 | 等长 Signature 变更 | JVM strict verify | Jarde 行为 |
| --- | --- | --- | --- |
| `erasure-conflict` | 类变量界从 `Exception` 改为 `Throwable`，方法 descriptor/物理 Exceptions 仍为 `Exception` | 通过 | 拒绝投影，保留 `throws java.lang.Exception` |
| `unbound-throws-variable` | 方法 throws 后缀 `^TE;` 改为未声明的 `^TX;` | 通过 | 拒绝投影，保留 `throws java.lang.Exception` |

所以“原 class 能经 JVM verification”不是 Signature 中 `E` 可安全输出为 Java throws 子句的充分证明。Erasure contradiction 的变体在 JVM 上可装载运行，不代表它有可被 `javac` 接受的 Java 源等价物；这里特意只把它用作 classfile reader 的反例。

## 冻结输入与校验值

在当前工作树，fixture 与 Java 8 class 的 SHA-256 为：

| 输入 | SHA-256 |
| --- | --- |
| [`BodyThrowsBoundaries.java`](negative/BodyThrowsBoundaries.java) | `7a2531f695190c7941bc61cf13b96bc804df42e249913ea8b20e790378b79039` |
| `HasCall.class` (`-g:none`) | `5609115817f393a670474d34e0cb83070fe5c13a64b39b81e11d3589d5f5064f` |
| `InheritedCall.class` (`-g:none`) | `2b649bde11b73e29248598985746f2f8f070bf8b303726ee4ad580ea9bae31c0` |
| `HasHandler.class` (`-g:none`) | `bbe20e301fd5acf1a9dfc18f3e16f4e193911dd8a973af3a910f6f6606a3ccae` |
| `HasThrow.class` (`-g:none`) | `133dc978b67302dab5eb134802118e4c23c8ac2f719c9fcc28da17c866c62587` |
| `erasure-conflict` mutated `HasThrow.class` | `9fe8a4d4d48abbdbe1f6923dd3b9b11b4223606d1c49e48ef015a82c77cc9fb4` |
| `unbound-throws-variable` mutated `HasThrow.class` | `d7f99b948c99b9773ce9e297076ef0aa77ed8d13834297531a08785557a212ee` |

重放环境：OpenJDK `23.0.1`（`--release 8`）、Cargo `1.98.1`。脚本每次打印当次 CLI digest、`javap` 声明行和复放结果；实现仍在编辑中，CLI digest 不作为冻结输入。构建输出及 Cargo target 随即清理。
