## ADDED Requirements

### Requirement: 已证 else-if 链的源码拼写

当一个已恢复的 `If` 语句的 else 正文恰为一个直接子 `If` 时，系统 SHALL 将两层语句写成 Java 8 `else if` 链，且 MUST 保留各条件的原求值次序、每个子语句的物理来源、source-map 覆盖、预算和停止语义。该呈现 MUST NOT 改动 CFG、SSA 或 AST。无法满足单子 `If` 形状时，系统 SHALL 保持现有 else 块拼写。

#### Scenario: 连续三个 else-if 条件

- **WHEN** `ChainOnly.chain` 的四次依序 `matches` 形成单子 `If` 链，默认分支另有一次 `hits += 10`
- **THEN** Jarde 完整类源码 SHALL 含三个 `else if`；原 class、固定 JADX、Jarde 的完整 Java 8 源码重编和验证运行 MUST 逐字同为五行 `10:1`、`20:2`、`30:3`、`40:4`、`10:14`

#### Scenario: else 块不只是一个直接 If

- **WHEN** else 正文有两个以上语句，或唯一语句是 fallback、guard、循环、赋值或其它非 `If` 节点
- **THEN** 系统 MUST NOT 将其压成 `else if`；原来源和语句顺序 MUST 保持

#### Scenario: 子条件的来源与停止

- **WHEN** 一条 `else if` 链包含多个各自有物理锚点的子 `If`，或 emitter/source-map replay 遇到预算停止或取消
- **THEN** 每个子条件 SHALL 保留自己的来源 span 与计费；停止时 MUST NOT 发布截断的完整源码或伪造缺失 span
