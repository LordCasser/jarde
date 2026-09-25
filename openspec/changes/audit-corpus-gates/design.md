## Context

参见 proposal 的 Why。当前 reader 测试递归枚举 `tests/fixtures` 下的 class 文件并将 population/结构指标作为一个整体断言；fingerprint 测试扫描 `tests/fixtures` 与 `fuzz/corpus` 的非排除文件，并同时核对清单和源码中的分类表。当前 fingerprint 运行报告 767 个文件，其中 `tests/fixtures` 751 个、`fuzz/corpus` 16 个；146 个未登记文件集中在 7 个 fixture tree：`p3-conditional-values`（59）、`proved-java-structure`（48）、`p3-carried-conditional-arguments`（13）、`p3-nested-array-initializers`（10）、`recover-generic-enclosing-member-call-sites`（8）、`p3-special-dispatch`（4）、`p3-typed-catch-boundary-return`（4）。其中有 80 个 class 文件，其余是 Java 源、运行基线和脚本。

## Goals / Non-Goals

**Goals:**

- 为 146 个差异文件记录分类及证据，确认测试消费关系、来源说明、可重建方式和可能的临时产物。
- 先解决被确认的临时产物，再更新 reader census、fingerprint 的源码分类和 JSON 索引，并核对差异。
- 保留这两条定向失败命令用于复放。

**Non-Goals:**

- 不顺手处理枚举恢复实现或其他架构债务。
- 不因 fingerprint 收录规则本身而扩大/重构 corpus 扫描器。
- 不在审核未完成时删夹具、覆盖指纹或只把旧计数改成当前观察值。

## Decisions

- 以 fingerprint 失败列出的完整文件差异作为审查清单；以 reader 的真实 `.class` 递归扫描结果核对其 class 子集。Fingerprint 差异不仅包含 class，还包含源文件、运行输出、脚本，因此不得只审 class。
- 优先使用测试引用、fixture README/哈希、证据目录、重建脚本及 git 状态证明来源。已有证据显示大部分属于新恢复用例或结构样本：例如 `p3-conditional-values` 的 README 描述 javac 语料及输出基线，generic call-site 的 README 给出证据来源、编译命令和全部八个 class 的 SHA-256；interface/special-dispatch 与 typed-catch 的测试/README 也明确引用其 class。
- 将三个没有测试引用、且命名/说明指向 source-only runner 的文件列为待确认临时产物候选：`tests/fixtures/p3-carried-conditional-arguments/ControlRunner.class`、`tests/fixtures/p3-conditional-values/mixed-short-circuit-instance-field/Runner.class`、`tests/fixtures/p3-nested-array-initializers/v8/ExtraUseRunner.class`。mixed instance-field README 明说 Runner 只保留源码；nested-array README 单独列出 ExtraUse subject，未列 runner。候选不自动等同于可删除项，实施时须查其重建/历史意图。`tests/fixtures/p4-modern/out` 已由 fingerprint 排除规则显式覆盖，不属于本次 146 项。
- `p3-carried-conditional-arguments` 当前由源码与 compiled inputs 构成但没有对应 README 或 test 文件引用，不能据名称认定其 class 和 ControlRunner 都应留存；应通过相关变更证据与预期测试使用再作决定。
- census 应在最终 fixture 集合上重新测量；除了 class 数量，还须保留测试断言的 bodies、handlers、branch targets、subroutines 等指标。fingerprint 只在确认保留集合后由源码分类表生成 JSON，并审查 JSON diff；不能以生成器成功代替审查。

## Risks / Trade-offs

- [错误移除仍有价值的夹具] → 先核对消费者和来源，待判定项留在范围内并阻止更新门禁。
- [只放宽 census 会把临时 class 永久化] → 在 census 更新前先裁定上述候选并检查整份新增 class 清单。
- [fingerprint 收录文件范围大于 reader 的 class 范围] → 分别核对全体 146 项和 80 个 class 子集。
