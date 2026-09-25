## Why

Java 8 的 `return receiver().value++` 和 `return data[index()]++` 会在写回新值后返回旧值。现有恢复没有认领跨越复制、读取、写入和返回的链：完整原 class 与 JADX 均可编译执行七行一致，Jarde 输出的两个方法却缺少 `return`，完整类无法编译。证据见 `../../evidence/java-syntax-2026-09-22/compound-assignments/postfix-analysis.md`。

## What Changes

- 对可由 SSA 身份完整证明的实例 `int` 字段和 `int[]` 元素后置 `++` 返回旧值，生成单次求值的 Java 表达式，保留写入、返回值、异常及调用顺序。
- 使用现有字段/数组成员证据和表达式发射路径，给后置更新增加真实表达式语义；失败时引用整条受影响链及物理来源。
- 以完整原 class、JADX、Jarde 的编译与执行对照验证正常、null、越界、溢出和额外消费者边界；维持已有简单字段递增与普通赋值回归。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 在严格数据流证明下恢复返回旧值的字段/数组后置递增；证明不足时仍保守拒绝并保留效果和来源。

## Impact

影响 `jarde-java` 的现有 AST、build 和 emitter，以及对应 Java 8 fixture/回归和来源记录。无新 crate、运行时依赖或公共 API；产品不执行目标字节码。本变更须在共享 `build.rs`/AST/emit 文件的 `recover-compound-lvalue-updates` 生产实现之后串行进行，互不扩大语法范围。前置递增、后置递减、局部保存旧值、表达式位置的任意更新与现有字段递增文本承载债务分别处理，不以本变更声称覆盖。
