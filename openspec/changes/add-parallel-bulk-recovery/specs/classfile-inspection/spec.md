## ADDED Requirements

### Requirement: Verified class preparation supports multiple method consumers

系统 SHALL 允许多个同类方法消费者复用同一次可信 class 准备：字节、内容身份、CP/结构、成员及其 Code 定位必须来自同一次读取。完整成员表已有定位时，逐方法消费 MUST NOT 重建类结构或重新全表搜索；方法处理的计数应随准备一次加实际方法需求增长，而非随方法数乘整个成员表增长。复用必须继续验证方法归属、重复候选、Code 外壳/边界及当前限制，不能信任调用方伪造的 offset。

#### Scenario: A large class has many methods

- **WHEN** 对同一类全部 M 个方法依次解码，完整结构一直持有
- **THEN** 类准备和成员定位构建各一次，后续不出现 M 次全表重扫；独立方法和复用路径的指令、CP 引用、debug、异常表及 spans 一致（B02）

#### Scenario: A locator has duplicate candidates or another owner

- **WHEN** raw name/descriptor 对应多个声明，或方法来自不同 class 字节/物理来源
- **THEN** 保留歧义或拒绝不匹配身份，不通过首个候选、显示名或外部 offset 绕过校验（B01/B02；A07/A18）

#### Scenario: A class is prepared without reading bodies

- **WHEN** 只准备 Header 与成员定位，某未需求方法含非法指令
- **THEN** 不解码该 Body；某方法被请求后才在自身边界报告损坏；准备不提高 dialect/verification/source 结论（B02；A16/A17）

#### Scenario: Preparation is incomplete

- **WHEN** 成员表结构读取没有完成
- **THEN** 不发布可证明任意名称不存在或唯一的完整定位结果；保留现有可靠前缀及诊断，不因新快速路径提升覆盖（B02；A14）
