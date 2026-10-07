# Task 3.1 — the gates, verbatim

Every command below was run in this worktree with `CARGO_INCREMENTAL=0` (the CI environment sets
the same; it is a build-profile setting, not a different gate). The authoritative reading of a test
run is the exit code plus `grep -c 'test result: ok'` plus **zero** `test result: FAILED` lines —
never a broad awk over the log.

## Formatting

```text
$ cargo fmt --all -- --check
$ echo $?
0
```

Rerun after the last edit of this change (the `cargo fmt --all` that preceded it touched only
`tests/recover_short_circuit_local_branch_reads.rs`; `git status` lists no other file).

```text
$ git diff --check
$ echo $?
0
```

## Clippy (CI's own allowlist)

```text
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ sh /tmp/ci-clippy.sh
    Checking jarde-query v0.1.0
    Checking jarde-jvm v0.1.0
    Checking jarde-java v0.1.0
    Checking jarde v0.1.0
    Checking jarde-cli v0.1.0
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 12.53s
$ echo $?
0
```

The extracted command is CI's 30-entry `-A` allowlist over
`cargo clippy --workspace --all-targets --all-features --locked --`; no warning was emitted.

## The workspace test suite (authoritative)

```text
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
```

| reading | value |
| --- | --- |
| exit code | **`0`** (captured as `EXIT=0`) |
| `test result: ok` lines | **337** |
| `test result: FAILED` lines | **0** |
| `error: … targets failed` blocks | none |

The gate was run five times over the slice's life; the two runs on the final file state (after
`cargo fmt` and after the reader crate's census was re-measured) both read 337 ok / 0 FAILED, and
the last one captured its exit code:

| run | log | reading |
| --- | --- | --- |
| 1 | `gate-tests.log` | 335 ok, 2 FAILED — the reader census (re-measured, below) and the `p4_plugins` flake |
| 2 | `gate-tests2.log` | 337 ok, 0 FAILED |
| 3 | `gate-tests3.log` | 337 ok, 0 FAILED (after `cargo fmt --all`) |
| 4 | `gate-tests4.log` | 336 ok, 1 FAILED — the `p4_plugins` flake again |
| 5 | `gate-tests5.log` | **`EXIT=0`, 337 ok, 0 FAILED** |

The tail of the final run:

```text
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
```

### The one flake, classified

The first full run of this gate (before the reader crate's fixture census was re-measured) reported
two failing targets; one was this change's own census update (fixed and re-measured, see
`05-fingerprint.md`), the other was `-p jarde --test p4_plugins`:

```text
thread 'the_plugin_plane_leaves_the_structural_planes_own_answer_untouched' panicked at tests/p4_plugins.rs:541:5:
assertion `left == right` failed: a plugin run leaves the structural plane's own answer identical
```

The two documents differ in exactly one field — `"elapsed_millis": Number(0)` against
`Number(1)` — so the failure is the known timing family (`p4_plugins`, listed in `handoff.md`'s
flake families), not a behavior change. Single-test rerun ×2 with `--all-features`:

```text
$ cargo test --locked --all-features --test p4_plugins the_plugin_plane_leaves_the_structural_planes_own_answer_untouched
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
$ … (second run, same)
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.00s
```

## The oracle leg (ignored)

```text
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
running 3 tests
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.64s
```

## The change's own suites

```text
$ cargo test --locked --all-features -p jarde --test recover_short_circuit_local_branch_reads
test result: ok. 5 passed; 0 failed; 3 ignored; 0 measured; 0 filtered out

$ cargo test --locked --all-features -p jarde --test recover_short_circuit_local_branch_reads -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out
```

The zero-regression suites of the family (`p3_mixed_short_circuit_local`, `p3_scv_concat_consumers`,
`p3_boolean_contexts`, `p3_hoisted_boolean`, `p3_boolean_short_circuit_return`,
`p3_loop_boolean_exit`, `p3_loop_boolean_do`, `p3_short_circuit_*`, `preserve_local_scope_plan`) are
part of the 337 targets above and passed; their fixtures' renders are pinned byte-for-byte by
`01-gating.out` and `fingerprint.txt`.

## OpenSpec

```text
$ openspec validate --all --strict
Totals: 309 passed, 0 failed (309 items)
```
