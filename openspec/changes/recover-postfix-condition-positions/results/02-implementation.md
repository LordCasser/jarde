# Task 2.1/2.2 — what changed, and where

The mechanism is Phase A's snapshot presentation, reaching the **condition positions** a loop's or a
branch's test reads. Four pieces, in two files. Nothing else in the corpus moved (see
`03-corpus-delta.md`).

## `crates/jarde-java/src/region.rs` — the crossing gate's paired exemption

* **`test_expression_instruction`** (new) is the one place that states what may stay in a test block:
  the value-producing instructions, a call or field read the branch consumes, a **throwing read** —
  an array length, and now an **array element** (`xs[i++]`, the trio's own shape) — under the same
  single-reader discipline the array length already had, the copy-and-store dance of
  `recover-dup-store-conditional`, and the **`iload slot; iinc slot, ±1` pair**. `test_is_pure` and
  `latch_test_suffix_is_effect_free` both read it, so a value the test writes has exactly one place
  where it is admitted.
* **`snapshot_condition_part`** (new) is the pair's own proof, mirroring the build's: the increment
  reads the value its own load read; the old value has exactly one consumer, in the same block,
  after the update, and that consumer is part of the condition (or the branch itself); nothing reads
  the updated value in between; the consumer is not a store back into the slot (`i = i++` keeps its
  refusal). Both halves are required together, the `unobservable_store_dance_part` precedent's shape.
* **`latch_test_suffix_is_effect_free`** no longer duplicates its own opcode lists (and no longer
  requires the block's *lead* to be increments): the lead is the body's own statements, which the
  build writes (`recover-postfix-condition-positions`: `last = xs[i];` shares the do-while's block
  with the `xs[i++] != 0` its condition starts at).
* **`FallbackReason::ChainPositionBound`** (`jre_region_chain_position_bound`) is the slice's own
  bound, stated by the two chain rules (`header_test_chain`, `latch_test_chain`): one postfix
  position per chain, at one of its ends. A chain that would present a second position or a middle
  one is refused with that reason, so the deferred shapes keep the refusal they had and the
  diagnostics name *why*.

## `crates/jarde-java/src/build.rs` — the two build-side pieces the anchors needed

* **The outer test block's lead** (`build_two_exit_return` + the `Region::TwoExitReturn` arm): the
  instructions before the first one the outer test's condition reads are the region's own prologue
  (`int i = 0;` sharing the block with the `if`), and `test_effects(outer, at)` writes them as the
  statements they are, in front of the return — the same treatment the short-circuit value's outer
  branch already takes. This is a **pre-existing gap**, not a postfix one: `int i = 0; if (a[i] > 0
  && i < a.length) return true; return false;` refuses the same way at the parent commit. Only the
  outer block's lead is admitted; an inner test's lead runs on one path and keeps the refusal.
* **A cross-block read of the pre-update value** (`prove_local_snapshots`): Phase A admitted only
  same-block reads before the increment. `find`'s first test reads the slot where the text still
  holds the pre-update value, in the block that transfers directly to the increment's block — the
  earlier test of the loop's condition chain — and `consumer_block_precedes` admits exactly that
  bounded form. Every other cross-block read keeps the refusal (measured: a cross-block read with no
  such predecessor keeps the time-annotation refusal, fail-closed).

## Not changed

* The A-phase proof's refusals: `i = i++`/`i = i--`, the field self-assignment and the multi-consumer
  form are refused **verbatim** (the A-phase suite and the new suite both pin them, and the sweep
  shows `NG`'s four traps byte-identical).
* The dup-store sibling: `recover_dup_store_conditional` is green and its `NEG.class` is
  byte-identical in the sweep; `while (read() != null)` keeps its eliminated form.
* `Region::SharedTailEarlyReturn`'s own test-block check: untouched (the patrol's `if` shape is a
  `TwoExitReturn`; the shared-tail family keeps its parent-commit behavior).

## Per-task evidence

| task | evidence |
| --- | --- |
| 1.1 | `results/01-gating.md` (the probe transcript per shape, the two experiments), `results/01-probes.patch` |
| 1.2 | `tests/fixtures/recover-postfix-condition-positions/` (README, sha256.txt, both legs), `tests/recover_postfix_condition_positions.rs`'s negative test |
| 2.1 | the two source files' diffs; the region detail `jre_region_chain_position_bound` on both negatives |
| 2.2 | `tests/recover_postfix_condition_positions.rs` (4 + 1 ignored, the replay green on both legs), `tests/recover_postfix_old_value_snapshot.rs` (4 + 1 green, the `condShape` expectation updated), `tests/recover_dup_store_conditional.rs` (3 + 1 green), `tests/p3_loop_test_values.rs` (the two stale refusals updated to the presented form) |
| 3.1 | `results/03-corpus-delta.md`, `results/04-gates.md` |
