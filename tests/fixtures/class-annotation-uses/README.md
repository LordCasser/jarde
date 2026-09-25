# 类级注解属性测试输入

`src/` 保存最小 Java 8 类声明输入：`VisibleTarget` 使用 RUNTIME-retention 的字符串值，`HiddenTarget` 使用 CLASS-retention 的整数值，`MixedTarget` 同时使用这两种属性，`EmptyTarget` 是无注解对照。`DuplicateTarget`/`Tag`/`Tags` 是 javac 合法的 repeatable 容器对照，`BoundaryRunner` 检查 CLASS-retention 在反射中仍不可见。`MemberPlacementTarget` 在字段、方法、参数和 type-use 放置 `MemberPlacement`，用来确认类声明恢复不会搬动这些位置的注解。

`v8/` 中 class 文件由以下命令生成，集成测试直接读取这些确定性 fixture：

```sh
javac --release 8 -g:none -Xlint:-options -d tests/fixtures/class-annotation-uses/v8 tests/fixtures/class-annotation-uses/src/*.java
```

重复条目、损坏值 tag 和不能拼写的类型描述符变体由 `openspec/evidence/java-syntax-2026-09-22/class-annotation-uses/boundaries/run_boundary_audit.py` 从 `HiddenTarget.class` 原字节受控生成，并记录其来源跨度及 SHA。
