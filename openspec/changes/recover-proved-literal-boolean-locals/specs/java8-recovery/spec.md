## ADDED Requirements

### Requirement: Proved literal boolean local flags preserve their consumers

系统 SHALL 在非参数局部标志的全部写入、合并和读取均被证明属于同一布尔用途，且存在独立布尔描述符消费时，将其 0/1 初始化与更新恢复为布尔声明和赋值，并保留相应消费语句；调试名称或整数形态本身 MUST NOT 构成布尔证明。

#### Scenario: Loop flag retains printing and labeled exit
- **WHEN** 完整输入含初始化 false、命中后置 true 并带标签跳出嵌套循环的局部标志，退出后参与布尔拼接与布尔返回，且其生产消费证明闭合
- **THEN** class/JAR 的 default/all 结果 SHALL 保留布尔赋值、带标签跳出、打印和返回；完整源码重编后，在有限原始用例上的返回、异常类别与 stdout SHALL 与原件相同

#### Scenario: Debug metadata does not supply missing proof
- **WHEN** 相同方法 Code 分别携带或不携带局部变量调试表
- **THEN** 两种输入 SHALL 获得相同的布尔类型结论；缺少代码证明时 MUST NOT 仅凭变量名称或调试描述符提升类型

#### Scenario: Mixed integer use or incomplete producer refuses promotion
- **WHEN** 存在非 0/1 或未证明写入、整数消费、歧义复用、未知合并输入，或只有整数分支测试而没有独立布尔消费
- **THEN** 系统 MUST NOT 通过本规则将该局部变量提升为布尔，也 MUST NOT 将消费语句伪报为已结构化恢复

#### Scenario: Proof stops preserve reporting and publication boundaries
- **WHEN** 布尔证明遇到真实预算耗尽或取消
- **THEN** 系统 SHALL 沿既有停止契约传播原因和实际用量，MUST NOT 发布未完成证明的声明、正文或来源映射，也 MUST NOT 把停止降为缺证成功
