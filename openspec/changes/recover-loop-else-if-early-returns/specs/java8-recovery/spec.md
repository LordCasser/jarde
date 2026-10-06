## ADDED Requirements

### Requirement: 循环内 else-if 阶梯的早退呈现

当循环体的 else-if 阶梯（≥2 条件臂）含早退 return 臂，且该形状满足 MVP 判据（单层阶梯、无异常表交叉、无 switch 混合）时，系统 SHALL 将阶梯呈现为单一 if-else 树（早退臂为终止叶），方法行为完整。

#### Scenario: 二分查找主锚
- **WHEN** 输入为固定 `BS` 全类（bsearch 形）或 `CB.loopElseIfRet` 的 class 并恢复
- **THEN** 该方法 SHALL 完整呈现（0 canonical-overlap、0 引注）且剥离编译后 `-Xverify:all` 行为与原 class 一致

#### Scenario: 对照三形零回退
- **WHEN** 输入为 `CB` 的无早退/单 if-else/无循环三形
- **THEN** 渲染 SHALL 逐字节不变（既有恢复能力零回退）

#### Scenario: MVP 外形仍拒
- **WHEN** 阶梯与异常表交叉、双层阶梯或 switch 混合
- **THEN** 拒绝 SHALL 保持（如实记录拒绝态，不产出行为不同文本）
