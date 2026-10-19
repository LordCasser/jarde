## ADDED Requirements

### Requirement: Preserve effectful body conditions after proved loop tests

对于已有纯循环测试之后、可由正常控制流独占的带效果条件主体，恢复器 SHALL 保留已证明的循环测试、主体效果的原执行位置、短路次序和break所属循环。系统 MUST 不因尝试把该主体继续合并为循环条件失败而丢失原本可完整表达的循环，也 MUST 不把效果搬入纯条件、移出循环或重复求值。未证明的结构仍遵循原拒绝和停止契约。

#### Scenario: Lookup and null guard remain inside each iteration

- **WHEN** 完整TestBreakInComplexIf原件在纯计数测试之后执行字符串构造、Map.get、null判断及受保护的字段判断
- **THEN** 输出 SHALL 是完整可编译的结构化目标方法，每次进入主体只执行一次lookup，null时不读字段，break时不执行后续length与计数更新，有限case的返回、异常类别、stdout和输入状态与原件相同

#### Scenario: Preserve a longer proved AND prefix

- **WHEN** 两个或更多已证明的AND循环测试后有带效果且可独占的主体条件
- **THEN** 输出 SHALL 保留整个已证明测试前缀和主体原执行位置，所有正常边和物理块有准确归属，不丢弃前缀测试或重复主体

#### Scenario: Preserve excluded shapes and operation stops

- **WHEN** 候选涉及初始表头效果、OR链、未证明目标、异常边、非独占入口，或恢复操作达到预算/取消
- **THEN** 系统 SHALL 保持相应既有证明或拒绝边界，停止仍为停止，不以成功主体替代结构失败，不发布半个正文或来源表

#### Scenario: Stable whole-class output and physical origins

- **WHEN** 同一完整debug或no-debug原件分别经class和plain JAR以及default和all evidence入口恢复
- **THEN** 系统 SHALL 保持目标身份、质量、完整源码及来源一致，来源均指向本方法物理指令；保留完整原件/JADX outer树及check/Point证据，并以候选API输出的完整TestCls/Point源码对重编和隔离运行（不注入原始Point binary），逐项报告来源缺口与工具链边界
