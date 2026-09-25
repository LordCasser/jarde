# 泛型成员构造的元数据负控

[`replay.py`](replay.py) 用父目录的 `Outer.java` / `UseObject.java` 重编 Java 8、`-g:none` 原始 class，然后保存原始目标 jar 与已编译 caller class。原始 caller 先经 `java -Xverify:all` 执行，trace 是 `minimal.Outer$Inner:1`、`null:1`。随后对 `Outer$Inner.class` 的单个 classfile 元数据做三种变异；每个变体仍以相同 caller 在 `-Xverify:all` 下运行，trace 与原始相同。

| 变体 | 变异 | JVM 验证/运行 | 证据 |
| --- | --- | --- | --- |
| `wrong-class-type-variable.jar` | 类 Signature 把声明变量 `V` 改为 `W`，字段、构造器及 getter 的 `TV;` / `()TV;` 保持不变 | 通过，trace 不变 | [`javap`](javap-wrong-class-type-variable.txt)、[`run`](run-wrong-class-type-variable.txt) |
| `wrong-constructor-signature-erasure.jar` | 构造器 Signature 从 `(TV;)V` 改为 `(Ljava/lang/Long;)V`，物理 descriptor 仍是 `(Lminimal/Outer;Ljava/lang/Object;)V` | 通过，trace 不变 | [`javap`](javap-wrong-constructor-signature-erasure.txt)、[`run`](run-wrong-constructor-signature-erasure.txt) |
| `wrong-innerclasses-owner.jar` | Inner 自己的 InnerClasses 行将 declaring outer 从 `minimal/Outer` 改为 `java/lang/Object` | 通过，trace 不变 | [`javap`](javap-wrong-innerclasses-owner.txt)、[`run`](run-wrong-innerclasses-owner.txt) |
| `missing-inner-target.jar` | 保留 Outer.class，省略 Outer$Inner.class | `-Xverify:all` 执行到构造调用时报 `NoClassDefFoundError` | [`run`](run-missing-inner-target.txt) |

三种变异都只更改属性中的声明信息；代码、调用描述符、字段布局和捕获 outer 初始化字节码保持不变。它们是 JVM 可加载/可验证的 classfile 变体，不是可由 Java 源码正常表达的声明。第一种 Signature 在 JVM 层面仍可解析为合法签名字符串，但变量引用在该类声明作用域内不成立；本测试未调用会解析泛型 Signature 的反射 API。因此只能据此说正常类加载、验证和执行不受影响，不能推断反射行为兼容。

可选的第二构造器重载变体由源码编译：在 `Inner(V)` 旁添加 `Inner(String)`，然后仅用原始 `UseObject.java` 对此变体 jar 执行 Java 8 caller-only 编译。编译成功，`javap-second-overload-caller.txt` 可检查实际 invokespecial 仍选择物理 `(Lminimal/Outer;Ljava/lang/Object;)V` 泛型构造器；运行 trace 仍为原始两行。这给重载解析约束提供了一个窄正例，不覆盖更复杂的交叉可转换参数或泛型构造器重载集合。

这里的对照目标是证明新源码投影必须检查哪些已冻结事实：目标类 Signature 的类型变量作用域、源级构造器 Signature 到物理 descriptor tail 的擦除关系、InnerClasses 的 declaring outer 一致性，以及目标 class 是否在选定 classpath 可用。缺目标 variant 的 `NoClassDefFoundError` 是依赖缺失的执行结果，不应作为可恢复构造的反例；三个属性变体则运行成功，但其声明证据不足/矛盾时也不能因此放宽源码投影。

环境版本见 [`tool-versions.txt`](tool-versions.txt)。所有 jar、变异 class、javap 文本、原始/变体 trace 与 fixture 输入都记录在 [`sha256.txt`](sha256.txt)。脚本用 `TemporaryDirectory` 清理中间编译目录，不运行 Cargo 或 Jarde。
