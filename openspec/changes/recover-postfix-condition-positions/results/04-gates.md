# Task 3.1 — the gates, verbatim

Every command below was run in this worktree on the final tree (`4f4ffa34` + this change), with the
local toolchain (`cargo 1.98.1`, `javac 23.0.1`, Corretto `1.8.0_432`).

## fmt

```
$ cargo fmt --all -- --check
FMT-OK
```

## clippy (ci.yml 46–76)

The task's own extraction (`sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' '`),
and the CI step's full form, which appends `-D warnings` (the extraction's `grep` drops that line):

```
$ sh /tmp/ci-clippy.sh
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 20.73s
$ sh -c "$(cat /tmp/ci-clippy.sh) -D warnings"
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 27.48s
```

## workspace tests — the authoritative totals

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
ok=329
failed=0
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

(`ok` is `grep -c 'test result: ok'`; `failed` is `grep -c 'test result: FAILED'`; the exit code is
the run's own. No flake family fired, so no single-test rerun was needed.)

## openspec validate

```
$ openspec validate --all --strict
…
Totals: 307 passed, 0 failed (307 items)
```

## The three frozen suites, and their replays

```
$ cargo test --test recover_postfix_condition_positions --test recover_postfix_old_value_snapshot --test recover_dup_store_conditional --all-features --locked
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

$ … -- --ignored
test the_recovered_text_compiles_and_runs_identically_on_both_legs ... ok   (×3)
```

## The ignored P3 compile-and-execute oracle (corpus-moving discipline)

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.11s
```

**No stale oracle expectation**: the oracle was run after the corpus sweep and needed no assertion
update (nothing in `p3_execution_comparison`'s sample set moved).

## The corpus differential and the census

```
$ sh openspec/changes/recover-postfix-condition-positions/results/03-corpus-sweep.sh
SELF-TEST OK: CP7 condition positions 0 -> 3; dup-store NEG byte-identical; 3 A-phase trap refusals present; bs.jar!BS.class byte-identical
pass A: moved=7 unrendered=1
pass C: moved=3 unrendered=1
moved classes: single-class=7 jar=3 total=10
```

Every one of the ten is classified in `03-corpus-delta.md`; the two `unrendered` candidates are
`package-info` entries, identical on both binaries. The two outgrowths (`NL.findMid`, `SW.sum2d`)
were verified behaviorally on both compiler legs (`results/behavior.sh`, `results/03-corpus-delta.md`).

The census and the fingerprint are re-measured per precedent:

```
$ cargo test -p jarde-reader --lib --all-features --locked classfile::tests::repository_class_fixtures
test result: ok. 1 passed; …                     # (828, 3596, 290, 2211, 8), ledger entry added
$ cargo test --test p5_corpus_fingerprint --all-features --locked -- --ignored regenerate_corpus_fingerprint
test regenerate_corpus_fingerprint ... ok        # +9 files (3 sources, sha256.txt, 6 class files)
$ cargo test --test p5_corpus_fingerprint --all-features --locked
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

## Not run, with the reason

* `cargo test --test jvm_bytecode_oracle --locked -- --ignored --exact jdk25_instruction_boundaries_match_public_bytecode_inspection`
  — the local JDK is 23.0.1 and the oracle compiles against JDK 25's preview `java.lang.classfile`
  API, so `javac` refuses the fixture (`ClassFile 是预览 API …`). This is an environment ceiling, not
  a result about this change: the failure is a preview-API refusal before any fixture is read, and
  the step belongs to CI's JDK 25 job.

## Commits (no push)

| commit | contents |
| --- | --- |
| `feat(java): present the postfix old value in loop and branch condition tests` | `crates/jarde-java/src/{region.rs,build.rs}` |
| `test(postfix-conditions): freeze the patrol trio, the negatives and both legs` | `tests/fixtures/recover-postfix-condition-positions/`, `tests/recover_postfix_condition_positions.rs`, the A-phase suite's `condShape` expectation + fixture README, `tests/p3_loop_test_values.rs` |
| `test(census): re-measure the fixture population and the corpus fingerprint` | `crates/jarde-reader/src/classfile.rs`, `tests/fixtures/corpus-fingerprint.json` |
| `docs(change): the gating transcript, the corpus differential and the gates` | `openspec/changes/recover-postfix-condition-positions/` |
