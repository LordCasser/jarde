# Computed-for task 1.1: same-IR diagnosis summary

The read-only verifier in this directory passes against the actual root-v2 execution record and captured raw streams. It also runs the existing v9 typed SSA verifier against its own frozen v3 evidence, then applies its typed def/use comparators to root-v2's combined raw. Root-v2 recorded one exact unit test passed, restored all 132 pre-run source pins, completed `cargo clean`, and recorded `target_absent=true`. The verifier independently hashes the current 132 pinned source files and confirms they still match the restored pins.

The single frozen method is `lastIndexOf([IIII)I` from the 419-byte `LoopCases.class` (class SHA-256 `004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33`; BLAKE3 `52ddcc0b5ab5de0702266901380266b53498433df87ca02f4861c61c7f4c9b19`). The root-v2 test's diagnostics all use the same reader-produced IR, canonical graph, SSA, decoded operations and recovered region.

The preheader sequence is BCI 0 `iload_3`, BCI 1 `iconst_1`, BCI 2 `isub`, BCI 3 `istore 4`. SSA says BCI 0 reads entry `Local(3)` (`ValueId(3)`) and writes stack `ValueId(9)`; BCI 1 writes `ValueId(10)`; BCI 2 reads 10 and 9 and writes `ValueId(11)`; BCI 3 reads 11 and writes `Local(4)` as a distinct `ValueId(12)`. `ValueId(11)` is instruction-defined at BCI 2 and has exactly the BCI 3 Store use. `ValueId(12)` is the preheader value entering the header phi, not the stack-input value. All these values have the recorded `Int` frame type.

Header BCI 5 has local-4 phi `ValueId(8)` with ordered inputs 12 and 21. Its instruction uses are at BCIs 5, 12, 19 and 22. The complete local-4 access set is Store BCI 3; reads BCIs 5, 12, 19 and 22; and the update write at BCI 22. The natural loop is nodes `{1,2,5}` / BCIs `{5,11,22}`, latch node 5 / BCI 22, reducible. Under the existing incoming `Frame::default()`, the terminal-return leaf is node 4 / BCI 19, whose BCI 21 is `ireturn`; its sole normal predecessor is BCI 11 and it has no outgoing edge. Update BCI 22 reads phi 8 and writes 21; goto BCI 25 follows it. Canonical update/latch edges are `11 -> 22` and `22 -> 5`.

The real Builder output reports `init_store_bci=Some(3)`, `statements=7`, `ragged=false`. The two Builder statements carrying direct origin BCI 3 are, in order: `Declare { ty: Int, name: "local4", value: None }`, then `Assign { name: "local4", value: arg3 - 1 }`. The local expression reads `arg3` from BCI 0, literal `1` from BCI 1, and subtract expression from BCI 2. Builder state is `deferred=[]`, `plans={}`, `bindings={}`, `rejected={}`, `binding_refused={}`. The output contains `While`, not `For`.

Static source cross-check against the restored [build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:18724): For-header initialization selection uses `rposition` for the proof's `init_bci`, accepts only `Declare { value: Some(_) }` or `Assign`, and requires every later statement before the loop to be an empty declaration. Here the exact BCI-3 statement was materialized as an empty declaration plus an assignment, so current Builder does not select it as a For initializer. The later Builder code separately constructs the update from `proof.update_bci`, requires one assignment and matching local name, then removes only the selected init statement. This is evidence about current consumption behavior, not proof that a computed-for implementation is complete.

This evidence is sufficient for root to accept/close diagnostic task 1.1 if it agrees. It does not implement or verify task 2.x: no For recovery, effect-ordering acceptance, return-leaf For admission, tail-source change, permanent regression, or full product/CLI/JDK acceptance is established. Do not report this current `while`/assignment output as a For result.

The separate root-v1 failed compile remains preserved at `/private/tmp/jarde-computed-for-builder-diagnostic-root-v1/0.stderr.raw`; its two errors were the wrong `ArrayInitializers` module path and a `Slot::Local` supplied where `Operation::Store.slot` is `u16`. The v3 patch corrected both; root-v2 is the actual passing run.

Exact test argv recorded by root-v2:

```text
cargo test -p jarde-java --lib --locked region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir -- --exact --nocapture
```

The pure-Python verification command used for this audit is:

```text
python3 /private/tmp/jarde-computed-for-builder-diagnostic-audit-luna-v1/verify_root_v2.py
```

It invokes the existing v9 Python verifier via `runpy`, then validates root-v2's execution/raw/pin bindings and Builder diagnostic. It invokes no Git, Cargo, JDK, or CLI commands.
