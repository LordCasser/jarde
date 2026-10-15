> **历史范围说明（2026-10-15）：** 本条款的header命名终点与“两-sided快照零回退”共同保留各自已批准范围。[验证记录](../../verification.md)的“移动的既有pin”和“Root独立验收”明确记载仅含B的standalone快照到外部用户父类A的迁移及追认；目标名称来自已读header，不表示已选择或读取目标物理定义。[早期snapshot规划](../../../recover-snapshot-hierarchy-widening/specs/java8-recovery/spec.md)的双侧要求是其MVP范围，[后续数组元素条款](../../../recover-heterogeneous-array-init/specs/java8-recovery/spec.md)另有具体store场景。这里的引用不把数组验收扩成全部method快照外用户interface场景，terminal重复物理定义政策也未在本次文档澄清中裁定。

## ADDED Requirements

### Requirement: 快照内类到平台接口的实参单边扩宽

当调用实参的呈现类型是快照内类，且该类（或其快照内父链）的 class-file header 逐字列出目标接口/父类名时，系统 SHALL 以该 header 链为证明接受实参扩宽并呈现调用，即使目标自身不在快照中。

#### Scenario: 匿名 Comparator 主锚（comparator-anon 巡查）
- **WHEN** 输入为固定 `CP`（`Collections.sort(list, new Comparator<User>(){...})`，`javac --release 8`）的 class 并恢复
- **THEN** sort 调用 SHALL 完整呈现（无 "no safe reference conversion evidence" 引注）；去注释呈现编译若成功，运行输出 SHALL 与原 class 一致（`[10,20,30]`）

#### Scenario: 顶层具名实现类对照
- **WHEN** 输入为固定 `AH/AC`（`AC implements Comparator<String>`，同调用形）
- **THEN** 调用 SHALL 完整呈现且行为一致（`[a, bb, ccc]`）

#### Scenario: 无 header 关系仍拒
- **WHEN** 实参类型与目标接口无快照 header 链关系
- **THEN** 系统 SHALL 保持现行引用转换拒绝

#### Scenario: 两-sided 快照零回退
- **WHEN** 输入为既有 `recover-snapshot-hierarchy-widening` 的正/负 fixtures
- **THEN** 渲染与本变更前逐字节一致
