
## 修复实测（2026-10-04，coder 修正轮；本文档副本随实现 worktree 合回）

修复按裁决落点实现，无障碍：折叠决策块结束后、`projected_text` 未认领文本时（涵盖 joint/instance/static-only 三条拒绝路），对屏选成员以 `pool_spelled_members=true` 重跑（`rerun_pool_spelled_structural_reads`：`analyze_method_ir` + `recovery_from_with_class_candidates` 既有入口；`recovery_from_with_class_candidates` 增旗标参数，既有三个调用方传 `false`）。屏选 = 方法体文本含「行集内 `$` 名（含数组形）`.class.<结构反射方法>(`」——保守筛子，精确判定在重跑守卫。采纳规则：仅当重跑文本引用守卫原句（"which this text spells in the pool's form"，run 自己的解释）才替换 outcome 与组装文本（成员文本在类文本中逐字原样放置，`replacen` 单点替换）；重跑无守卫引注则保留第一遍（无 feeds 退化保护）。

| 步骤 | 修复后实测 |
| --- | --- |
| jarde family 口径 `RF` | `simpleName()` 整方法引注（1 处 `@bytecode`），引注即守卫原文：`the call at BCI 2 reads getSimpleName … pool's form …`（[results/RF-jar-after-guard.txt](results/RF-jar-after-guard.txt)） |
| 家族 `javac --release 8` | **exit 1**（`RF.java:18: 错误: 缺少返回语句`——响亮失败，[results/recompile-refused.err](results/recompile-refused.err)） |
| `main` 旁证 | `RF$Inner.K` 纯字段读不受影响（身份缝池形，未动） |
| 四向 | A12-jar 恢复（`Nested.class`，0 引注）✓ / A12-single 拒绝 ✓ / RF-jar 拒绝 ✓ / N2 `midLevel` 恢复逐字 + `multiLevel`/`recvChain` 拒绝 ✓ |
| corpus | 465 类双腿（基线 vs 修复后）**仍 0 差异** |
| 门禁 | 全量测试 296 目标 / 2933 passed / 0 failed；fmt 干净；CI clippy `--all-features` exit 0；openspec 268/268 |

测试：`tests/nested_class_literal_values.rs::a_refused_fold_reruns_the_structural_reader_and_the_family_stays_loud`——断言拒绝文本、`RF$Inner.K` 旁证不受伤、**家族负向编译**（javac 必须失败）、`RF$Inner` 单元自身仍恢复可编译、原类输出冻结。
