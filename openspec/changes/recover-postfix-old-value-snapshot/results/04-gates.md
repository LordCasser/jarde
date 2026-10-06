# Task 3.3 — the gates, verbatim

Run in this worktree on the final tree (after `cargo fmt --all`).

## `cargo fmt --all -- --check`
```
(exit 0, no output)
```

## CI-exact clippy

```
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
    Checking jarde-java v0.1.0 (/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11147-8c2b-76f3-af2d-b0508e018964/crates/jarde-java)
    Checking jarde v0.1.0 (/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11147-8c2b-76f3-af2d-b0508e018964)
    Checking jarde-cli v0.1.0 (/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11147-8c2b-76f3-af2d-b0508e018964/crates/jarde-cli)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 19.41s
```

## `cargo test --workspace --all-targets --all-features --locked --no-fail-fast`

```
exit=0
target results ok: 323
target results failed: 0
```

The known flake family `p4_plugins` failed once in an earlier round of the same command and
passed on the single-test rerun (the family rule: `p4_plugins` is the timing family the handoff
lists); the final run above is green with no rerun.

## `openspec validate --all --strict`

```
✓ change/type-immediate-functional-receivers
Totals: 303 passed, 0 failed (303 items)
```

## `git diff --check`

```
(exit 0, no output)
```

## The census and the fingerprint

- `crates/jarde-reader/src/classfile.rs` fixture census: `(765, 3179, 286, 2009, 8)` ->
  `(783, 3275, 286, 2023, 8)` (+18 classes, +96 bodies, +14 branch targets).
- `tests/fixtures/corpus-fingerprint.json`: regenerated with the test own command; the diff is
  130 added and 0 removed lines (26 new files x 5).

## The corpus render differential

`results/03-corpus-sweep.sh` (parent binary `/tmp/posv-baseline-bin/jarde-cli`, kept for a
replay; rebuild with `git archive HEAD~1 \| tar -x -C /tmp/posv-base && CARGO_TARGET_DIR=/tmp/posv-base-target cargo build -p jarde-cli --all-features --locked`):
29 classes moved, every one classified in [03-corpus-delta.md](03-corpus-delta.md).
