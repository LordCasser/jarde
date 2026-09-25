## Why

Java 8 `Base()` 在短路条件求值后把结果写入静态字段；当前 Jarde 把汇合处的 `putstatic` 留为 `@bytecode`，生成的类源码仍能编译，却把 `visibleDuringSuper=true` 改成 `false`。这类“可编译但效果缺失”的输出不能用编译成功充当恢复成功，须单独恢复或明确拒绝。

## What Changes

- 在现有 SSA、Region 和表达式构建边界内，证明短路条件两条路径汇合后供给同一个字段写入的值与求值位置；证明成立时写出一次保持原求值顺序的字段赋值。
- 证明不成立时，保留包含字段写入及必要生产者的来源缺口，不能以缺少该效果的 Java 正文声称完整恢复。
- 固定 Java 8 原 class、JADX、Jarde 的编译和运行对照，特别检查基类构造期间的虚调用观察结果。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 短路条件值在控制流汇合后作为 `putstatic` 右值时，字段写入必须保留且只执行一次；证据不足时声明缺口，不能生成悄然丢失该写入的可编译文本。实例字段的接收者求值另行验收。

## Impact

预计触及 `crates/jarde-java` 的 Region 所有权、条件值/字段赋值及来源保留路径；为现有 `If` 无法表示的共享 false 块引入一种私有、受限 Region 形状，复用现有 AST、SSA、预算与 source map，不新增通用 CFG 重写器、crate 或外部依赖。`anonymous-super-dispatch` 的匿名类捕获/构造顺序属于 `present-proved-java-structure` 2.10/5.3，本 change 不调整捕获字段的写入顺序、不内联匿名类，也不修改 JADX 算法。
