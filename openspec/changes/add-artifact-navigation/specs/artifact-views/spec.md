## ADDED Requirements

### Requirement: Class navigation listing preserves physical containment

类导航列举 SHALL 在调用方声明的物理范围内返回类条目，每个条目携带其真实物理位置：standalone snapshot root，或 container origin 与 entry ordinal/raw name。同一内部名存在多个物理定义时 MUST 全部返回，MUST NOT 合并为一条、去重或按遍历顺序 first-wins。列举 SHALL 区分两种证据等级：archive entry 候选列举只按大小写敏感的 raw name 规则给出候选，MUST NOT 声称该类已找到、可加载或已解析；header 确认的类声明列举 SHALL 真实读取每个候选的 class Header，只有读取成功的候选才进入确认集合，并携带该次读取的 `this_class` 与 class access flags。确认列举在失败、预算耗尽或取消时 MUST 返回带物理 origin 的诊断、可靠前缀与非 Complete execution，不得用完整空列表表示“范围内没有类”。

#### Scenario: Entry candidate is not a declaration

- **WHEN** 容器含一个 raw name 以 `.class` 结尾但字节无法构成合法 class 的 entry，而调用方只请求 archive entry 候选列举
- **THEN** 该 entry 仍作为候选按真实 origin/ordinal 返回，报告不把路径当作类声明证据；损坏诊断只出现在读取该候选的确认列举中

#### Scenario: Duplicate definitions stay distinct

- **WHEN** 同一 WAR 的上层目录入口与显式展开的嵌套库入口含同一内部名、甚至相同字节的类
- **THEN** 两条条目各保留自己的 container origin、entry ordinal 与 raw name，不合并，也不因内容摘要相同而丢失任一定义（验收 A07、A08）

#### Scenario: Header-confirmed listing reads the header

- **WHEN** 调用方请求 header 确认的类声明列举
- **THEN** 每个确认条目携带真实读取的 `this_class` 与 class flags，读取计入既有 `class_headers`/`class_bytes` 预算；未读取 Header 的候选不出现在确认集合中（验收 A16）

#### Scenario: Path name and declared name stay separate

- **WHEN** entry 的 raw path 暗示的内部名与 Header 的 `this_class` 不一致
- **THEN** 条目同时保留真实物理 raw name 与 Header 事实，不按其中一方改写另一方，也不据此声称该 entry 是可绑定的类定义

#### Scenario: Partial listing keeps the reliable prefix

- **WHEN** 确认列举在预算耗尽、取消或结构损坏时停止
- **THEN** 返回已确认前缀、定位到物理 entry 的诊断与非 Complete execution，未扫描范围显式标出，不返回完整空列表（验收 A14）

### Requirement: Ambiguous navigation names return candidates

按名称查找类或成员时，匹配不唯一 SHALL 返回全部候选及其物理依据与选择所需信息；系统 MUST NOT 静默返回第一个匹配项，也不得用显示名替代物理身份作为选择结果。friendly 输入（点分隔类名、内部名、成员 descriptor 与过滤条件）SHALL 被接受，但任何匹配结果 MUST 绑定真实物理身份（definition location、entry ordinal、class bytes、成员 descriptor）。名称在已扫描范围内无匹配时 SHALL 返回空候选与已扫描范围，MUST NOT 伪造定义或声称已解析。

#### Scenario: Duplicate class names return candidates

- **WHEN** 两个物理位置含同一内部名的类，且调用方只按名称查找
- **THEN** 返回全部候选及各自 origin/entry 身份与选择依据，不返回其中一个并宣称已找到唯一类（验收 A07）

#### Scenario: Friendly name binds a real definition

- **WHEN** 调用方用点分隔名（如 `com.demo.A`）与内部名（如 `com/demo/A`）两种写法分别查找同一唯一候选
- **THEN** 两种写法返回同一物理身份，结果的 location/ordinal/class bytes 可复核，不以拼写文本作为身份

#### Scenario: Overloaded member selection

- **WHEN** 同一类中同名方法存在多个 descriptor 而调用方未给出 descriptor
- **THEN** 返回全部候选及其 descriptor/flags，不选择第一个；调用方给出 descriptor 后匹配唯一并绑定该成员的物理身份（验收 A13）

#### Scenario: Empty match is not a definition

- **WHEN** 名称在已扫描范围内没有匹配
- **THEN** 返回空候选、已扫描范围与真实 ExecutionReport；完整扫描下的无匹配是 Complete 结果，不被改写成扫描失败，也不伪造定义或错误身份（验收 A14）
