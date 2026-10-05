# Root pre-audit (2026-10-05, before dispatch)

## Diagnosis production points (all in `crates/jarde-java/src/build.rs` render paths)

| Family | Text | Site |
|---|---|---|
| copy | "the copy at BCI {bci} has no proved local assignment" | `build.rs:20501` — `Operation::Duplicate` arm of `render_value`; needs `local_assignments[bci]`, else Err |
| old-value store | "the value at BCI {at} is the value local {slot} held at BCI {bci}…" | `build.rs:20543` area — `Operation::Load { slot }` arm (read refused when slot was overwritten between) |
| dependency chain | "the dependency chain from BCI {anchor} … not bounded" | `build.rs:17880` |

## Structural reading (for Q-i)

These are **per-value render failures**: a value expression that cannot be proven refuses its own render. The quoted `@bytecode` region and the surviving statements are decided **upstream** of `render_value` — at the statement partition (which statements of the body are presented vs quoted). The observed unsoundness is exactly: the failing *value* drags its own statement into the quote, but **semantically-bound sibling statements survive** because they render fine on their own:

- `i = i++`: the `istore` of the old value fails (old-value family) → its statement quoted; the `iinc` renders as `local0 = local0 + 1` and `return local0` survives → compilable wrong.
- `this.flags |= 1<<bit`: the receiver `dup@1` fails (copy family) → the compound assignment statement quoted; method tail (`return;` / `return this.flags;`) survives.
- `elems[size++] = t`: the dup_x1 chain fails (dependency family) → the aastore statement quoted; tail survives.

So the guard belongs at the **statement partition**: when a statement's render failed via any of the three families, the partition must also quote the statements whose **effects read or wrote the same SSA values / slots / fields** that the failed render touched (iinc's target slot; the load feeding the surviving return). The conservative structural rule: quote every statement whose render consumed a value that this run *failed* to prove anywhere (i.e. cross-check survivors' inputs against the failed-value set). Q-ii (existing-refusal texts unchanged) is then automatic: quoting more never changes refusal text of other bodies.

## Feasibility note

`local_assignments`, `instructions` (names record) and the SSA value graph are all available at the partition layer (same file). No new mechanism; this is a tightening of the existing quote-region selection. Worst case (over-quoting) degrades presentation to loud refusal — the safe direction.

## 预审计修正（2026-10-05，实现者发现 + root 独立验证）

**void 方法的"整方法退化到引注"不安全（既有洞）**：explanation-only 的 void 方法体剥离注释后是**空方法体——可编译且静默 no-op**（root 探针 V/V2 独立复现：`flags |= 1<<3` 原 8 vs 引注形 0）。非 void 靠缺 return 语句不可编译；void 无此安全网。**当前主线任何 explanation-only void 方法都在发可编译静默错文本**（先例 BS/CB/TC 恰全非 void，故未被巡查撞见）。BF/BG（void 复合 RMW 锚）的 spec 验收因此必须有呈现修复。

**裁决**：呈现修复属本片范围——零语句 + void 描述符的体加一行代码态拒绝标记 `jarde_refused_body();`（未定义符号→剥离注释编译必败，符号名自文档化；jarde 保留名）。非 void explanation-only 渲染逐字节不变；签名行不动；除此之外不发明文本。守卫本体复用 build.rs ~7520/~7569 的 `stmts.clear()` + 全 BCI fallback 先例（无新结构）。
