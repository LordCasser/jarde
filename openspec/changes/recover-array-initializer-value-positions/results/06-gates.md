# 3.3 — the gates, verbatim

Every command below ran in this change's own worktree, over the final file state, with the disk
checked first (`df -h /`: 48 Gi available, no `cargo clean` needed).

## `cargo fmt --all --check`

```
FMT CLEAN
```

## clippy — the CI's own command (`sed -n '46,76p' .github/workflows/ci.yml`)

```sh
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
sh /tmp/ci-clippy.sh
```

```
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.76s
CLIPPY WARNINGS/ERRORS: 0
```

## `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`

Authoritative totals of the run — exit code, the count of `test result: ok` lines, and the count of
`test result: FAILED` lines (never a broad awk):

```
TESTS EXIT=0
ok_lines=328
failed_lines=0
```

328 is the parent commit's 327 plus this change's own new test target
(`tests/recover_array_initializer_value_positions.rs`). No known flake family
(`export_cli` timing, `ordinary_generic_projection`, `bulk_recovery_*`, `p4_plugins`, `class_source`
scratch) failed in this run, so no single-test rerun was needed.

## The change's own suite, and the suites the corpus delta touches

```
tests/recover_array_initializer_value_positions.rs   4 passed; 0 failed; 1 ignored
tests/recover_loop_else_if_early_returns.rs          4 passed; 0 failed; 1 ignored
```

Both ignored tests are the replays; both were run explicitly and passed:

```
cargo test --locked --test recover_array_initializer_value_positions -- --ignored
test the_recovered_text_compiles_and_runs_identically_on_both_legs ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 4.64s

cargo test --locked --test recover_loop_else_if_early_returns -- --ignored
test every_stripped_anchor_answers_what_its_class_answers ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 5.65s
```

## The oracle leg (corpus-moving discipline)

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.29s
```

No oracle expectation is stale under this change, so none was updated.

## `openspec validate --all --strict`

```
Totals: 306 passed, 0 failed (306 items)
```

## `git diff --check`

```
DIFF CHECK CLEAN
```

## The corpus census and fingerprint (the same commit's chore)

* the fixture-population census in `crates/jarde-reader/src/classfile.rs` re-measured
  `(811, 3445, 290, 2145, 8)` → `(820, 3536, 290, 2161, 8)`: +9 classes (the four dual-leg fixtures
  `MD`/`MD2`/`MD3`/`AV` on two legs, plus the hand-built `AVN`), +91 bodies (`MD` 7, `MD2` 7, `MD3` 5
  and `AV` 25 per leg, `AVN` 3), and +16 branch targets (`MD.sumJag`'s two loops and `AV`'s
  `foreach` loop and `condIdx` comparison, per leg). No handler record and no subroutine was added.
  A re-measure with its own comment paragraph, not a relaxation.
* `tests/fixtures/corpus-fingerprint.json` regenerated: **+13 entries, 0 removed, 0 changed** (65
  lines) — the four sources, the hand-built `AVN.class` and the eight leg class files; `README.md`
  and `build_avn.py` are excluded by the manifest's own rules (prose and the seed generators).
  No existing entry moved.

## What was verified beyond the gates

* the dual-leg replay (`results/dual-leg-replay.sh`) and the corpus-delta replay
  (`results/04-corpus-delta-replay.sh`), both on `javac --release 8` **and** real javac 8, under
  `-Xverify:all`, against the fixtures' own class files;
* the corpus sweep (`results/03-corpus-sweep.sh`): 22 moved classes, every one of them replayed
  (see `results/05-corpus-delta.md`);
* the hand-built negatives' refusals, byte for byte, and the quote that accounts for the refused
  dance's instructions (asserted in `the_hand_built_negatives_keep_their_refusals`).
