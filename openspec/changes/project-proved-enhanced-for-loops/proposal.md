## Why

当前 Jarde 已能把冻结的 Java 8 数组遍历恢复为可执行的计数 `for`，把 `Iterable` 遍历恢复为可执行的 `while`，但都未呈现增强 `for`。本地 JADX 1.5.6 对同组数组样本能输出增强 `for`，对带调试局部表的 `Iterable` 对照也能输出该语法；这一源码形态差距适合在现有循环、SSA 与类型契约内，以严格等价证明逐步闭合。

## What Changes

- 对已证明的数组遍历循环呈现 Java 8 增强 `for`，保持数组表达式只求值一次、元素顺序、转换、副作用、异常和循环转移；证明失败时保留当前可执行的计数循环。
- 在数组切片独立验收后，调查并限定 `Iterable`／`Iterator` 形态的类型及转换证明；只有完整证据满足 Java 源语言要求时才投影增强 `for`，否则保留现有迭代循环。
- 固定原 class、JADX 与 Jarde 的完整类编译及 `-Xverify:all` 对照，并加入合法手写循环反例，检验不可误删索引消费或把不同数组合并。
- 不声称识别唯一原源码写法；不处理任意循环重构、跨类泛型推断或现有 P0 区域归属、异常局部作用域债务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：要求对完整证明等价的数组遍历呈现增强 `for`，并为后续可证明的 `Iterable` 投影规定保守边界与验证门槛。

## Impact

限定于 `jarde-java` 既有循环区域、SSA 事实、语句 AST、类源码构建与发射路径，以及相应测试和 OpenSpec 证据；不改变 core/CLI 分层，不增加依赖。数组与 Iterable 当前执行基线见 `../../evidence/java-syntax-2026-09-22/enhanced-for/`；其中 Jarde 的旧 `Iterable` 编译失败记录已由 `present-proved-java-structure` 的 2c.6 修复，须以本变更的当前重放为准。
