# 3.3 — the gates, verbatim

Every command below was run in this worktree on the final tree (the implementation, the fixtures, the
tests, the census ledger and the regenerated fingerprint), with the local toolchain (`cargo 1.98.1`,
`javac 23.0.1`, Corretto `1.8.0_432`).

## fmt

```
$ cargo fmt --all -- --check
FMT-OK
```

(The first check failed on four spots of this change's own new code; `cargo fmt --all` was applied and
the check re-run clean.)

## clippy (ci.yml 46–76)

The task's own extraction (`sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh`),
and the CI step's full form, which appends `-D warnings` (the extraction's `grep` drops that line):

```
$ sh /tmp/ci-clippy.sh
    Checking jarde-java v0.1.0 (…)
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 10.30s
$ sh -c "$(cat /tmp/ci-clippy.sh) -D warnings"
    Checking jarde-java v0.1.0 (…)
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.47s
```

Zero warnings and zero errors on the final tree. (The first run carried two warnings from this
change's own new code — `redundant_closure` and `iter_on_map_keys` — both fixed before the final
run.)

## workspace tests — the authoritative totals

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
330
0
```

(`330` is `grep -c 'test result: ok'`; `0` is `grep -c 'test result: FAILED'`; the exit code is the
run's own — never a broad awk.)

Three full runs were made; the two earlier ones carried flakes only, each of them a
scratch-directory or wall-clock artifact of a test that does not read the recovery text this change
touches, and each rerun in isolation (×2, `--all-features`) green:

| run | totals | failure | isolated rerun ×2 |
| --- | --- | --- | --- |
| 1 | 329 ok / 1 FAILED | `p3_short_circuit_transfer_gateway::backward_second_entry_to_gateway_is_refused` — `scratch directory: Os { code: 17, AlreadyExists }` (the test's own `Scratch::new` names its directory by pid + nanoseconds and uses `create_dir`, so two tests of one binary can collide in one nanosecond) | ok, ok; and the whole binary 3×: 6 passed each |
| 2 | 329 ok / 1 FAILED | `d3_artifact_binding::the_evidence_is_rebuilt_after_every_temporary_of_the_first_request_is_dropped` — two `UsageSnapshot`s differing in `elapsed_millis: 1` vs `0` and nothing else (the known wall-clock family) | ok, ok |
| 3 | **330 ok / 0 FAILED** | — | — |

## The change's own suite, and the three precedent suites

```
tests/recover_boolean_int_bitwise_operands.rs        5 passed; 0 failed; 1 ignored
tests/recover_conditional_rhs_field_compound.rs      4 passed; 0 failed; 1 ignored
tests/recover_chained_field_assignment.rs            3 passed; 0 failed; 1 ignored
tests/recover_inline_conditional_concat_operands.rs  4 passed; 0 failed; 1 ignored
```

The ignored tests are the replays; all four were run explicitly and passed:

```
$ cargo test --all-features --locked --test recover_boolean_int_bitwise_operands --test recover_conditional_rhs_field_compound --test recover_chained_field_assignment --test recover_inline_conditional_concat_operands -- --ignored
test the_recovered_text_compiles_and_runs_identically_on_both_legs ... ok   (×4)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 5.14s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 3.43s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 3 filtered out; finished in 1.73s
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 5.26s
```

## The ignored P3 compile-and-execute oracle (corpus-moving discipline)

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.86s
```

**No stale oracle expectation**: the oracle was run after the corpus sweep and needed no assertion
update (nothing in `p3_execution_comparison`'s sample set moved — the sweep's only moved classes are
this change's own anchors and fixtures).

## The corpus differential, the census and the fingerprint

```
$ sh openspec/changes/recover-boolean-int-bitwise-operands/results/03-corpus-sweep.sh
SELF-TEST OK: BW refusals 2 -> 0 with both anchors; BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical
pass A loose candidate classes: 2822
pass A: moved=6 unrendered=1
archive candidate classes: 739
pass C: moved=0 unrendered=1
moved classes: single-class=6 jar=0 total=6
unrendered candidates: A=1 C=1
```

Every one of the six is classified in `results/05-corpus-delta.md`: the patrol's frozen `BW.class`,
the committed copy `patrol-BW.class`, and `BW`/`BWR` on both legs — nothing else in the corpus moved,
in either posture. The two `unrendered` candidates are the same `package-info` entries every
precedent sweep printed, identical on both binaries.

The census and the fingerprint are re-measured per precedent:

```
$ cargo test -p jarde-reader --lib --all-features --locked classfile::tests::repository_class_fixtures
test classfile::tests::repository_class_fixtures_validate_without_false_target_rejections ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 177 filtered out; finished in 0.26s
    # (828, 3596, 290, 2211, 8) -> (835, 3649, 290, 2273, 8), ledger entry added
$ cargo test --test p5_corpus_fingerprint --all-features --locked -- --ignored regenerate_corpus_fingerprint
test regenerate_corpus_fingerprint ... ok        # +10 files (3 sources, 7 class files)
$ cargo test --test p5_corpus_fingerprint --all-features --locked
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

The census delta is this change's own seven classes and fifty-three bodies: `BW`/`BWR`/`BWN` on both
legs (forty-four bodies) plus the frozen `patrol-BW.class` copy (nine), with sixty-two branch targets
(`BW` six, `BWR` eight, `BWN` fourteen per leg, plus the copy's six) and no handler record or
subroutine. The fingerprint diff is exactly ten added files, all under
`tests/fixtures/recover-boolean-int-bitwise-operands/`, and nothing else.

## `git diff --check`

```
$ git diff --check
DIFF-CHECK-OK
```

## openspec validate

```
$ openspec validate --all --strict
Totals: 307 passed, 0 failed (307 items)
```

## Not run, with the reason

* `cargo test --test jvm_bytecode_oracle --locked -- --ignored --exact jdk25_instruction_boundaries_match_public_bytecode_inspection`
  — the local JDK is 23.0.1 and the oracle compiles against JDK 25's preview `java.lang.classfile`
  API, so `javac` refuses the fixture. This is the environment ceiling the precedent records, not a
  result about this change.

## Commits (no push)

| commit | contents |
| --- | --- |
| `feat(java): read the int-ified boolean operand of a bitwise expression back as boolean` | `crates/jarde-java/src/build.rs` |
| `test(boolean-int-bitwise): freeze the patrol anchor, the positives and the negatives on both legs` | `tests/fixtures/recover-boolean-int-bitwise-operands/`, `tests/recover_boolean_int_bitwise_operands.rs` |
| `test(census): re-measure the fixture population and re-render the corpus fingerprint` | `crates/jarde-reader/src/classfile.rs`, `tests/fixtures/corpus-fingerprint.json` |
| `docs(change): the gating transcript, the class-level resolution, the corpus differential and the gates` | `openspec/changes/recover-boolean-int-bitwise-operands/` |
