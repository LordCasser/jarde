## ADDED Requirements

### Requirement: switch arm 到外层 loop 的出边恢复为 continue

系统 SHALL 在整数 switch 的某 arm 终边恰好落在外层 loop 的 latch 或 test 出口块时，将该 arm 以 arm 内 `continue;` 呈现，并从 switch join 候选中排除该 arm；其余 arm 的公共汇合为 join，全部 arm 均 continue 或自然落出时 join 即 latch。switch 的 selector 块 SHALL 唯一属于 Switch region。落点非 latch/test 的 arm 出边 SHALL 保持既有拒绝。

#### Scenario: continue 与 join 共存

- **WHEN** `while (…) { switch (…) { case …: …; default: if (…) continue; … } [join 语句] }` 三方 Java 8 重编运行
- **THEN** switch 与 join 语句、arm 内 `continue;` 全部呈现，`java -Xverify:all` 逐路径与原 class 一致

#### Scenario: join 即 latch

- **WHEN** 全部 arm 出口即 loop latch（无 join 后语句的 continue 形态）
- **THEN** 恢复为 arm 内 continue/自然落出，行为一致

#### Scenario: 边界与对照不变

- **WHEN** arm 出边到任意非 latch/test 块、或输入为无 continue 的 switch-in-loop 对照
- **THEN** 前者保持既有拒绝；后者与本变更前逐字一致

#### Scenario: 退化诊断真实

- **WHEN** 某形状仍不可构造而退化 quote
- **THEN** 最终诊断指向该 walk 的真实首个构造失败，Fallback 块集互斥无重叠
