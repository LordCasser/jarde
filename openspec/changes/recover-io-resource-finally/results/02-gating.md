# 门控实验（基线 = 父提交 `c99e26a5`；本片 = 当前 worktree）

命令：`sh results/02-gating.sh`（自测：每个渲染先断言 `// jarde: presentation of` 自头部；任一渲染
无头部即停）。基线二进制 = 父提交 `c99e26a5` 的独立 worktree 构建
（`/tmp/io-baseline-target/debug/jarde-cli`），本片二进制 = 本 worktree 的
`target/debug/jarde-cli`。逐输入渲染正文存于 `gating/`。

## 结果（`02-gating.out`）

| 输入 | 判定 | 说明 |
| --- | --- | --- |
| `io-patrol-jar`（锚，巡查冻结件） | **moved** | `countLines` 由整方法拒 → 完整 `try { … } finally { r.close(); }` |
| `io-v8` / `io-v8-javac8` | **moved** | 与 jar 摘要**相同**（三腿逐字节一致：`27d2b672…`） |
| `mid-v8` / `mid-v8-javac8` | **moved** | `IOMidRead.countRemaining`（同形，caller-owned 流） |
| `negatives-v8` / `negatives-v8-javac8` | identical | 多资源嵌套 try、close 带返回值形：拒绝逐字 |
| `depth-v8` / `depth-v8-javac8` | **moved** | `NestedDepth.threeLayer` 呈现为单个 `new` 表达式；`fourLayer` 仍拒 |
| `widening-v8` / `widening-v8-javac8` | **moved** | 两个 `java.io` 实参位各自带自身表行的 cast |
| `lk-patrol-jar` / `lk-v8` / `lk-v8-javac8` | identical | **LK 三法逐字节不变**（三腿摘要同为 `79b61c4d…`） |
| `lk-negatives-v8` / `lk-negatives-v8-javac8` | identical | 四个锁卫负例逐字 |
| `lk-probe-v8` / `lk-probe-v8-javac8` | **moved** | io-wrapping 单方法探针 = 锚同形，按登记翻转（显式更新于 `tests/recover_lock_guard_loop_finally.rs`） |
| `local-scope-crossing`（双腿） | **moved** | `resourceAcrossFinally` = 锚同形（显式更新于 `tests/preserve_local_scope_plan.rs`）；`flatFinally` 拒绝不变 |
| `p3-handlers`（`Guarded`，真两份副本 finally） | identical | 两副本非锁形呈现/拒绝逐字 |
| `void-loop-fixed`（CF-16 固定形） | identical | 固定证书形状未被本证书夺取（完成形非 `SavedReturn`） |
| `nested-ctor-x4` | **moved** | `threeLayer` 由拒转呈现（深度 2→3 的正面） |
| `nested-ctor-x3` | identical | 二层/双嵌套/同型两次等既有正面逐字 |

## 结论

1. **证书单独翻转同形**：`IOMidRead.countRemaining`（无构造链、无深度需求）在只保留证书的构建
   里也翻转（[03-component-gating.md](03-component-gating.md)），说明行集形状本身是被证书承认的，
   而非被宽化表或深度顺带带过。
2. **零回退**：LK 三腿、LK 四负例、`Guarded`、CF-16 固定形、`X3` 全部逐字节不变。
3. **显式更新的三处对照**（未删除任何断言）：
   * `tests/recover_lock_guard_loop_finally.rs` 的 `LockGuardProbe` 探针（改判为"按证书呈现"）；
   * `tests/preserve_local_scope_plan.rs` 的 `resourceAcrossFinally`（改判为"整段呈现"，同测试内
     `flatFinally` 保持拒绝）；
   * `crates/jarde-java/src/init.rs` 的 `threeLayer` 深度负例（改判为新终态，并新增 `fourLayer`
     越界核查）；`crates/jarde-java/tests/nested_ctor_argument_sites.rs` 的 `threeLayer` 正面。
4. **登记边界不动**：`IO.readAll`、多资源/带返回值负例、固定 void-loop 形、LK 三法。
