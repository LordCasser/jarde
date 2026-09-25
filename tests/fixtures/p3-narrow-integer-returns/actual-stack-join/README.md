# Java 8 实际操作数栈 join fixture

`ActualStackJoin.java` 使用 Java 8 语法编译。受控 stack rewrite 删除两臂的 `istore_1` 与
join 处的 `iload_1`，保留两臂常量在操作数栈上，并把 class major 改为 49、移除
`StackMapTable`。随后三份变体各自只将 `runByte`、`runChar` 或 `runShort` 的返回描述符改为
B、C 或 S。Code 字节保持不变；永久 class 各为 379 字节。

重建时从仓库根目录运行：

```sh
python3 tests/fixtures/p3-narrow-integer-returns/regenerate_task12_fixtures.py /tmp/task12-fixtures
```

stack rewrite 与返回描述符补丁分别复用
`openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-switch-returns/actual-stack-join/patch_stack_join.py`
和 `patch_descriptors.py`。`ActualStackJoinRunner.java` 在 `java -Xverify:all` 下检查每个
返回值；B/C/S class 的 runner 输出分别保存在各自的 `expected.txt`。
