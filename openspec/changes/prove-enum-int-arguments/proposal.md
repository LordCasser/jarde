## Why

普通 Java 8 enum 常量的 int 构造实参只要不是单个字节码 literal，当前 class-source 证明就拒绝整个常量组。有限静态字段读和加法表达式会因此退回到 enum body 中不合法的普通字段声明；冻结对照证明这些参数可由相同构造点的代码事实精确恢复并重编运行。

## What Changes

- 在现有完整 enum 常量组证明中支持受限 int 实参：已有整数 literal、单个 int `getstatic`，以及该字段读与一个整数 literal 直接相加的表达式。
- 只有每个常量、初始化前缀与后缀、constructor call/字段写、`$VALUES`/标准 enum 方法和 use census 全部通过时，才将实参放入完整 enum 声明；未识别或未完整证明时继续原子拒绝整个组。
- 保留静态字段读取的位置和次数，不把值折叠为编译期常量；整数加法的顺序与溢出语义随原表达式保留。
- `String...`、数组构造实参和其他构造器参数类型明确留给独立 change。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩展完整类源码中的已证明 Java 8 enum 常量，使其保留有限 int 静态字段/加法构造实参，并在组内任一形状无法证明时整体降级。

## Impact

实现会扩展 `src/enum_constants.rs` 的类级 enum 证书与 `src/class_source.rs` 的常量投影；可能复用 `jarde-java` 已有的静态字段和整数加法表达式拼写。行为边界由 `openspec/specs/java8-recovery/spec.md` 的 delta 规定。无需新增 CLI/API、依赖或顶层分析阶段。原 class、JADX 1.5.6 与 Jarde 的冻结输入、完整输出、SHA、javap 和运行对照位于 `openspec/evidence/java-syntax-2026-09-27/enum-constructor-arguments/`。本 change 不实现 `String...`。
