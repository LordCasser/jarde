## ADDED Requirements

### Requirement: java.util 集合层级的调用实参上转型可呈现

系统 SHALL 在调用实参的被呈现类型为 JDK 8 `java.util` 集合类或接口、且要求类型为其经固定层级（实现→接口、接口→超接口，至 `Collection`/`Iterable` 根）可达的祖先时，按既有平台回答呈现该实参并保留要求类型拼写。`List → Iterable` 既有行为、非 java.util 类型与用户自定义集合 SHALL 不变（后者保持拒绝直至 resolution 层证明启用）。

#### Scenario: 集合传参恢复

- **WHEN** `max(new ArrayList<String>(...))` 形态以原 class/固定 JADX/Jarde 三方 Java 8 重编运行
- **THEN** 调用与结果局部完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为既有 `List→Iterable` 场景、用户类实现集合接口、或表外类型
- **THEN** 输出与本变更前逐字一致
