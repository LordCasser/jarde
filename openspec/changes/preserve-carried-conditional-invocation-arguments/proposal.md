## Why

Java 8 构造器可先计算一个条件实参，再在保留该值的栈上计算第二个条件实参，最后执行 `this(...)`。冻结的 `ConstructorPairProbe` 原 class 与 JADX 的六条执行路径一致，Jarde 却在第二次汇合后的 BCI 22 无法证明第一个实参，引用调用并输出不可编译类。单条件实参已可恢复；缺口在已有条件值跨后续汇合的传递证明。

## What Changes

- 扩展现有条件值 Phi 证明，允许已证明的条件实参沿精确的栈传递/同值 Phi 穿过下一段条件图，到达同一次调用的实参槽。
- 仅在真实 CFG/SSA 来源、一次消费、调用实参顺序、类型和中途效果顺序均闭合时，复用现有 `Conditional` 表达式与 `constructor_call` 发射，保留所有 BCI 来源。
- 证明缺失、额外入口、异常或循环转移、独立效果或预算/取消时保持完整保守引用；单条件实参及其它 Phi 消费者行为不退化。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 允许条件值作为顺序求值的较早调用实参，跨后续条件实参图传递到同一调用。

## Impact

只使用现有 canonical CFG、SSA、Region、条件值证明、调用描述符、Builder 与 source map。无需构造器专用 Region、公开 API、新 crate 或生产期 JADX 依赖。本 change 只处理有界的顺序条件实参与调用，任意跨语句值搬运、普通控制流重构和 class 初始化重排另案。
