# EM-11 主线独立验收

Root 将实现提交 `690811d69563aeea07cc4d9d0d0eed8856a404d1` 拣入主线为 `46968a97` 后，重新构建 `jarde-cli`，在新的空目录独立执行 [replay.py](replay.py)。固定 JADX、原 class 与原/JADX/Jarde **完整 Java 8 源码**均编译成功、`java -Xverify:all` 运行成功；同类和继承 `ArrayList→List`、输入类层级、`null`/数组切片输出逐字一致。主线生成的 `OverloadCalls` 与 `HierarchyCalls` 源码和提交的 [`observed-after/source/jarde`](observed-after/source/jarde) 逐字节相同。缺父类与接口定义的选定输入仍在 BCI 7 拒绝。`jarde-java` 的 223 项库测试和 OpenSpec strict 验证通过。

该证明只涵盖已选来源下的单个普通引用实参及唯一目标签名，不把泛型、装箱、varargs 或缺失类层级推定为已解决。
