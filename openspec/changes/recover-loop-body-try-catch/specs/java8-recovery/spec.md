## ADDED Requirements

### Requirement: 循环体内单行具名 catch 的异常范围准确恢复

当 Java 8 方法的循环体内包含单行具名 catch、其保护区不跨该循环的测试或递增块、handler 无普通前驱且其唯一续接点汇合回循环内部时，系统 SHALL 只在异常派发顺序、效果顺序与保护区结构全部闭合时恢复该 try/catch，且恢复输出重编运行后与原 class 行为一致。

#### Scenario: 固定三循环形锚点
- **WHEN** 输入为固定 `DF`（plainFor/whileLoop/loopCatch）与 `DV` Java 8 class 并恢复、重编、运行
- **THEN** 三循环形的循环内 try/catch SHALL 全部恢复，完整类通过 `javac --release 8` 与 `java -Xverify:all`，`loopCatch({1,-2,3})` 输出 `3`，与原 class 一致

#### Scenario: 对照组零回退
- **WHEN** 输入为无 try 的同构循环、循环内 try-finally（continue 交互）与已验收 fragmented-loop-catches 既有 fixture
- **THEN** 既有恢复 SHALL 逐字节不变；循环内 try-finally 的等价分布输出与原 class 行为一致

#### Scenario: 结构判据不满足时维持拒绝
- **WHEN** 保护区跨循环测试/递增块、handler 有普通前驱、异常表行交叉或共享、或 SSA 局部读写不能在同一词法 catch 内闭合
- **THEN** 系统 MUST 保持整方法响亮拒绝并给出可定位诊断，不得因放宽不可归约检查而发布行为不同的输出（jadx 在本形状把 throw 提出 try 外的行为错码是反例）

#### Scenario: for-each 形与插桩结论
- **WHEN** 输入为 for-each + try/catch 简化形（无 throw-if）
- **THEN** 恢复 SHALL 与 Q-iii 插桩结论一致：同门则同片覆盖，独立门则维持拒绝并在 change 目录记录
