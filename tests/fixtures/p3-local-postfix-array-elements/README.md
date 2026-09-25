# Java 8 局部后置自增数组元素

`v8/ArrayPostfixElement.class` 与 `v8/ArrayPostfixElement-plus2.class` 来自冻结证据；SHA-256 分别为 `b3ec3d79d577f6483952fac584b96bdcdd69ca814615b338f4797112744352bb` 和 `7ad6f0722c3092f3f2e0d3094ad81d1f3acdcf22860a72566e07d984e4e4c8bf`。前者的 `iload_0; iinc 0,1; iastore` 可投影为 `arg0++`；后者的增量为 2，必须拒绝该投影。

不同槽控制先以 `javac --release 8 -g:none -Xlint:-options -d v8 DifferentSlotArrayElement.java DifferentSlotRunner.java` 编译，再运行 `python3 patch_different_slot.py`，把唯一 `iinc 0,1` 改成 `iinc 1,1`。`v8/DifferentSlotArrayElement.class` 的 SHA-256 为 `e281d54913a96926b7f46dea8728f5eaf8c1fef94824c13ee166d4259ef544cb`；原 class 经 `java -Xverify:all -cp v8 DifferentSlotRunner` 输出 `[1, 0, 0]`。若误写为 `a++`，第三元素会变成 2。

`tests/p3_local_postfix_array_elements.rs` 检查正例、两项拒绝边界、所有真实 BCI 来源、预算/取消及 Java 8 完整类的五行验证运行。
