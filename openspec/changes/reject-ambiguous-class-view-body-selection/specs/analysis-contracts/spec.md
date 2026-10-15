# Spec Delta

## MODIFIED Requirements

### Requirement: Task-oriented target selection binds real physical identity

任务导向操作 SHALL 接受两种目标输入：friendly 名称（点分隔类名、成员名、descriptor 与可选过滤条件），或调用方已有的物理身份；两种输入 MUST 收敛到同一物理身份（definition location、class bytes、variant 与成员 descriptor），并复用与物理导航相同的名称匹配与歧义规则。匹配不唯一时操作 SHALL 返回候选及各自物理依据并不执行，MUST NOT 静默取第一个；调用方给出的身份不属于本次 artifact 时 MUST 返回可定位的输入错误，不得改用同名定义。名称搜索 SHALL 在范围与相关成员表完整后才作唯一或缺失判断；若搜索因损坏、预算或取消未完成，操作 MUST 返回独立的未完成选择结果，保留可靠候选、coverage、execution、diagnostics 与实际用量，不启动方法分析/恢复，不把零候选当成未找到，也不把一个候选当成唯一。

#### Scenario: Friendly name selects one definition

- **WHEN** 调用方用 friendly 名称发起操作，且范围内只有一个匹配的物理定义
- **THEN** 操作绑定该定义并发布所选择身份；结果与直接给出同一物理身份的请求一致，不以显示名作为身份

#### Scenario: Ambiguous name is not resolved silently

- **WHEN** 名称匹配多个物理定义或同类内多个同名 descriptor
- **THEN** 操作返回候选与选择依据，不执行分析、不选择第一个；调用方回传所选身份后才继续（验收 A07）

#### Scenario: Existing identity is used as given

- **WHEN** 调用方给出既有 `PhysicalMethodId`
- **THEN** 操作按该身份执行；身份属于另一 snapshot 或其他物理定义时返回可定位的输入错误，不替换为同名定义（验收 A18）

#### Scenario: One candidate precedes a damaged candidate

- **WHEN** 名称搜索已确认一个匹配方法，随后同名候选读取失败
- **THEN** 返回未完成选择及已确认候选和失败来源，不执行该方法、不丢弃搜索诊断；该候选只有在调用方显式选择其物理身份后才可用于独立操作（验收 A07、A14）

#### Scenario: No candidate is confirmed before a stop

- **WHEN** 损坏、预算或取消使搜索在确认任何目标之前停止
- **THEN** 返回零可靠候选的真实停止结果，不报告完整缺失或用法错误；完整搜索确实无匹配时才允许未找到结果（验收 A14）

#### Scenario: Member selection has an unread suffix

- **WHEN** 类声明已确认，但相关成员表在目标选择完成前中断
- **THEN** 保留成员表停止和已确认前缀，不把未到达的成员宣称为不存在，也不把部分重载集合宣称为唯一匹配

#### Scenario: Listed body identity does not select one duplicate record

- **WHEN** 完整类成员表含两个或更多 raw name 与完整 descriptor 相同的物理记录，调用方用其中任一列举项提供的不含 member ordinal 的物理方法身份请求 class-view 方法体
- **THEN** 操作 SHALL 返回既有歧义结果与全部匹配记录，保留每项原 index、flags 和物理来源；无论记录为 abstract、native 或有 Code，也无论记录顺序，MUST NOT 首取某项或返回唯一的无方法体结论

#### Scenario: Body name and identity share duplicate ambiguity

- **WHEN** 调用方对同一完整物理成员表分别用名称加精确 descriptor，以及任一匹配项的物理方法身份请求方法体
- **THEN** 两种请求 SHALL 使用一致的歧义判定与候选顺序；歧义结果 MUST 在类准备和方法体解码前产生，method_bodies、code_bytes 及 IR 用量不得因该歧义请求而增加

#### Scenario: Foreign owner remains an input error before body ambiguity

- **WHEN** 请求的物理方法 owner 不属于已绑定的类，而该类中存在相同 raw name/descriptor 的一个或多个记录
- **THEN** 操作 MUST 保留原 foreign-owner 输入错误，MUST NOT 用这些记录替代请求，也不得把该错误改为本类的歧义或无方法体结果

#### Scenario: Unique declarations retain their own body kind

- **WHEN** 完整成员表中请求的 raw name/descriptor 唯一，目标分别为 abstract、native 或有 Code 的声明
- **THEN** class-view SHALL 保留原逐 body 结果、原无方法体种类及实际 Code 解码；唯一 abstract/native 声明不得新增方法体准入，正常 Code 声明不得因新增歧义门而被拒绝

