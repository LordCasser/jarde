## ADDED Requirements

### Requirement: 循环体内单行具名 catch 的异常范围准确恢复

当 Java 8 方法的循环体内包含单行具名 catch、其保护区不跨该循环的测试或递增块、handler 无普通前驱且其唯一续接点汇合回循环内部时，系统 SHALL 只在异常派发顺序、效果顺序与保护区结构全部闭合时恢复该 try/catch，且恢复输出重编运行后与原 class 行为一致。

#### Scenario: 固定三循环形锚点
- **WHEN** 输入为固定 `DV`（plainFor/whileLoop/forEach）与 `DF`（loopCatch/loopFinally）Java 8 class 并恢复、重编、运行
- **THEN** 三循环形的循环内 try/catch SHALL 全部恢复，完整类通过 `javac --release 8` 与 `java -Xverify:all`，`loopCatch({1,-2,3})` 输出 `3`，与原 class 一致

#### Scenario: 对照组零回退
- **WHEN** 输入为无 try 的同构循环、循环内 try-finally（continue 交互）与已验收 fragmented-loop-catches 既有 fixture
- **THEN** 既有恢复 SHALL 逐字节不变，循环内 try-finally 的等价分布输出与原 class 行为一致；固定 DF.loopFinally 的 handler51 质量差距 MUST 独立记录并以实际完整类编译/运行判断验收，不得从诊断推断编译失败、删除该方法或修补生成源码

#### Scenario: 结构判据不满足时维持拒绝
- **WHEN** 保护区跨循环测试/递增块、handler 有普通前驱、异常表行交叉或共享、或 SSA 局部读写不能在同一词法 catch 内闭合
- **THEN** 系统 MUST 保持整方法响亮拒绝并给出可定位诊断，不得因放宽不可归约检查而发布行为不同的输出（jadx 在本形状把 throw 提出 try 外的行为错码是反例）

#### Scenario: for-each 形与插桩结论
- **WHEN** 输入为 for-each + try/catch 简化形（无 throw-if）
- **THEN** 恢复 SHALL 与 Q-iii 同次事实对应；独立 SSA/作用域门 MUST 拆分具体验证及实现任务，未证明时保持准确拒绝，不得以普通 for/while 通过关闭三形验收

#### Scenario: 无限异常重试与正常 break
- **WHEN** 输入为本地 JADX TestTryCatchInLoop 的单行具名 catch 代表，try 正常完成唯一 break，handler 唯一普通回跳 try 起点，普通 CFG 本身没有自然环
- **THEN** 系统 SHALL 在保留原异常派发的局部所有权证明后，用既有 Endless/Try/LoopBreak/LoopContinue 完整恢复；完整类双 JDK、debug/none 重编运行与原计数/事件顺序一致，未匹配的 AssertionError 仍逃逸

#### Scenario: 重试所有权与停止边界
- **WHEN** retry 候选存在 handler 普通入口、另一回跳/出口、共享 protected owner、无法证明的 normal continuation，或新增证明中预算耗尽/取消
- **THEN** 系统 MUST 保持定位明确的拒绝或原 StopReason，不能重入并重复认领 try、吞掉方法尾部、停止后发布部分正文或禁用 overlap 检查
