## Why

固定 JADX `TestDup2x1` 对应的 `return this.value = input` 在 Java 8 class 中生成 `dup2_x1`。原始与 JADX 完整源码可重编运行，Jarde 把该指令视为未知操作，既不能写字段赋值也没有返回语句，完整源码无法编译。该差距属于 EM-07 字段赋值结果，不应混入 EM-19 数组读写。

## What Changes

- 对准确的 `aload_0; lload_1; dup2_x1; putfield self.field:J; lreturn` 形态，证明两份 category-2 值分别流向字段写入和方法返回，恢复等价的 `this.field = arg1; return arg1;`。
- 在字段身份、宽度、SSA 使用、指令/异常范围、返回路径或预算证据不完整时保留现有拒绝，不把其它 `dup2_x1`、静态字段、数组写入或带副作用右值纳入首片。
- 增加原/JADX/Jarde 的 Java 8 完整源码重编、验证运行以及错误 owner/消费者/异常处理和预算停止负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 对已证明的单个实例 `long` 字段赋值并返回该值，发布可重编、行为等价的结构化源码。

## Impact

影响 `jarde-java` 同次方法 SSA/字段写入恢复、定向测试与 EM-07 冻结证据；不改核心 crate 边界、CLI/API、依赖或通用 Java 赋值表达式 AST。冻结基线见 [EM-07 审计](../../evidence/java-syntax-2026-09-27/em07-long-assignment/report.md)。
