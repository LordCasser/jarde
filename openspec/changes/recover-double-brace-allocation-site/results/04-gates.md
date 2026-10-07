# Task 3.1 — the gates, verbatim

Every command below was run in this worktree. The authoritative reading of a test run is the exit
code plus `grep -c 'test result: ok'` plus **zero** `test result: FAILED` lines — never a broad awk
over the log.

## Formatting and whitespace

```text
$ cargo fmt --all -- --check
FMT-EXIT=0
$ git diff --check
DIFF-CHECK-EXIT=0
```

Rerun after the last edit of this change (`06-fmt-clippy.txt`).

## Clippy (CI's own allowlist, `.github/workflows/ci.yml` 46–76)

```text
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ sh /tmp/ci-clippy.sh
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 6.52s
CLIPPY-EXIT=0
```

Zero warnings (`grep -c warning` over the run: 0). The first run of this gate caught two unit-test
call sites in `src/member_inner.rs` that the new `CaptureReadReceiver` parameter had to reach
(they pass `EntryThis`, the reading they were written under); they were fixed before this record.

## The workspace test suite (authoritative)

```text
$ CARGO_INCREMENTAL=0 cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
```

| reading | value |
| --- | --- |
| exit code | **`0`** (`EXIT=0`, recorded in `gate-tests.log`) |
| `test result: ok` lines | **338** |
| `test result: FAILED` lines | **0** |
| flake reruns needed | none |

The baseline (HEAD `9c9fdd13`) reads 337 targets; this slice adds its own suite
(`tests/recover_double_brace_allocation_site.rs`, four tests), which is the 338th.

## The oracle leg (ignored, corpus-moving discipline)

```text
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
running 3 tests
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.77s
```

**3/3 green, no stale expectation to update**: the P3 corpus (`tests/fixtures/p3-corpus/`,
`Flags.java`) declares no anonymous companion, so no body this change presents is one the oracle
replays. The verbatim log is `05-oracle-leg.txt`.

## The order-sensitive controls (non-negotiable)

```text
$ cargo test --locked --all-features --test ctor_reorder_dispatch_guard --test fixture_behavior_guards --test p3_ordinary_new_invokes
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.46s
test result: ok. 7 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.03s
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.01s
$ cargo test --locked --all-features --test fixture_behavior_guards -- --ignored
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 7 filtered out; finished in 0.78s
```

The `anonymous-super-dispatch` fixture (the superclass constructor that virtually dispatches on
`this`) is the `ctor_reorder_dispatch_guard` anchor and stays byte-identical: its child declares the
overridden method, so the double-brace admission refuses it by its second criterion before any
order question is asked — and the guard's own recompiled-run comparison is unchanged.

## The A suites, the anonymous family, and this change's own suite

```text
double_brace_capture:                test result: ok. 4 passed; 0 failed; 0 ignored; …
recover_synthetic_ctor_super_order:  test result: ok. 1 passed; 0 failed; 0 ignored; …
recover_double_brace_allocation_site: test result: ok. 4 passed; 0 failed; 0 ignored; …
anonymous_parameterized_root:        test result: ok. 4 passed; 0 failed; 0 ignored; …
anonymous_supertype_return:          test result: ok. 6 passed; 0 failed; 0 ignored; …
p3_anonymous_class_facts:            test result: ok. 1 passed; 0 failed; 0 ignored; …
```

`tests/double_brace_capture.rs` carries the **semantic update** this change owes: the host-form
assertion read `return new DB$2(s);`/`DB.dbl = new DB$1();` and now reads the double-brace form at
both anchors, with every companion-side assertion (the capture order, the synthetic declaration,
the queryable physical child, the two probes) unchanged — an assertion update, not a deletion. The
`anonymous_superclass_*` suites in `tests/class_source.rs` are part of the 338 targets and stay
green.

## The census and the fingerprint

```text
$ python3 results/corpus-fingerprint.py results/fingerprint.txt
899 class file(s) fingerprinted into …/fingerprint.txt
$ diff fingerprint-previous.txt fingerprint.txt | grep -c "^[<>]"
20
```

**Twenty added lines, zero changed, zero removed**: the twenty new fixture class files. Every
pre-existing fixture's render digest is byte-identical to the previous slice's manifest. The
manifest renders each file through `--policy single-class`, which holds no companion — that view
cannot exercise this change at all (the control's own line reads `…:exit4`, the path-A refusal, on
both legs), so the corpus scans below are the reading that does.

```text
$ cargo test --locked --all-features --test p5_corpus_fingerprint
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.09s
$ cargo test --locked --all-features --test p5_corpus_fingerprint -- --ignored regenerate_corpus_fingerprint
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.14s
```

`tests/fixtures/corpus-fingerprint.json` gains 105 lines (the new fixture entries and their
carriers) and rewrites none (`git diff --stat`: `1 file changed, 105 insertions(+)`).

The reader crate's own fixture census
(`crates/jarde-reader/src/classfile.rs::repository_class_fixtures_validate_without_false_target_rejections`)
moved from `(879, 3829, 334, 2445, 8)` to `(899, 3867, 334, 2445, 8)`: **+20 classes, +38 bodies**,
no handler record, no branch target, no subroutine — the ten classes and nineteen bodies per leg of
the new fixture set. The tuple and its paragraph were updated in the same commit.

## The corpus scans (three two-leg sweeps, every delta classified)

`02-corpus-scans.txt` records all three; `03-stderr-classification.txt` the stderr reading:

| scan | captures per leg | render (`.txt`) deltas |
| --- | --- | --- |
| `scan-corpus.sh` — fixtures, `--policy single-class` | 899 | **0** (this view holds no companion) |
| `scan-corpus-jars.sh` — every fixture **directory** as one jar | 899 | **4**: the patrol pair's `DB` and the new control's `DBS` (both legs) |
| `scan-evidence.sh` — evidence jars and loose classes | 2725 | **1**: the patrol's `db.jar::DB.class` |

The anchor delta is `01-evidence-anchor-delta.txt` (both anchors, and the exit status 4 → 0). The
stderr deltas beyond `elapsed_millis` are `ir_items`-only (the retention's own `weight = 2` charge)
and probe work (a class the admission read and did not claim: the child read, and the child's
preparation when the shape gates passed), with **no render change** — classified in
`03-stderr-classification.txt`.
