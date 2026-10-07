# 1.1 两形拒绝链的插桩复核（门控实验的基线）

工作树 = `3e45800a`（多锁巡查提交）。所有测量用同一 checkout 构建的 `jarde-cli`（debug，
`--locked`），输入 = 巡查冻结件 `ml.jar` 与本次 fixture 的两条重编腿（javac 23.0.1
`--release 8 -g:none` / 真实 javac 8 Corretto 1.8.0_432 `-g:none`）。

## 基线复现（三条腿逐字一致）

```
// BCI 41: the exceptional path repeats code the normal path also runs — the `finally` copy javac emits for a `finally` clause; this candidate lacks the complete straight-body, copy, range, and ownership proof needed to merge them into one `finally`
// 2 live block(s) are reachable only through edges the normal-flow view leaves out: [58, 41]      (nestedLocks)
// BCI 27: …  // 2 live block(s) are reachable only through edges the normal-flow view leaves out: [37, 27]   (interruptibly)
```

与巡查 `results/jarde-ML.txt` 逐字一致（BCI 41 / BCI 27，四件套句）。**同时实测：**
`multiAwait` 在本工作树的三条腿上**仍被拒**（`jre_region_exception_edge` @BCI 15 +
`5 live block(s) … [22, 31, 70, 40, 46]`），与巡查 README 的"multiAwait 完整恢复（0 引注）"
不符——巡查该行是**未复现的登记**，见 `04-corpus-and-gates.md` 的账本更正。本片把 multiAwait
按"逐字节不变的控制项"钉住（baseline == current），不声称其已恢复。

## 插桩（临时 `eprintln!`，测量后 `git checkout` 还原）

在 `prove_finally_copy`/`prove_lock_guard_finally` 的每一处 `return Ok(None)` 前打点
（`line!()` 标签），并在 `cleanup_sequence` 的 `calls > 1` 拒绝处与
`shared_finally_candidate` 的锁卫探针处打点；另有临时单测打印 canonical 块、行表与
`shared_finally_candidate(block 0)` 的答案。测量（`v8` 腿，两形同构）：

```
=== method nestedLocks rows:
  row 0: [14..24) -> 41 type None
  block 0: [0, 1, 4, 7, 8, 11, 14, 15, 16, 19, 20, 21, 24, 25, 28, 31, 32, 35, 38]
  block 58: [58]
  block 41: [41, 42, 43, 46, 49, 50, 53, 56, 57]
  finally_copy handler entry = Some(41)
TRACE FC at line 2931                      <- prove_finally_copy 的"行前调用"拒绝
  prove_finally_copy = Ok(false)
TRACE LG at line 3685                      <- prove_lock_guard_finally 的第一处检查
  prove_lock_guard_finally(block 0) = Ok(false)
  shared_finally_candidate(block 0) = Ok(false)
=== method interruptibly rows:
  row 0: [7..17) -> 27 type None
  block 0: [0, 1, 4, 7, 8, 9, 12, 13, 14, 17, 18, 21, 24]
  block 37: [37]
  block 27: [27, 28, 29, 32, 35, 36]
  finally_copy handler entry = Some(27)
TRACE FC at line 2931
  prove_finally_copy = Ok(false)
TRACE LG at line 3685
  prove_lock_guard_finally(block 0) = Ok(false)
  shared_finally_candidate(block 0) = Ok(false)
```

## 两处拒绝点（按名重验后的行号）

| # | 位置 | 判据 | 两形实测 |
| --- | --- | --- | --- |
| **R1** | `guard.rs::prove_lock_guard_finally` 的副本文法（`lock_guard_copy`：恰好 `load; field; call` 三条，`:3593`）+ 行前调用数（`let [acquire] = acquisitions` 恰好一个，`:3696`）；同族还有共享四件套的 `cleanup_sequence` `calls > 1`（`:2754`） | finally 体只证**单调用** | nestedLocks 两个获取/两组释放：cheap gate（`:10846` 的 `count == 1`）先拒，证明本身也拒 |
| **R2** | `guard.rs::prove_finally_copy` 的行前调用拒绝（`:2925-2929`，注释明文"This slice does not infer a `try` boundary after a call that may throw"）+ `prove_lock_guard_finally` 的 `row.start_bci != current.bci()`（`:3678`） | 行前可抛调用即拒；且保护区必须起于当前块 | interruptibly 的行前 `lockInterruptibly` @BCI 4 → 四件套拒（BCI 27）；两形的行起点都在**块内**（canonical 不在行界切块），故锁卫证书在 `current.bci()==0` 处第一检查即拒 |

补充实测（两形共同的结构事实）：region 走查**确实**在 block 0 询问了
`shared_finally_candidate`（`starts_catch` 为真），是 cheap gate 与证明本身把它挡在门外——
所以两个准入都必须落在**锁卫证书**上（`prove_finally_copy` 的完成形只证 `Return`，两形的
正常完成是 `goto`/transfer，故四件套路径即使放宽行前调用也到不了）。

## 门控设计（每准入单独可翻）

- **A（多语句 finally 体）**：副本序列（≤2 组，每组 `load; field; call`）+ SSA 配对 +
  释放序=获取逆序；它自带"保护区起于块内"的放宽（nestedLocks 的行起点 14 在 block 0 内）。
- **B（行外可抛获取）**：单组副本路径放宽到"获取调用在全部异常行范围之外"（行集自证顺序：
  调用完成才入保护区；抛出时 unlock 不跑），含保护区起于块内的同一放宽。

单独门控由此成立：A-only 构型里单组路径仍要求 `row.start_bci == current.bci()` →
interruptibly 不翻；B-only 构型里副本文法仍是单组 → nestedLocks 不翻。矩阵见
`03-gating.md`。
