# Task 3.1 — the gates, in the authoritative form

Date: 2026-10-07 (subagent worktree, not pushed). Baseline: this worktree's parent commit
`d1f0aa39`. Every gate ran on the **final** state of the worktree (after the last source edit, the
census re-measure and `cargo fmt`).

## The workspace suite (authoritative totals)

```
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
EXIT=0
ok-targets=325
failed-lines=0
```

Read the way the discipline states it: the **exit code** (0), the count of `test result: ok` lines
(**325**) and the count of `test result: FAILED` lines (**0**) — never a broad `awk` sum. The
previous change's run reported 324 targets; this change adds one target (its own root test file).

## Formatting and lints

```
$ cargo fmt --all -- --check
fmt=0

$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ sh /tmp/ci-clippy.sh
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 1m 06s
clippy=0
```

The clippy line is the CI job's own 30-item allowlist, verbatim, with `-D warnings`: it finishes
with zero warnings.

## Specification

```
$ openspec validate --all --strict
Totals: 305 passed, 0 failed (305 items)
validate exit=0

$ git diff --check
diff-check exit=0
```

## The suites the acceptance names

```
$ cargo test -p jarde-java --locked --test p3_scv_concat_consumers        → 5 passed; 0 failed
$ cargo test -p jarde-java --locked --test p3_repeq_boolean_arguments     → 2 passed; 0 failed
$ cargo test -p jarde-java --locked --test p3_conditional_values          → 7 passed; 0 failed
$ cargo test -p jarde-java --locked --test p3_patterns                    → 83 passed; 0 failed
$ cargo test -p jarde-java --locked --test p3_preceded_catches            → 11 passed; 0 failed
```

`p3_patterns`' `a_chain_a_branch_cuts_is_refused_rather_than_written_as_one_expression` and
`p3_preceded_catches`' `a_cross_block_builder_keeps_its_split_refusal_at_its_own_to_string` /
`a_split_refusal_names_the_to_string_the_chains_own_builder_reaches` are the corpus's own
`jre_concat_split` negatives: they pass with the refusal texts unchanged.

## The change's own test

```
$ cargo test --test recover_inline_conditional_concat_operands --locked
test result: ok. 4 passed; 0 failed; 1 ignored

$ cargo test --test recover_inline_conditional_concat_operands --locked -- --ignored
test result: ok. 1 passed; 0 failed
```

## The oracle ignored leg (corpus-moving discipline)

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 48.72s
```

No stale oracle expectation moved: the sweep's ten moved classes are not in this leg's set.

## The corpus differential, the fingerprint and the census

`evidence/03-corpus-delta.md` states the sweep (`SELF-TEST OK`, `moved classes: single-class=8
jar=2 total=10`, every delta classified), the fingerprint regeneration (`+85` lines, `0` removed)
and the fixture-population re-measure (`(789, 3311, 286, 2061, 8)` → `(801, 3363, 288, 2095, 8)`).
Its full transcript is `evidence/03-corpus-sweep.out`.

## What is not run here

No independent review is performed by this change's own tests: task 3.2 (root's independent
re-check of the gating experiment, the read-only sub-proof reuse and the anchors/negatives) stays
open, and nothing above claims it.
