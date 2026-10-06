# Task 3.3 — the gates, verbatim

Every command below was run in the worktree
(`/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a111be-fffa-73a2-8069-38fc4e6bde28`)
against the change's final tree. Disk was checked before each build round (`df -h /`, never below
15 GiB).

## `cargo fmt --all -- --check`

```
$ cargo fmt --all -- --check
FMT CLEAN
```

## The CI-exact clippy

```sh
sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh && sh /tmp/ci-clippy.sh
```

```
$ sh /tmp/ci-clippy.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 24.72s
CLIPPY_EXIT=0
```

Zero `warning:` lines in the log (`grep -c '^warning' = 0`).

## The workspace tests (authoritative form)

```sh
cargo test --workspace --all-targets --all-features --locked --no-fail-fast
```

The totals are read as the task's acceptance states — exit code, `grep -c "test result: ok"`, and
zero `test result: FAILED` lines (no broad awk sums: test *names* starting with "test result" poison
them).

| pass | exit | `test result: ok` | `test result: FAILED` | notes |
| --- | --- | --- | --- | --- |
| 1 (first full run) | 101 | 319 | 4 | the four deltas this change had to answer: the fixture census, the corpus fingerprint, and two loop guards the unconditional `Duplicate` acceptance moved (`p3_java_recovery`'s two hand-built array-length cases, `p3_loop_test_values`'s `storeTest`) |
| 2 | 101 | 323 | 0 | the guards were answered by pairing the copy with its store (`unobservable_store_dance_part`); this pass aborted one *target* on a stack overflow — see below |
| 3 (final) | 0 | 324 | 0 | every target, including the deep-recursion ones |

The stack overflow of pass 2 was a real finding, not a flake: the change had grown
`render_value`'s own frame, and `tests/p3_immediate_functional_receivers.rs`'s over-deep lambda
helper (a three-hundred-term sum rendered at the recursion bound) overflowed where the baseline
passed. The measurement is reproducible both ways:

```
$ git stash push crates/jarde-java/src/build.rs … && cargo test -p jarde --test p3_immediate_functional_receivers an_overdeep
test an_overdeep_lambda_helper_renames_instead_of_walking_an_unbounded_body ... ok

$ git stash pop && cargo test -p jarde --test p3_immediate_functional_receivers an_overdeep
thread '…' has overflowed its stack
```

The arm was moved into `duplicate_expression` (`#[inline(never)]`), so the recursive render's frame
is the baseline's again and the dance's frame is paid only where a copy is rendered. Pass 3 is the
result: 324 `ok`, zero `FAILED`, exit 0.

## `openspec validate`

```
$ openspec validate recover-dup-store-conditional --strict
Change 'recover-dup-store-conditional' is valid

$ openspec validate --all --strict
Totals: 305 passed, 0 failed (305 items)
```

## `git diff --check`

```
$ git diff --check
(no output)
```

## The corpus differential and the fingerprint

* `results/03-corpus-sweep.sh` — 6 moved classes, all classified in `03-corpus-delta.md`
  (this change's own four fixtures plus the two patrol anchors); two `package-info` candidates
  render under no name on both binaries, printed and not dropped.
* `tests/fixtures/corpus-fingerprint.json` — regenerated with the test's own documented command
  (`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`).
  The diff is **45 added lines and zero removed lines**: this change's nine new files (three sources
  and six class files), five lines each. No existing entry moved.
* `crates/jarde-reader/src/classfile.rs` — the fixture-population census is a **re-measure**, not a
  relaxation: `(783, 3275, 286, 2023, 8)` → `(789, 3311, 286, 2061, 8)`, +6 classes, +36 bodies,
  +38 branch targets, with handler records and subroutines unchanged. The comment above the tuple
  gains this change's paragraph naming the three classes, the two legs and the counts, exactly as
  the preceding changes' paragraphs do.

## The guard suites this change must not move

```
$ cargo test -p jarde-java --test p3_inner_assignment      → 4 passed; 0 failed
$ cargo test -p jarde-java --test p3_java_recovery         → 45 passed; 0 failed; 1 ignored
$ cargo test -p jarde --test p3_local_rewrite              → 7 passed; 0 failed
$ cargo test -p jarde --test p3_loop_test_values           → 4 passed; 0 failed
$ cargo test -p jarde --test recover_postfix_old_value_snapshot → 4 passed; 0 failed; 1 ignored
$ cargo test -p jarde --test recover_dup_store_conditional → 3 passed; 0 failed; 1 ignored
$ cargo test -p jarde --test recover_dup_store_conditional -- --ignored → 1 passed; 0 failed
```

The three standing order-sensitive controls are inside the workspace run: the anonymous
super-dispatch fixtures (`tests/fixtures/proved-java-structure/anonymous-super-dispatch/`, read by
`tests/p3_java_recovery.rs` and the structure tests) and the ordinary-new-void-effect shape
(`recover-statement-position-news`'s controls) both pass, and the corpus sweep renders their classes
byte-identically.
