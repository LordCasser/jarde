# 比较结果位于非零栈深度

`patch.py` 在冻结 long_eq 的操作数之前压入一个无后续读取的 Integer，比较及零分支读取的是其上方的值。同步调整 max_stack、Code 长度和 StackMapTable 后，原 patched class 通过 `java -Xverify:all`，结果为 7/9。

2026-09-23 实现后的 debug CLI 完整输出在 `jarde-after.java.txt`。其中 long_eq 原样恢复 `if (arg0 == arg2)`；`LowerStackRecovered.java` 只取这个未修改的方法，重编译并执行得到同样的 7/9。这是该方法的执行验收，不是整个 NumericComparisons 混合夹具的重编译声明。

准入应按真实 SSA 读取的值证明，不把零分支操作数错误限定为 Slot::Stack(0)。
