# Array field literal control baseline

This is the pure-literal control for JADX `arrays/TestArrayInitField.java`,
`TestArrayInitField.test()`: it preserves the exact static `byte[] a = { 10, 20, 30 }` and instance
`byte[] b = { 40, 50, 60 }` field-initializer shape, with only the public default constructor.
The Runner observes the static and instance contents and checks that two constructions receive
distinct instance arrays. It is paired with the ordered-marker `array-field-initializers` baseline.

Root 已实际执行 `prepare-baseline-luna-v1.py`，完整结果保存在 `baseline-root-v1/`。
每组 31 条真实命令、8 个完整类对照腿：Corretto 8/OpenJDK 23 原程序 2/2，fresh JADX 1.5.6 default/none 4/4，冻结 Jarde CLI 2/2 均完整重编并匹配原始三流。
两组由 [root 独立验收](../array-field-initializers-root-verification-v2.json) 核验，共 **2093 checks/0 errors**，包含闭合文件 hash、真实物理成员/flags 和逐方法 BCI。

Jarde 已将 static 数组 a 提升到字段声明；实例 b 保留构造器赋值。后者是呈现缺口，原样完整源码语义通过。该窄形状不代表 EM18 整单元完成。
