# Tasks 3.1–3.3 — the gates, verbatim

Date: 2026-10-07 (UTC). All commands run in this worktree, in this order.

## The main anchor (task 3.1)

Both legs of the frozen fixture, each rendered, compiled and run (see
[02-gating-and-baseline.md](02-gating-and-baseline.md) for the transcripts and
`tests/double_brace_capture.rs` for the CI form):

```
$ javac --release 8 -g:none -d out23 DB.java 'DB$1.java' 'DB$2.java'   # exit 0
$ <jdk8>/bin/javac -g:none -d out8 DB.java 'DB$1.java' 'DB$2.java'     # exit 0
$ java -Xverify:all -cp out23 DB     -> 2/z
$ <jdk8>/bin/java -Xverify:all -cp out8 DB -> 2/z
$ cargo test --test double_brace_capture --all-features --locked
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.73s
```

The falsifiability self-check (the two perturbations, then reverted):

* the new arm disabled → `the_capture_companion_presents_the_capture_write_after_the_constructor_call`
  and `the_recompiled_family_compiles_and_runs_as_the_original_classes` **FAIL** (the regression is
  caught); the zero-regression test stays green;
* the argument walk's invocation arm dropped → `the_super_argument_probes_keep_the_current_refusal_and_the_byte_order`
  **FAILS** (the call-bearing argument would be reordered); the other three stay green.

## The order-sensitive standing controls (task 3.2)

```
$ cargo test --test ctor_reorder_dispatch_guard --all-features --locked
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.27s
$ cargo test --test fixture_behavior_guards --all-features --locked
test result: ok. 7 passed; 0 failed; 6 ignored; 0 measured; 0 filtered out; finished in 0.03s
$ cargo test --test fixture_behavior_guards --all-features --locked -- --ignored
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.59s
$ cargo test --test recover_synthetic_ctor_super_order --all-features --locked
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.57s
$ cargo test -p jarde-java --test p3_patterns --all-features --locked
test result: ok. 83 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.03s
$ cargo test --test anonymous_parameterized_root --all-features --locked
test result: ok. 4 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.34s
$ cargo test --test anonymous_supertype_return --all-features --locked
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.35s
```

Two stale pins the change deliberately moved, each an *assertion update* with the reason recorded
at the site (no test deleted, no assertion weakened):

* `crates/jarde-java/tests/p3_patterns.rs::a_synthetic_capture_before_a_user_class_constructor_call_stays_where_the_bytecode_made_it`
  — the dispatch guard's generator pin said "any non-Object super keeps the byte order". It now
  asserts both sides of the narrowed criterion: the same bytes **with** the class's own method
  (`read()I`, the shape that can read the moved field during the call) keep the byte order, and
  the constructor alone (no code the call could run) moves the group past it. The fixture-level
  dispatch guard is untouched and green.
* `crates/jarde-reader/src/classfile.rs::classfile::tests::repository_class_fixtures_validate_without_false_target_rejections`
  — the fixture-population measurement, re-measured per its own instruction and its convention:
  `(844, 3702, 290, 2273, 8)` → `(857, 3723, 290, 2273, 8)` (+13 classes, +21 bodies: the two new
  fixture families), with the change's line appended to the running comment.

## The full workspace suite (authoritative totals)

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
exit 0; test result: ok lines: 333; test result: FAILED lines: 0; passed: 3155
```

(Exact tail and totals in `03-workspace-tests-4.out`; the earlier runs are kept beside it:
run 1 was before the fixture additions (332 targets / 3151 passed), run 2/3 caught the two stale
pins above — run 3 also hit the registered load-sensitive flake
`bulk_recovery_delivery::one_declaration_bounds_the_librarys_own_presentation_too` at its 4-worker
case (`budget_stops` empty), which passes twice in isolation under `--all-features` and is listed
in `handoff.md` as a known flake family. The authoritative run above is green on the same test.)

## The corpus-moving discipline (task 3.2/3.3)

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 44.72s
```

No stale oracle expectation: the oracle leg is green as it stands (output in
[05-oracle-leg.out](05-oracle-leg.out)). The corpus delta and the fingerprint:

```
$ sh .../scan-corpus.sh   <baseline> <change> /tmp/jarde-scan-ctor-order    # 857 captures/leg, 2 render deltas
$ sh .../scan-evidence.sh <baseline> <change> /tmp/jarde-scan-evidence      # 2725 captures/leg, 1 render delta
$ cargo test --test p5_corpus_fingerprint --locked                          # FAILED: 17 unlisted new files
$ cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
$ git diff --stat tests/fixtures/corpus-fingerprint.json
 tests/fixtures/corpus-fingerprint.json | 85 ++++++++++++++++++++++++++++++++++
 1 file changed, 85 insertions(+)        # 17 new entries, zero rewritten
$ cargo test --test p5_corpus_fingerprint --locked
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out; finished in 0.28s
```

## fmt, clippy, openspec, diff check (task 3.3)

```
$ cargo fmt --all && cargo fmt --all -- --check
fmt clean
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ sh /tmp/ci-clippy.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 21.52s     # exit 0, no warning
$ openspec validate --all --strict
Totals: 307 passed, 0 failed (307 items)
$ git diff --check
diff check clean
```

## The implementation's own diff

`crates/jarde-java/src/ctor_order.rs` (+~200 lines: the module's section on the move, the second
arm, `declares_only_initializers`, `call_arguments_avoid`, `value_observes_moved`),
`crates/jarde-java/src/build.rs` (+1 line: the `class_methods` argument), plus the two stale pins
above and the new fixture/test/fingerprint files. No other production file is touched.
