## 1. 锁定证据契约

- [x] 1.1 冻结原 `ChainExtraBoundary.assign` BCI 30 为已呈现 true 对照，以及同方法的额外入口/异常边 verifier-valid 控制：field plan claim 为真、最终 `@bytecode` 含来源但无 FieldAssign，现有 record/summary 误报；核对 ordinary Field、FieldAssign、两条 field++ 表示、constructor 与 `<clinit>` 的最终 AST。
- [x] 1.2 枚举 `Program.stmts` 全部语句/表达式、后构建变换、`FieldIncrement` 计划和报告物化调用点；确定已提交 AST 回执的最小私有落点及预算边界。

## 2. 按最终 Java 操作报告

- [x] 2.1 在最终 Program 上有预算地收集 field plan 可证明且实际发射的字段 BCI；引用注释不算呈现，synthetic accessor 不混入本方法字段；field++/++field 用计划与最终语句互证，不扫描文本。
- [x] 2.2 让 FieldRecord、未呈现理由和全方法 summary 共享该回执；保留 plan claim 给 Builder，保持原 plan 拒绝代码、RuleDetails 选择/range/逐项计费及 stop 状态。

## 3. 独立验收

- [x] 3.1 两个 `ChainExtra` 拒绝控制的 BCI 30 为 `presented=false` 且仍可追来源，原 class 则保持 true；普通读/写、嵌套、field++、constructor、`<clinit>` 成功场景仍为 true；没有已发射字段的 fallback 和合成 accessor 不误报。修正旧“source-map 命中即算呈现”的弱测试。
- [x] 3.2 验 essential/all/range/预算/取消、定向及相邻字段/来源回归，格式、适用 Clippy、`openspec validate report-committed-field-presentations --strict`；记录剩余边界和 Cargo target 清理。
