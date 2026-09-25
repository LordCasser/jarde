# 类级类型变量作用域证据

## 范围与重放

本记录冻结 Java 8 类级泛型变量的正例，以及作用域/擦除边界。没有改生产代码或任务勾选。运行 `sh openspec/changes/recover-class-type-variable-signatures/evidence/replay.sh` 可从源码重建 fixture，分别用 `javac --release 8 -g` 和 `-g:none` 编译，执行 `javap -v`、JADX、当前 Jarde `class-source`，重编完整调用方并用 `java -Xverify:all` 运行和检查反射。脚本中的 Signature 变造器只替换 class/method `Signature` 常量，不改变 descriptor、Code 或物理父类/接口。Cargo 产物放在 `mktemp` 私有目录，退出时删除。

本次环境：OpenJDK/Javac 23.0.1（fixture 目标为 Java 8）、JADX 1.5.6、Cargo 1.98.1。Jarde 的 `cargo run` 在脚本私有 target 下完成构建。最后一次重放退出码为 0；独立临时 Cargo target 与中间 class/source 已由 trap 清除。

实现前的 Jarde 基线已由[普通参数化签名边界](../../../evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures/boundaries.md)冻结：类头无 `<U>`，方法以 `jvm_signature_scope_unproved` 回退 `Object`，泛型调用方无法编译。本脚本读取的是当前共享工作树，因而实测为恢复后的成功文本；不能把旧失败冒充为本轮输出。

## 正例

`ClassVariableBoundary<U>` 的 class Signature 是 `<U:Ljava/lang/Object;>Ljava/lang/Object;`，`identity` descriptor 为 `(Ljava/lang/Object;)Ljava/lang/Object;`，方法 Signature 为 `(TU;)TU;`。`-g` class SHA-256：`5fb125714b5079da1bb6fd8088ea0ebdf32e36776fc0d495633308296a4ef4c3`；`-g:none`：`b3f3d10f205b9b6b135fb83c6cca10bc2dd5f722a97e93e7bc72f97d36d93e`。

另两项正例覆盖边界。`BoundedClassVariableBoundary<U extends Number & Comparable<U>>` 的 Signature 是 `<U:Ljava/lang/Number;:Ljava/lang/Comparable<TU;>;>Ljava/lang/Object;`，方法 descriptor 擦除到 Number。`ClassVariableContract<U extends CharSequence>` 的 Signature 是 `<U::Ljava/lang/CharSequence;>Ljava/lang/Object;`，`identity` 是带方法 Signature `(TU;)TU;` 的抽象接口方法，无 Code 属性。它有一个 `StringContract` 实现。边界类 `-g`/`-g:none` SHA 分别为：

| class | `-g` SHA-256 | `-g:none` SHA-256 |
|---|---|---|
| `BoundedClassVariableBoundary` | `5e924d97c4c0b0622503d2180ceb02df71198d19772b0a2f3f683cb3edd90a7e` | `81dfbad5abfa5931e03d6480b071f3b53aab87c1cb172a6d00def3eb13c39238` |
| `ClassVariableContract` | `957d46e285e8b6a0520e8131c99898865276335891fc2712e9f2055a6c2d4f59` | `0bb26d288a18645075e7b5b45714857b32c5dc7acf4dbf2d2513cad3b4c9e82e` |

原 class、JADX 1.5.6 与当前 Jarde 均保留 `U`；有界变量和接口上界也保留。Jarde 类头只在证明 Signature 擦除与物理父类/接口相符后投影。它为无 Code 的接口方法输出 `U identity(U)` 并明确标为无方法体。原/JADX/Jarde 三方都对 `-g`、`-g:none` 输入完成 Java 8 泛型调用方编译。调用方以 `-Xverify:all` 执行，得到 `class-variable`、`19`、`interface-variable`；三方反射均看到 class 类型变量 `[U]`，方法参数和返回泛型类型均为 `U`。运行输出中所有值、参数/返回反射文本逐项相同；正例运行只使用本轮重新编译目录，没有预编译 runner 后备。

