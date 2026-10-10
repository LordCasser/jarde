## ADDED Requirements

### Requirement: Proven local source types survive frame erasure

系统 SHALL 在局部变量的全部写入能够证明同一个准确源类型时保留该类型，避免 JVM 的 int 形状或首个 null 初始化抹去 char 与引用类型事实。系统 MUST 保持原调用绑定、求值顺序、物理来源和未证转换的拒绝边界。

#### Scenario: A char result is stored then consumed by char append and switch

- **WHEN** TestSwitch.TestCls 中局部 c 的写入来自准确 charAt 返回类型，后续以 char 参数调用 append 并参与 switch
- **THEN** 系统恢复完整有效调用及 char 选择值；完整生成 class 原样重编运行保留所有字符、分组标签、空 case 与 default 效果，全部物理指令来源可查询

#### Scenario: A null initialized local is assigned only exact String values

- **WHEN** TestSwitchNoDefault.TestCls 中局部 s 初始为 null，所有其余写入均为准确 String，随后作为 String 参数传递
- **THEN** 系统恢复 String 声明、完整 switch 和调用；未匹配 case 保留 null，完整 class 原样重编运行与原 class 相同

#### Scenario: Writes do not prove a common narrow or reference type

- **WHEN** 局部写入包含未证 int→char、互不相同或未知引用、仅 null，或类型事实不能覆盖整个源变量生命周期
- **THEN** 系统保持准确较宽类型或现有明确拒绝，不以调用所需类型、debug 名称或局部类型注记单独伪造类型证据

#### Scenario: A type proof stops before publication

- **WHEN** 局部类型证明遇到预算不足或取消
- **THEN** 系统 SHALL 传播既有 Stop，MUST 不发布部分正文或部分来源映射
