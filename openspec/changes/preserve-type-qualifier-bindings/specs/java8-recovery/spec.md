## ADDED Requirements

### Requirement: Local naming preserves static type qualifier bindings

恢复普通静态方法调用及静态字段读写时，系统 SHALL 保证其类型限定符不因本方法分配的局部、参数或合成名称而绑定到另一变量或成员。修正名称 SHALL 保留原字段/方法目标、求值效果和已有名称证据，不能只以生成文本通过编译作为正确标准。

#### Scenario: A generated parameter would hide a default package owner

- **WHEN** 当前恢复规则产生的参数名与静态字段或外部静态调用的默认包 owner 同名，且参数类型也有可编译的同名成员
- **THEN** 完整恢复类 SHALL 可原样重编译且执行与原 class 相同，静态调用结果及实际被读写字段保持原目标，null 参数不改变这一结果

#### Scenario: A debug name would hide a package prefix

- **WHEN** 合法 debug 局部名与所需限定类型的包首段冲突，例如 java 与 java.lang.Math
- **THEN** 系统 SHALL 以现有可解释名称机制避免冲突并保留原 debug 名证据，完整恢复源码可以直接编译和执行

#### Scenario: Existing suffixes and synthetic names obey the same constraints

- **WHEN** 初始局部候选及其后缀已被本方法其它必要名称占用，或后续表达式需要分配合成名称
- **THEN** 所有新名称 SHALL 确定性地避开全部相关约束，变量引用与声明一致，不能重新遮住已有静态类型限定符

#### Scenario: Irrelevant class facts do not rename unaffected methods

- **WHEN** 冲突类型仅出现在本方法未使用的类事实中，或某方法没有实际名称冲突
- **THEN** 本项 SHALL 保持该方法已有名称决定，不为全类潜在名字进行无依据的重命名

### Requirement: Qualifier naming preserves bounded provenance

类型限定符所需的命名工作 SHALL 受现有预算与取消约束，且证据选择不改变名称或正文。

#### Scenario: Essential and full evidence use one naming decision

- **WHEN** 同一名称冲突方法在充足预算下分别请求默认和完整来源
- **THEN** 正文 SHALL 逐字相同，完整来源保留实际静态访问的指令、池项和成员身份，原名称证据仍可解释

#### Scenario: Limited naming work stops honestly

- **WHEN** 约束收集或名称碰撞处理耗尽预算或收到取消
- **THEN** 系统 SHALL 有界停止，不发布部分预占造成的错误正常绑定，不伪造未支付来源，也不在证据重放阶段另选名称
