## ADDED Requirements

### Requirement: java.lang 装箱家族的调用实参上转型可呈现

系统 SHALL 在调用实参的呈现类型为 JDK 8 java.lang 装箱类或 String、且要求类型为其固定层级（六数值装箱→Number；装箱与 String→Comparable；String→CharSequence）可达祖先时，按既有平台回答呈现。既有闭集、`List→Iterable` 与用户类 SHALL 零变化。

#### Scenario: 泛型数值方法调用恢复

- **WHEN** `larger(3, 7)`（`<T extends Number>` 擦除形参）三方 Java 8 重编
- **THEN** 调用完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入走既有闭集、Boolean→Number 或用户类
- **THEN** 前者与本变更前逐字一致；后两者保持拒绝
