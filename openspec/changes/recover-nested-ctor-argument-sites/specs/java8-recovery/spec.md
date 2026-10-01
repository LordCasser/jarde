## ADDED Requirements

### Requirement: 构造实参位的完整嵌套构造可呈现

系统 SHALL 在外层构造的实参位出现完整内嵌构造站点（同块 `new; dup; …; invokespecial`、ctor 与内嵌分配匹配、值单用途、区间连续）时，递归证明该内嵌站点并将其呈现为外层实参位的构造表达式。既有构造呈现与内嵌非完整站点（缺 ctor、双用途、跨块、超两层）SHALL 逐字不变或保持拒绝。

#### Scenario: 嵌套构造链恢复

- **WHEN** `readCause(new Exception("top", new Exception("inner")))` 三方 Java 8 重编运行
- **THEN** 内外构造完整呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: 既有与负例不变

- **WHEN** 输入为单参构造作方法实参、throw 位构造，或内嵌双用途/缺 ctor/三层形状
- **THEN** 前两者与本变更前逐字一致；后者保持既有拒绝
