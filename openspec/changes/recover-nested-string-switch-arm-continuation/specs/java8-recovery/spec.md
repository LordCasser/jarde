## ADDED Requirements

### Requirement: 已证 switch case 内的子 switch 续接不得丢失

当一个 switch case 内的子 switch 返回唯一、仍受该 case 支配且无外部入口的后继时，系统 SHALL 在该 case 的 Region 内继续走访到后继及其可证明的终点；MUST 保持每个 canonical block 唯一归属，并且只有完整 String-switch lowering 证书才能把两级分派呈现为字符串标签。

#### Scenario: 外层 default 内的 String switch

- **WHEN** 外层 default 在 BCI 62 包含内层 hash switch BCI 71，两个标签写 slot 4 后与 hash default 在 BCI 123 汇合，BCI 125 的最终 switch 到 BCI 152/154/156 返回
- **THEN** Jarde SHALL 在同一 outer arm 内拥有这些 block，完整源码 SHALL 含嵌套 String switch、无 `@bytecode` 并通过 Java 8 重编；原/JADX/Jarde `-Xverify:all` 的 a/b/c/d/null 五行 MUST 一致

#### Scenario: 普通碰撞与独立 hash 消费

- **WHEN** hash 仅供受证分派使用，或 hash 值另有独立消费者
- **THEN** 前者的普通碰撞/分组样本 SHALL 保持三方八行一致；后者 MUST 保留该消费者及物理分派，不得误折叠，原/Jarde 六行结果 SHALL 一致

#### Scenario: 续接边界不能证明

- **WHEN** 子 switch 后继有 case 外入口、穿过其它 case、回到循环、经过异常/子程序边或有多个不可比较续接
- **THEN** 系统 MUST 保留现有引用/解释或安全的原结构，不能丢失后继、越过父 arm 边界或双重认领 block

#### Scenario: 预算或取消中断

- **WHEN** case 后继/字符串分派证书的有界走访被预算或取消中断
- **THEN** 系统 MUST 返回不完整/中止结果，不得发表只包含第一级 switch 的完整源码
