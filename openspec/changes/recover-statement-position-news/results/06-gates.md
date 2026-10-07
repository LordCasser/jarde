# 3.1/3.3 — the gates, verbatim

Every command below was run in this worktree on the final tree (the implementation, the fixtures,
the tests, the census ledger and the regenerated fingerprint), with the local toolchain
(`cargo 1.98.1`, `javac 23.0.1`, Corretto `1.8.0_432`).

## fmt

```
$ cargo fmt --all -- --check
FMT-OK
```

(The first check failed on two spots of this change's own new code and one of the updated test;
`cargo fmt --all` was applied and the check re-run clean — twice, because the tests and the census
ledger were edited after the first clean run.)

## clippy (ci.yml 46–76)

The task's own extraction (`sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh`),
and the CI step's full form, which appends `-D warnings` (the extraction's `grep` drops that line):

```
$ sh /tmp/ci-clippy.sh
    Checking jarde-java v0.1.0 (…)
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 14.74s
$ sh -c "$(cat /tmp/ci-clippy.sh) -D warnings"
    Checking jarde-java v0.1.0 (…)
    Checking jarde v0.1.0 (…)
    Checking jarde-cli v0.1.0 (…)
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 13.89s
```

Zero warnings and zero errors on the final tree.

## workspace tests — the authoritative totals

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
331
0
```

(`331` is `grep -c 'test result: ok'`; `0` is `grep -c 'test result: FAILED'`; the exit code is the
run's own — never a broad awk. The parent commit's run carries one target fewer: this change's
`tests/recover_statement_position_news.rs`.)

The first run on this tree carried four failures, all of them stale expectations of the corpus the
change moves, each updated or regenerated as the discipline requires — none of them a regression:

| target | failure | resolution |
| --- | --- | --- |
| `tests/class_source.rs` | `anonymous_superclass_refuses_unproved_local_declaration_sites`: the child method is now complete, so the anonymous fold happens and `anonymous_child_methods_incomplete` is no longer stated | the block's assertion updated (fold + child statement + absence of the old diagnostic) |
| `tests/recover_javac8_allocation_qualifier_null_check.rs` | `a_construction_with_no_rendering_reader_refuses_on_both_legs`: the reader is now the statement position's discard | renamed and updated; the member-form boundary it froze is kept verbatim |
| `tests/p5_corpus_fingerprint.rs` | `corpus_files_match_the_recorded_fingerprint`: 17 new fixture files | `--ignored regenerate_corpus_fingerprint` |
| `jarde-reader` lib | `classfile::tests::repository_class_fixtures_validate_without_false_target_rejections`: `(844, 3702, 290, 2273, 8)` vs the recorded `(835, 3649, 290, 2273, 8)` | ledger entry added for this change's nine classes and fifty-three bodies |

The second full run is the `331/0` above: no flakes were seen in either run.

## The change's own suite

```
$ cargo test --locked --all-features --test recover_statement_position_news
test result: ok. 8 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

$ cargo test --locked --all-features --test recover_statement_position_news -- --ignored
test the_recovered_text_compiles_and_runs_identically_on_both_legs ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

The ignored test is the replay: the patrol's frozen `B5` and this change's `SP`/`SB` compile on both
compilers (`javac --release 8` and Corretto `1.8.0_432`), run under `-Xverify:all` and answer
exactly what their original classes answer; `B6` and `SPN` — the classes a refusal keeps incomplete
— must not compile; and the hand-built `SPC`'s original class answers `CST` under `-Xverify:all`
while its quoted text does not compile.

The two updated tests and their neighbours:

```
$ cargo test --locked --all-features --test class_source anonymous_superclass_refuses_unproved_local_declaration_sites
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 101 filtered out
$ cargo test --locked --all-features --test recover_javac8_allocation_qualifier_null_check
test result: ok. 6 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
$ cargo test --locked --all-features --test p3_ordinary_new_invokes
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

## The ignored P3 compile-and-execute oracle (corpus-moving discipline)

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.58s
```

**No stale oracle expectation**: the oracle was run after the corpus sweep and needed no assertion
update (nothing in its sample set moved — the sweep's only moved classes are the delta list of
`results/05-corpus-delta.md`).

## The corpus differential, the census and the fingerprint

```
$ sh openspec/changes/recover-statement-position-news/results/03-corpus-sweep.sh
SELF-TEST OK: B5 refusals 3 -> 0 with all three statements; SPC/SPN byte-identical; BW/BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical
pass A loose candidate classes: 2838
pass A: moved=13 unrendered=1
archive candidate classes: 739
pass C: moved=1 unrendered=1
moved classes: single-class=13 jar=1 total=14
unrendered candidates: A=1 C=1
```

Every one of the 14 is classified in `results/05-corpus-delta.md`; the two `unrendered` candidates
are the same `package-info` entries every precedent sweep printed, identical on both binaries. No
class became more refused or more quoted.

The census and the fingerprint are re-measured per precedent:

```
$ cargo test -p jarde-reader --lib --all-features --locked classfile::tests::repository_class_fixtures
test classfile::tests::repository_class_fixtures_validate_without_false_target_rejections ... ok
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 177 filtered out
    # (835, 3649, 290, 2273, 8) -> (844, 3702, 290, 2273, 8), ledger entry added
$ cargo test --test p5_corpus_fingerprint --all-features --locked -- --ignored regenerate_corpus_fingerprint
test regenerate_corpus_fingerprint ... ok
$ cargo test --test p5_corpus_fingerprint --all-features --locked
test result: ok. 5 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out
```

The census delta is this change's own nine classes and fifty-three bodies: `SP` (ten bodies), its
`SP$Inner` companion (one), `SB` (six) and `SPN` (nine) on both compiler legs, plus the hand-built
`SPC` (one) — no handler record, no branch target and no subroutine, so the tuple's last three
values do not move. The fingerprint diff is exactly seventeen added files, all under
`tests/fixtures/recover-statement-position-news/` (nine class files and eight sources), and nothing
else: no entry removed, none changed.

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
