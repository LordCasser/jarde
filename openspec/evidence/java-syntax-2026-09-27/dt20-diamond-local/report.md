# DT-20：调试局部泛型与菱形构造的分界

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestConstructorGenerics`：有调试信息时精确断言 `Map<String, String> map = new HashMap<>();`，`noDebugInfo()` 分支则断言 raw `new HashMap().get("test")` 加 `String` 强转。这里保留这两个分支，不把无调试信息时不能还原的泛型局部计为同一个缺口。[replay.py](replay.py) 在 `-g`/`-g:none` 下分别编译原始 [Diamond.java](Diamond.java)，并将原 class、固定 JADX、Jarde 的完整类型源码与同一个 [Runner.java](Runner.java) 按 Java 8 重编，以 `-Xverify:all` 运行。两个模式的三方行为均为 `true`；两次独立重放的 7 个输出文件逐字相同。

`-g` class 的 `LocalVariableTable` 明确给出 `map` 在 slot 1、读取范围 `[8,20)`、擦除类型 `Ljava/util/Map;`；同一 `Code` 内的 `LocalVariableTypeTable` 在**同一 slot、名称和范围**给出 `Ljava/util/Map<Ljava/lang/String;Ljava/lang/String;>;`。关键 BCI 是 `new@0`、构造 `invokespecial@4`、唯一 `astore_1@7` 和范围内 `aload_1@8`：初始化 store 不在局部变量表范围内，恰好以一条指令结束于范围起点。JADX 保留局部泛型并写菱形构造；Jarde 修复后输出 `java.util.Map<java.lang.String, java.lang.String> map = new java.util.HashMap<>();`，并保留 `get` 结果强转。`-g:none` class 没有 LVTT；Jarde 仍输出 raw `HashMap` 与显式 cast。完整原始/JADX/Jarde 类型源码均以 Java 8 重编并经 `-Xverify:all` 运行得到 `true`。

Jarde reader 的同轮 `MethodCodeFacts` 原先只读取 `Code` 内 `LocalVariableTable`，跳过了 `LocalVariableTypeTable`。已有 reader 方法 `Signature` parser、局部变量身份/复用和预构建声明规划提供了接缝；**LVTT 是此输入中唯一携带两个 `String` 类型实参的事实**，仅靠 SSA 和擦除字节码不可能精确还原。实现为同次 Code 读取增加 raw LVTT 事实，再通过现有 `DebugLocal` 与 `LocalVariable` 身份，在声明规划前一次证明 slot、范围、名称、LVT 擦除、SSA 写入/使用和 Java 8 构造目标。语义 `Type::Reference` 保持擦除类型；源码拼写与精确分配 BCI 只在同一个已证局部投影中发布。第二写入、同槽寿命复用和不同泛型实参控制均拒绝投影，并在 Java 8 verifier 下通过；`-g:none` 保留 raw/cast。

此证据只约束固定 `Map<String,String>` 局部接 `new HashMap<>()`，不把其它泛型集合、跨分支局部、继承关系或整个 DT-20 标为追平。JADX 测试的 `noDebugInfo` 输出是文本断言而非语义损失测试，故保留独立控制。

修后 [replay.py](replay.py) 还以 `-g` 编译了第二次写入、同 slot 两段不同 `map` 寿命和 `Map<Integer,Integer>` 三个合法对照；它们都拒绝 `String` 泛型投影和构造菱形，原始与 Jarde 回退源码均按 Java 8 verifier 运行。错 LVTT 范围、畸形签名和重复 LVTT 会拒绝投影；当前 JVM 对这三种变异输入都抛出 `ClassFormatError`，因此只验证 Jarde 对它们的 raw 回退和回退源码 verifier。固定正例、无 debug 对照和负例结果记录在 [results.json](outputs/results.json)。
