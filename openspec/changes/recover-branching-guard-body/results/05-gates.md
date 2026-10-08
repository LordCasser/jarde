# Task 3.1 — the gates (this worktree, final state)

```
$ cargo fmt --all -- --check                      # clean (rerun after the final edits)
$ sh /tmp/ci-clippy.sh                            # ci.yml 46-76 verbatim: exit 0, no warning
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
    WORKSPACE-EXIT=0
    grep -c "test result: ok"      = 345
    grep -c "test result: FAILED"  = 0
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.15s
    ORACLE-EXIT=0
$ openspec validate --all --strict
    Totals: 316 passed, 0 failed (316 items)
```

(345 = the previous state's targets + this slice's `tests/recover_branching_guard_body.rs`; 316 =
the previous state's items + this change. The workspace run reports 3206 individual tests passed
and 92 ignored, with no `test result: FAILED` line — so no known flake family was sighted and no
single-test rerun was owed.)

### Verbatim gate tails (final state, authoritative)

```
$ cargo fmt --all -- --check
fmt-exit=0

$ sh /tmp/ci-clippy.sh        # = ci.yml 46-76's own command
    Checking jarde-cli v0.1.0 (…/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 23.36s
clippy-exit=0
# the same command with the job's own tail (`-D warnings`) appended: Finished … in 20.98s, exit 0

$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
    test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s
    WORKSPACE-EXIT=0
  # authoritative totals: exit 0 + `grep -c "test result: ok"` = 345 + `grep -c "test result: FAILED"` = 0

$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
    test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.15s
    ORACLE-EXIT=0

$ openspec validate --all --strict
    Totals: 316 passed, 0 failed (316 items)

$ cargo test --test recover_branching_guard_body --locked -- --ignored
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 2.06s
$ cargo test --test recover_nested_lock_finally_bodies --locked -- --ignored
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.50s
$ cargo test --test recover_io_resource_finally --locked -- --ignored
    test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.45s
$ cargo test --test recover_loop_test_copy_store --locked -- --ignored
    test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.64s
$ cargo test --test recover_lock_guard_loop_finally --locked -- --ignored
    test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```

### The corpus gates

* census: 968 classes, **10 renders moved, every one classified** (`04-corpus-and-oracle.md`);
* committed fingerprint: purely additive (90 insertions, 0 removals; 18 new entries);
* reader population assertion: updated to the measured `(968, 4161, 456, 2660, 8)` with the
  fixture's registration comment, green.

### Disk and `/tmp` occupancy

`df -h /` before every build round; the lowest free space seen during the slice was 28 GiB (above
the 20 GiB floor), after the workspace test's link step. `/tmp` occupancies this slice created and
their fate:

| path | what | fate |
| --- | --- | --- |
| `/tmp/gate-baseline-cli` | the HEAD build (66 MB) | removed at the end of the slice |
| `/tmp/bg-gate/` | the admission binary, the two render sets, the gating matrix, the census, the logs | removed |
| `/tmp/bg-scratch/`, `/tmp/bg-scratch2/` | the reading controls' sources and classes | removed (the sources are committed under `results/reading-controls/`) |
| `/tmp/bg-replay/` | the manual driver replay | removed |
| `/tmp/ci-clippy.sh` | the CI clippy command | removed |
| `/tmp/trace-lg-transfer.txt` | the HEAD instrumentation transcript | removed (committed as part of `results/trace-admission.txt`) |
