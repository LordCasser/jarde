## Why

Java 8 的非空数组初始化器会编译为 `newarray/anewarray; dup; index; value; *astore` 链。jarde 已能写出空数组分配，却不能把这条有界链恢复为一个可消费的数组值；四个真实方法因此只留下字节码引用，完整类缺少返回语句。原 class 与 JADX 的 14 项整类执行结果一致。

## What Changes

- 在现有数组创建值的构建阶段识别**单一基本块、单一数组身份、常量长度和按序且仅一次写入每个元素**的初始化链；把每个元素表达式附到同一个数组创建表达式，写出 `new T[]{e0, …}`。
- 保持分配先于元素求值、元素从左到右、生产者只执行一次；将分配、`dup`、索引、存储及元素生产者的来源归入该表达式。
- 无法证明数组不逃逸、长度/索引/组件转换、控制流或异常顺序时保留完整拒绝；不把普通数组赋值重写为初始化器。
- 只扩展已有 `NewArray` 的表达能力与局部形状认领；不新增全局别名分析、一般数组初始化推断器或外部类型层级解析。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：对字节码直接证明的数组初始化链恢复 Java 数组创建表达式。

## Impact

涉及现有 `NewArray` AST/构建/发射、数组 store 与 `dup` 的认领边界、类型/来源及回归测试。根证据见 `../../evidence/java-syntax-2026-09-22/array-initializers/`：1281 B、13 个 Code、SHA-256 `998bdb54c863d92cb63cc08c654df21bc674961c652fc46a5c3eaf7de2915c86`；冻结 CLI `7747b60a…` 输出 38 个引用，整类 `javac` 失败。本项在部分维度分配形状确定后串行实施，避免同时改动 `NewArray`。