## 拒绝边界

负例原 class 均由 `-g`/`-g:none` 编译；合法原件通过 `-Xverify:all`。变造只改 Signature，以下三个变体也均通过 JVM verifier 并能执行，因此不能依赖 verifier 拒绝它们：

| 变体 | 物理信息 | 被改 Signature | 反射事实 / 当前 Jarde 处理 |
|---|---|---|---|
| `ParentMismatch` | 父类 `Object` | class 父类改为 `Thread` | 反射同时报告物理 `Object`、泛型 `Thread`；Jarde 因 `jvm_signature_erasure_mismatch` 保留物理类头，不发布 U。 |
| `InterfaceMismatch` | 接口 `Runnable` | 接口改为 `Callable` | 反射分别报告 `Runnable`、`Callable`；Jarde 报 `jvm_signature_erasure_mismatch`（interface 0），保留物理类头，不发布 U。 |
| `UnboundClassVariable` | class 声明 U，方法擦除为 Object | `identity` 改为 `(TV;)TV;`，V 未声明 | verifier 与方法调用通过，反射访问泛型参数时抛 `NullPointerException`。Jarde 保留已证 class U，但拒绝此方法 Signature（`jvm_signature_scope_unproved`），输出 Object 参数/返回。 |

Signature-only 变造 class 的 SHA-256：ParentMismatch `0a498cb38c9d36520f83bb7e6a74b38c3c24a154693778140bf13b528c977e18`；InterfaceMismatch `ca7e2f09ec3cf82229952a67e57e6227288c3539e6422fa26c54309f14a4b2f9`；UnboundClassVariable `bd1aef17ab6eb6946b9f9eadf9eb565b162ff8d6406bfec5efc95268201229aa`。JADX 可输出前两项中 Signature 声称的 Thread/Callable，也会把未绑定的 V 写入 Unbound 方法头；这些声明不能作为安全投影依据。

合法作用域对照也通过 verifier：方法级 `<T>` 与类级 `<T>` 同名时，JADX 保留两层 T；当前 Jarde 报重复作用域的 `jvm_signature_scope_unproved`，拒绝方法 Signature 并按 descriptor 输出 Object。成员内类的方法引用外层 T 时，JADX 输出 `T identity(T)`，但扁平单类输入没有外层类型变量声明；当前 Jarde 因可用 Signature scope 中没有 T 而拒绝该方法 Signature。它的单类输出编译失败于 JDK 23 对 `this.this$0` 构造器赋值的预览语法诊断，因此这里只记录缺失外层作用域，不将整类编译失败归因于泛型头。内类场景是单类输入的 scope 边界。

父类或接口的 Signature 若带泛型实参，例如 `Comparable<U>`，当前 class-source 还会拒绝类级头；仅证明 `<U>` 而不投影参数化继承关系不足以保证继承成员可重编。该拒绝属于独立边界，不由本记录的普通接口正例覆盖。

## 重放产物身份

主 fixture 源文件哈希：`ClassVariableBoundary.java` `8e285466a46828b8b843528f60b28a999827cecb73ed461427a61b3a2d4cba9a`；`BoundedClassVariableBoundary.java` `06f963c66c51a28cab1d5852d61dc6e10eb8249b246c5fe860c037051c6bb987`；`ClassVariableContract.java` `5d8975190ac625ae0c66ca78ff148cc4e9898c4298606c8adf88fdf90ed5dee3`。`patch_signature.py` SHA-256 为 `c3eeb7abba07ca09ef30501e96e899f9884e8ba23094e6b5f6c53b8bf82f5b04`；最终 `replay.sh` SHA-256 为 `40fd5d2637b2c35a1803b88786af84086f15aae7d952303fff4b5d3aa8b94d99`。脚本重放时还会打印所有 fixture、主要 Jarde/JADX 输出和变造 class 的 SHA-256。
