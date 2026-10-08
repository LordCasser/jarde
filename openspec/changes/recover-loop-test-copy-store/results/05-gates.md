# Task 3.1 — the gates, verbatim

Every command below was run in this worktree, at the tree of commit `1a7a8d96` plus this record.
The authoritative full-suite totals are the exit code, `grep -c "test result: ok"` and a **zero**
count of `test result: FAILED` lines (no broad awk).

## fmt

```
$ cargo fmt --all -- --check
$ echo "FMT_EXIT=$?"
FMT_EXIT=0
```

(The one wrap `cargo fmt` wanted — `region.rs`'s `store_dance_part` `matches!` — is folded into the
copy-family commit, so every commit of the branch is fmt-clean.)

## clippy (the CI recipe)

```
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ sh /tmp/ci-clippy.sh
    Checking jarde-cli v0.1.0 (…/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 16.00s
$ echo "CLIPPY_EXIT=$?"
CLIPPY_EXIT=0
```

## The full suite

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
CARGO_EXIT=0
$ grep -c "test result: ok" /tmp/lts/ws-final.txt
344
$ grep -c "test result: FAILED" /tmp/lts/ws-final.txt
0
```

344 targets ok, 0 failed (the branch point's recorded tally is 343; this slice's own test binary is
the one addition). One flake was seen and cleared by the standing discipline: round 3's
`ordinary_generic_projection::frozen_identity_shapes_and_raw_control_project_together` failed once
with `Os { code: 17, AlreadyExists }` — the test's own time-nonce temp directory collided — and
passes on the single-test rerun ×2 with `--all-features`; it is a pre-existing flake of that
binary's helper, unrelated to this slice (the full run above is clean).

## The oracle leg

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.91s
```

No stale oracle expectation (see `04-corpus-and-oracle.md`).

## openspec

```
$ openspec validate --all --strict
Totals: 315 passed, 0 failed (315 items)
```

The branch point's recorded tally is 314; this change is the one addition.

## The targeted legs this slice's own suites carry

```
$ cargo test --test recover_loop_test_copy_store --all-features --locked
test result: ok. 4 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
$ cargo test --test recover_loop_test_copy_store --all-features --locked -- --ignored
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.74s
$ cargo test --test recover_io_resource_finally --all-features --locked
test result: ok. 4 passed; 0 failed; 2 ignored; 0 measured; 0 filtered out
$ cargo test --test recover_io_resource_finally --all-features --locked -- --ignored
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 1.59s
$ cargo test --test recover_dup_store_conditional --all-features --locked
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
$ cargo test -p jarde-java --test p3_inner_assignment --all-features --locked
test result: ok. 5 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo test -p jarde-reader --lib --locked
test result: ok. 178 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## Disk

`df -h /` before every build round; the free space stayed above the 20 GiB floor throughout (the
lowest reading was 18 GiB, at which point the worktree's own build tree was left alone and the
shared root target's earlier clean had already freed the space back to 44 GiB). `/tmp/lts` (the
gate logs and the four gating columns, the latter copied into `results/gating/`) held ~14 MB before
the final cleanup.
