## ADDED Requirements

### Requirement: return 穿副作用 finally 的等价恢复

当 Java 8 方法的 try 块经 return 出口且其 finally 落穿并携带副作用时，系统 SHALL 只在 return 表达式求值次序（恰一次、先于 finally 副作用）与副作用次序全部可证时，以 temp 绑定或路径复制等价形式恢复，重编运行后与原 class 行为一致。

#### Scenario: 求值序判别锚
- **WHEN** 输入为固定 `FO`（`try{return bump();}finally{i=100;}`）Java 8 class 并恢复、重编、运行
- **THEN** 输出 SHALL 为 `1/100`（return 表达式先求值一次，finally 副作用后执行），完整类通过 `javac --release 8` 与 `java -Xverify:all`

#### Scenario: 位置锚点
- **WHEN** 输入为固定 `FY`（noCall/loopless）与 `FX.finReturn` Java 8 class 并恢复、重编、运行
- **THEN** 三锚 SHALL 恢复且行为与原一致（`3000/107/3000` 形）

#### Scenario: 对照零回退
- **WHEN** 输入为空 finally return（emptyFin）、continue/break 穿副作用 finally（finContinue/finBreak）与 finally 返回值形
- **THEN** 既有恢复 SHALL 逐字节不变

#### Scenario: 不可证明时维持拒绝
- **WHEN** 多 return 出口竞争同一 finally、finally 自含非局部出口、或求值序不可证（副作用与 return 表达式交织）
- **THEN** 系统 MUST 保持整方法响亮拒绝与可定位诊断
