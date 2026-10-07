# Task 3.1 — the gates, verbatim

Every gate below was run in this worktree after the last edit (the census expectation's update is
the last source edit; `fmt`/`clippy`/the workspace run were all repeated after it).

## `cargo fmt --all -- --check`

```
(no output; exit 0)
```

## Clippy — exactly CI's flags (`.github/workflows/ci.yml` lines 46–76)

```
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
sh /tmp/ci-clippy.sh
    Checking jarde-java v0.1.0 (…/crates/jarde-java)
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.17s
CLIPPY-EXIT=0
```

No warning line is printed: the workspace is clean under the CI flag set.

## `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`

Authoritative totals, read as exit code + `grep -c "test result: ok"` + zero
`test result: FAILED` lines (never a broad awk):

```
WORKSPACE-TESTS-EXIT=0
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
test result: ok. 341 lines    (341 "test result: ok", 0 "test result: FAILED")
passed=3189 failed=0 ignored=87
```

## `openspec validate --all --strict`

```
Totals: 313 passed, 0 failed (313 items)
```

## The corpus fingerprint (`tests/p5_corpus_fingerprint.rs`)

The change's own fixtures are a corpus addition, so the manifest was regenerated with the
documented regenerator and the diff reviewed (9 entries added, one per fixture file — the
`README.md` is an excluded extension and the same bytes are already asserted by the fixture suite):

```
cargo test --locked -p jarde --test p5_corpus_fingerprint
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.09s
```

## The census (`crates/jarde-reader/src/classfile.rs`)

The fixture-population census moved with the new fixtures: `(927, 3957, 368, 2475, 8)` →
`(933, 3993, 368, 2475, 8)` — six classes and thirty-six bodies (eighteen per leg: `CP`'s four,
`MX`'s five, `NEG`'s nine), with the handler-record, branch-target and subroutine counts unchanged.
The expectation was **updated** (a new paragraph in the same list, the tuple re-measured), never
deleted or weakened. Measured both ways: without the new fixtures the recorded tuple passes
unchanged, with them the new tuple does.

## The change's own suites

```
cargo test -p jarde --test recover_instance_field_assignment_chains --test recover_chained_field_assignment --locked
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.18s   (recover_chained_field_assignment)
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.04s   (recover_instance_field_assignment_chains)
```

## The oracle leg

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.27s
```

No stale expectation: nothing was updated, deleted or weakened.

## Flake families

The first workspace run (before the census expectation was re-measured) reported three failures;
two of them are the known flake families and one was the census above.

* `p3_two_exit_return::complete_class_compiles_and_matches_eight_verified_paths` —
  `Scratch::new`'s name is `jarde-two-exit-{pid}-{nanos}` and `fs::create_dir` fails on a
  collision, which parallel execution can produce; isolated rerun ×2 with `--all-features`:
  `test result: ok. 7 passed; 0 failed` twice.
* `jarde-cli export_cli::a_counted_dimension_override_is_accepted_and_stops_the_run_at_that_dimension`
  — the cancellation/budget timing family the previous changes recorded; isolated rerun ×2 with
  `--all-features`: `test result: ok. 11 passed; 0 failed` twice.
* `jarde-reader classfile::repository_class_fixtures_validate_without_false_target_rejections` —
  the census: a real delta, classified above and re-measured.

The re-run of the full workspace gate after the census update is clean: `failed_lines=0`.
