## Why

固定的 CF-10 步长索引反例中，循环条件 `i < values.length` 在 BCI 6 使用 `arraylength`。Jarde 的 Region 规则虽已有数组长度表达式构建能力，却把这个条件生产者当作不能置于循环测试的指令，连带使完整方法被引用回退，源码缺少返回语句。原 class 与固定 JADX 的完整源码都能以 Java 8 重编并输出 `4`。

## What Changes

- 仅当同一测试块中的 `arraylength` 是终端条件分支的 SSA 生产者，且现有表达式构建能按原顺序、一次性呈现时，允许循环规则把它留在条件中。
- 恢复步长为 2 的索引循环及其循环后返回；不把它改写为 foreach。
- 未被条件消费的数组长度读取、额外效果或不完整证据继续拒绝；不放宽其它循环形状、数组操作和局部声明门槛。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：允许经同次 SSA 证明的数组长度条件生产者保留在循环测试中。

## Impact

影响 `jarde-java` 的现有 Region 测试块准入和定向回归，不新增公开 API、crate、依赖或通用类型推断。CF-10 的 `List→Iterable` 调用是另一独立任务；CF-08 的循环分支汇合和其它 counted-loop 形状也不纳入本项。
