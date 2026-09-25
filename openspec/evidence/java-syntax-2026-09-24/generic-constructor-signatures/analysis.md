# 泛型构造器的三方对照（下一候选）

Java 8 顶层类 [`GenericConstructor`](fixture/GenericConstructor.java) 有空正文泛型构造器 `<T extends Number> GenericConstructor(T value)`，物理 descriptor 为 `(Ljava/lang/Number;)V`，方法 `Signature` 为 `<T:Ljava/lang/Number;>(TT;)V`。[正常调用方](fixture/GenericConstructorCaller.java)以显式 `<Integer>` 实参实例化，并通过 `Constructor.getTypeParameters()`、`getGenericParameterTypes()` 检查泛型来源。原 class 的 `-g`/`-g:none` 均以 `javac --release 8` 编译并在 `java -Xverify:all` 下输出 `types=T`、`parameter=T`。此外，[静态类型边界](fixture/GenericConstructorTypeCheck.java)显式指定 `<Integer>` 却传 `Double`；原 class 下的 `javac` 退出 1。

JADX 1.5.6 对两种变体均写出泛型构造器，类与正常调用方重编、反射和执行与原 class 一致；错误静态类型边界仍由 `javac` 拒绝。实现前 Jarde 把构造器声明写成 `GenericConstructor(Number value/arg1)`：类可重编，但 `getTypeParameters()` 数量从 1 降为 0、泛型参数从 `T` 变为 `Number`。正常调用方本身仍能编译，因为 Java 8 接受构造器调用处的显式类型实参语法，但运行到反射数组索引时失败；更有判别力的是错误静态类型边界在 Jarde 类下反而通过编译，说明泛型约束确实丢失。正文为空，无法把这一差异归因于复杂值流。

reader 已有方法 `Signature` 的完整语法树和局部变量第一界/descriptor 擦除证明；实现前源码层 `project_method_signature` 把 `<init>` 排除在普通投影之外，现有 `generic_method_declaration` 又只接受静态直接参数返回的有正文方法，因此未把构造器签名交给其特殊声明形式。本轮[独立 OpenSpec](../../../changes/recover-proved-generic-constructor-signatures/design.md)复用 reader proof 与构造器声明拼写，对空正文和本类 `<init>` 调用绑定设证明门，没有另造 Signature parser。不能只复制 JADX 的类型替换而忽略正文、重载与 `this(...)` 链。

JADX 的 `ConstructorVisitor` 在 SSA 和 move-inline 后把 `INVOKE` 变为构造器节点，并会省略任意无参 `super()`；这提供了“先确认调用接收者与构造器种类，再处理源级声明”的参考顺序。本轮 Jarde 只取这项分析思想，不复制省略策略：同轮 `InitRecord` 必须指向 `Object.<init>()V`，Code/SSA 还需精确证明 `aload_0; invokespecial; return` 及参数没有参与效果，源码保留可审计的 `super()`。JADX 的能否输出并不替代泛型声明的擦除与调用绑定证明。

[replay.py](replay.py) 完整重放退出码 0，私有 Cargo target/临时类自动清理。环境：OpenJDK 23.0.1（目标 Java 8）、JADX 1.5.6、Cargo 1.98.1。原 class SHA-256：`-g` 为 `21927dd7dc307a4bbf9802e3bff168d48c4de2a3173bee8db8fbe8d827b70647`，`-g:none` 为 `db168207554e00ef3e30798d61c251a8854cde1c5d1cebfbe203d121c0d63eb8`；JADX 源码分别为 `6e652bc5ba578570cfdc9b2a124962c1094d7e3ba49cf43152861c838280474c`、`4ebda17923db77dba7eb4dc7fd15be43a3eafae666a8073c7dc97188da099299`；实现前 Jarde 源码分别为 `6f1247eb8e013e1c04760e8502f9869d630bc9bc102d9487f1b34ad0841a6bc1`、`02e2c8c74af68fa518ab3d5a8014759c32dfe799406c1fd316ca66ff1ae5ef15`。

实现后 root 独立重放：Jarde `-g` 写出 `public <T extends java.lang.Number> GenericConstructor(T value)`，`-g:none` 写出相同类型约束、无调试名 `arg1`。两份完整类、正常调用方均 Java 8 重编并 `-Xverify:all` 运行；构造器反射分别返回类型参数 `T` 和泛型参数 `T`；错误 `<Integer>`/`Double` 调用 `javac` 退出 1，与原 class 和 JADX 一致。新 Jarde 源码 SHA-256：`-g` 为 `82f1b02a821f047bff791bc21b26999ada7848ae0a3d08547585b4891f6b9f4b`，`-g:none` 为 `b515bfd463e8d079c2875cb660a327f02f7ac31a8004d4c0259b4923499e710e`。本轮还按[负例重放](negative/replay.py)逐一确认五个拒绝形状保留物理声明并可重编。

## 构造器负例重放

从仓库根目录运行 `python3 openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/negative/replay.py` 可重放负例。脚本用 `javac --release 8 -g` 编译 [negative/fixture](negative/fixture)，只替换 `SignatureBoundary.<init>(Number)` 的 `Signature` 常量，保留 descriptor、Code 和 flags；每个原始类及两个变造类均由 `java -Xverify:all` 实例化。`javap -v` 断言两个变造 Signature 仍挂在 `(Ljava/lang/Number;)V` descriptor 上。Cargo target 和所有临时 class/source 都放在 `TemporaryDirectory` 下并在退出时清理。

| 负例 | 构造器 Signature / 正文 | 原 class verifier | Jarde class-source | Jarde 输出 `javac --release 8` |
| --- | --- | --- | --- | --- |
| 未绑定变量 | `<T:Ljava/lang/Number;>(TU;)V` | 通过 | 保留物理 `Number` 构造器，无泛型声明 | 通过 |
| 第一界擦除不符 | `<T:Ljava/lang/Integer;>(TT;)V`，物理参数仍为 `Number` | 通过 | 保留物理 `Number` 构造器，无泛型声明 | 通过 |
| 参数参加正文 | 正常 `<T:Ljava/lang/Number;>(TT;)V`，正文写入 `saved` 字段 | 通过 | 保留物理 `Number` 构造器，无泛型声明 | 通过 |
| `this(...)` 链 | 正常 Signature，泛型构造器委托本类无参构造器 | 通过 | 保留物理 `Number` 构造器，无泛型声明 | 通过 |
| 本类构造器调用绑定 | 正常 Signature，类静态工厂执行 `new <Integer> SameClassCall(value)` | 通过 | 保留物理 `Number` 构造器，无泛型声明 | 通过 |

脚本逐例断言 Jarde 命令成功、输出保留 `<init>(Ljava/lang/Number;)V` 物理身份和 `Number` 参数、没有泛型构造器投影；五个回退类源码均通过 Java 8 编译。本类调用绑定例的 `create(Integer)` 改为 `void` 后，classfile 仍含 `SameClassCall.<init>(Number)` 的自类 `Methodref`，但不再依赖当前有缺陷的非 void 正文返回恢复，因此得到完整闭环。五个例子的构造器签名均留在物理声明，未发生部分泛型投影。

本轮命令 `python3 openspec/evidence/java-syntax-2026-09-24/generic-constructor-signatures/negative/replay.py` 退出码 0；脚本断言 `javap -v` 中自类构造器 `Methodref` 存在、所有五个 Jarde 回退类通过 `javac --release 8`，并确认预期构造器非投影。TemporaryDirectory 清理了私有 Cargo target。
