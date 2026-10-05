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
