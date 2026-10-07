# 门控实验（证书单独翻转锚；负例不翻）

命令：`sh results/01-gating.sh`（自测：每个渲染先断言 `// jarde: presentation of` 自头部；任一渲染无头部即停）。
基线二进制 = 父提交 `6f8ed47a` 的独立 worktree 构建（`/tmp/lk/baseline-target/debug/jarde-cli`），
本切片二进制 = 本 worktree 的 `target/debug/jarde-cli`。逐输入的渲染正文存于 `gating/`。

## 结果（`01-gating.out`）

| 输入 | baseline sha256 | current sha256 | 判定 |
| --- | --- | --- | --- |
| `patrol-jar`（锚，巡查冻结件） | `bb57f9a7ea8c…` | `79b61c4db829…` | **moved** |
| `v8-LK` | `bb57f9a7ea8c…` | `79b61c4db829…` | **moved** |
| `v8-javac8-LK` | `bb57f9a7ea8c…` | `79b61c4db829…` | **moved** |
| `v8-negatives`（4 个负例） | `27ec707215f0…` | `27ec707215f0…` | identical |
| `v8-javac8-negatives` | `27ec707215f0…` | `27ec707215f0…` | identical |
| `v8-probe`（io-wrapping 单方法探针） | `87e1fd1e8703…` | `87e1fd1e8703…` | identical |
| `v8-javac8-probe` | `87e1fd1e8703…` | `87e1fd1e8703…` | identical |
| `p3-handlers`（真两份副本、非锁） | `5aecd2387441…` | `5aecd2387441…` | identical |
| `local-scope-crossing`（1.2/1.3 负例面） | `31a5506f5b7a…` | `31a5506f5b7a…` | identical |
| `local-scope-positive` | `c823e459bc27…` | `c823e459bc27…` | identical |
| `io-jar`（锚 13 整类） | `42f4235f12e7…` | `42f4235f12e7…` | identical |

## 结论

1. **证书单独翻转锚**：`LK.put/take/tryLockQuick` 三法的整段拒绝（`jre_region_exception_edge` /
   `jre_guard_finally_copy`）被替换为 `lock(); try { … } finally { unlock(); }` 正文，且三腿（巡查 jar、
   javac 23、真 javac 8）渲染**逐字节相同**（`patrol-jar` 与两腿的 current 摘要一致）。
2. **负例不翻**：真两份清理副本形（非锁）、unlock 接收者不同一形、自保护行形、io-wrapping 局部句柄形
   在两侧逐字节相同——拒绝句与引注 BCI 都是基线原文（`01-gating.sh` 的自测之外，
   `tests/recover_lock_guard_loop_finally.rs` 把这些句子逐字钉住）。
3. **声明规划不是 gate**：`local-scope-crossing`/`local-scope-positive`/`io-jar` 两侧相同——本切片的 gate
   全在 guard/region 层，与 1.2/1.3 取证一致。
4. `LK` 正文的 diff（`gating/patrol-jar.{baseline,current}.java`）逐段可读：三法各去掉 1–2 段引注、
   写出一个 `try/finally`、一次 `unlock()`、保存返回值就地声明。

## `preserve_local_scope_plan` 的两条 LK 形对照（显式处置）

上一片把 `ScopePlanCrossing.flatFinally` / `resourceAcrossFinally` 两条负例钉在 `tests/preserve_local_scope_plan.rs`
（`a_crossing_local_keeps_the_dependent_slice_refused`），并注明"re-slice 落地时需显式更新"。
本切片的实测处置：**两条仍逐字拒绝**（上表 `local-scope-crossing` 两侧相同；该测试 5/5 通过、断言未删）。
原因逐条记录：

* `flatFinally`：命名 catch + any 两行 + 自保护行共 3 行的 `try/catch/finally`，且行从 BCI 0 开始——
  不是单 any 行形状；其拒绝（`local 1 crosses a quoted fallback region …` + `@bytecode 0 15 28`）不变。
* `resourceAcrossFinally`：局部句柄 + 构造链（行前 3 次 invoke）+ 自保护行（`[52,54)→52`）——本证书要求
  "行前唯一 invoke"且"单行、无自保护行"且接收者是字段读，三条都不满足；拒绝不变。

因此**未新增**断言、**未删除**任何断言：这两条对照的"显式更新"是"实测确认不动并写明原因"，
证据在本表与本文件，钉住它们的测试文件本身无需改动（改动它反而会掩盖"零回退"的证据）。
