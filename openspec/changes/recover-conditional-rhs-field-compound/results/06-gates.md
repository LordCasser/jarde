# 3.1 — the gates, verbatim

Every command below ran in this change's own worktree, over the final file state (the last edit
before them is the removal of a redundant `#[allow]`), with the disk checked first (`df -h /`).

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
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.67s
```

Zero warnings and zero errors (`grep -cE "^warning|^error"` over the run: `0`).

## `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`

Authoritative totals of the final run — exit code, the count of `test result: ok` lines, and the
count of `test result: FAILED` lines (never a broad awk):

```
EXIT=0
327
0
```

Three full runs were made while the change was being finished; the two earlier ones carried flakes
only, each of them a scratch-directory or wall-clock artifact of a test that does not read the
recovery text this change touches, and each rerun in isolation (×2, `--all-features`) green:

| run | totals | failure | isolated rerun ×2 |
| --- | --- | --- | --- |
| 1 | 326 ok / 1 FAILED | `p4_plugins::the_plugin_plane_leaves_the_structural_planes_own_answer_untouched` — the JSON documents differ in `"elapsed_millis": 0` vs `1` and nothing else (the known `p4_plugins` timing family) | ok, ok |
| 2 | 325 ok / 2 FAILED | `class_source::anonymous_inner_this_refuses_a_cross_class_constructor_reference` — `javac` could not find the scratch `Other.java`; `p3_two_exit_return::observable_equals_calls_keep_their_lazy_order_and_count` — `scratch directory: Os { code: 17, AlreadyExists }` (a timestamp collision with run 1's own scratch tree) | ok, ok each |
| 3 | **327 ok / 0 FAILED** | — | — |

## The change's own suite, and the two precedent suites

```
tests/recover_chained_field_assignment.rs            3 passed; 0 failed; 1 ignored
tests/recover_conditional_rhs_field_compound.rs      4 passed; 0 failed; 1 ignored
tests/recover_inline_conditional_concat_operands.rs  4 passed; 0 failed; 1 ignored
```

The ignored tests are the replays; this change's own was run explicitly and passed:

```
cargo test --locked --test recover_conditional_rhs_field_compound -- --ignored
test the_recovered_text_compiles_and_runs_identically_on_both_legs ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out
```

## The oracle leg (corpus-moving discipline)

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 46.70s
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
  `(805, 3405, 288, 2095, 8)` → `(811, 3445, 290, 2145, 8)`: +6 classes, +40 bodies, +2 handler
  records (`RCN.inTry`'s catch, one per leg) and +50 branch targets (`BI.earlyRet`'s five, `RC`'s
  eleven and `RCN`'s nine, per leg). A re-measure with its own comment paragraph, not a relaxation.
* `tests/fixtures/corpus-fingerprint.json` regenerated: **+50 lines, 0 removed** — the ten new files
  (three sources, the frozen `bi.jar` and six class files; the README is prose and the manifest
  skips `.md`), five lines each; no existing entry moved.
