# DT-20：调试局部泛型与菱形构造的分界

固定 JADX `2fb1b16386941660fda07e9017285aec40fcb37f` 的 `TestConstructorGenerics`：有调试信息时精确断言 `Map<String, String> map = new HashMap<>();`，`noDebugInfo()` 分支则断言 raw `new HashMap().get("test")` 加 `String` 强转。这里保留这两个分支，不把无调试信息时不能还原的泛型局部计为同一个缺口。[replay.py](replay.py) 在 `-g`/`-g:none` 下分别编译原始 [Diamond.java](Diamond.java)，并将原 class、固定 JADX、Jarde 的完整类型源码与同一个 [Runner.java](Runner.java) 按 Java 8 重编，以 `-Xverify:all` 运行。两个模式的三方行为均为 `true`；两次独立重放的 7 个输出文件逐字相同。

`-g` class 的 `LocalVariableTable` 明确给出 `map` 在 slot 1、BCI `[8,20)`、擦除类型 `Ljava/util/Map;`；同一 `Code` 内的 `LocalVariableTypeTable` 在**同一 slot、名称和范围**给出 `Ljava/util/Map<Ljava/lang/String;Ljava/lang/String;>;`。JADX 保留局部泛型并写菱形构造；Jarde 保留 `map` 名称，但声明为 raw `java.util.HashMap map = new java.util.HashMap()`，`get` 结果被强转为 `String`。因此完整源码虽可编译运行，DT-20 的有调试信息语法未恢复。`-g:none` class 没有 LVTT；JADX 和 Jarde 均采用 raw `HashMap` 与显式 cast，Jarde 多保留一个中间局部，但不应在本片猜泛型参数。

Jarde reader 的同轮 `MethodCodeFacts` 已读取 `Code` 内 `LocalVariableTable` 并传给 `RecoveryFacts::debug_locals`，但当前 `local_debug_table` 跳过 `LocalVariableTypeTable`。已有 reader 方法 `Signature` parser、局部变量身份/复用和预构建的声明类型规划；这些可作为接缝，但 **LVTT 是此输入中唯一携带两个 `String` 类型实参的事实**，仅靠 SSA 和 `new HashMap`/`get` 的擦除字节码不可能精确还原。因此这一切片确需为同次 Code 读取增加 LVTT 泛型类型事实，并以 slot、范围、名称、LVT 擦除、SSA 写入/使用和 Java 8 构造目标作一致性证明；不是新建通用类型推断器。错误、重复、缺失或冲突的 debug 元数据应沿当前 raw/cast 路径拒绝泛型投影。

此证据只约束固定 `Map<String,String>` 局部接 `new HashMap<>()`，不把其它泛型集合、跨分支局部、继承关系或整个 DT-20 标为追平。JADX 测试的 `noDebugInfo` 输出是文本断言而非语义损失测试，故保留独立控制。
