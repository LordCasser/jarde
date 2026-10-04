# 1.1 基线重验记录（root 补记要求的开工实测）

时间：2026-10-04（实现者开工时）。基线：worktree HEAD `8cac818e`；巡查基线 `30e54613`。

## 落点重验（以锚点名为准）

六个锚点全部按名核实，行号与 root 记载逐字一致（改动前）：

| 锚点 | 行号 | 核实 |
| --- | --- | --- |
| `fn latch_tested_loop` | 10470 | ✓ |
| `if latches.len() != 1 { return Ok(None) }` | 10483 | ✓ |
| `fn latch_test_chain` | 10133 | ✓ |
| `fn first_latch_test_suffix` | 10389 | ✓ |
| `fn latch_test_suffix_is_effect_free` | 10424 | ✓ |
| `self.latch_test_chain(`（另一 LoopShape 产生点） | 8244 | ✓ |

`git log --oneline 30e54613..HEAD -- crates/jarde-java/src/region.rs` 为空（零改动）；
`git log -S 'latches.len() != 1'` 命中均为 docs 提交。环 1 改过 `emit.rs`、环 2/3 改过
`src/facade.rs`（与 root 记载一致）。

## 原 class 行为重放（`java -Xverify:all`，Corretto 1.8.0_432）

```
S5  → 13/9/20/13        == o5.out   ✓
S3  → [px:3] / [px:3, px:-1, px:5] / [a!, b!]          == o3.out   ✓
S4  → [k:v] / [c!] / [px:3, px:5]                       == o4.out   ✓
Svc → [px:3] / a=1;b=2;hits=1 / true                    == orig.out ✓
```

fixture SHA 核对（`shasum -a 256`）与巡查 `results/fixture-sha256.txt` 逐字一致：
`S3 79880859…`、`S4 1df9631a…`、`S5 bee1eb6c…`（未重编替换）。

## 当前主线（改动前）渲染实测

`S5.mainline-before-diag.json`（`class-source --policy single-class --format json --evidence
region_details` 的完整报告）诊断摘录：

```
jre_region_loop_shape        …header is the block at BCI 4…
jre_region_uncovered_blocks  …3 live block(s)…: [41, 20, 25]
```

- S5：`outerContinueInner`、`labeledOuter` 两形拒绝（与巡查/`results/S5-*.base.json` 同签名）；
  `outerContinueOnly`、`innerOnly` 恢复。
- S3：`nestedBreak`、`nestedNoJump` 拒绝；S4：`nestedPreStored` 拒绝；Svc：`lookup` 整方法拒绝。
- 基线渲染全文见 `../before-after/*.baseline.txt`。

## 与 spec 旧数字的差异

**无差异**。spec 期望输出（S5 `13/9/20/13`、Svc 行为 `[px:3]`）在当前主线重放全部成立，无需以实测
替代任何旧数字；拒绝签名也与巡查逐字一致。
