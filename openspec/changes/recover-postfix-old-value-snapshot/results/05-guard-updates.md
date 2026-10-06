# The guard expectations this change updates

The acceptance allows a guard test that pins a **Phase-A anchor's refusal** to be updated to the new
recovered state (assertion change, never deletion). Two assertions move; nothing is deleted and no
negative is weakened.

## `tests/p3_local_rewrite.rs`

The fixture's `LocalRewrite.saved`/`conditional` are exactly two consumer positions of this slice:

| test | before | after |
| --- | --- | --- |
| `a_store_of_a_superseded_load_is_refused_with_the_read_named` → `a_store_of_a_superseded_load_writes_the_postfix_expression` | the store at BCI 4 and the load at BCI 0 are quoted, the return is refused as an undeclared local (`saved(x) == y` shape) | the store writes `int local1 = arg0++;` and the return reads `local1`; the test now asserts the postfix form **and** that neither the increment's own statement nor a store of the slot's name survives |
| `a_branch_on_a_superseded_load_is_refused_with_the_read_named` → `a_branch_on_a_superseded_load_tests_the_postfix_expression` | the region is quoted whole, the quote names the branch at BCI 4 and the load at BCI 0 | the condition writes `if (arg0++ > 0)` with both arms presented |

Both renamed tests keep the *shape* of their assertion (what must not be written) and add the
recovered form; the body-identity controls in the same file (`a_slot_written_and_read_again_…`,
`a_value_loaded_before_a_loop_…`) are untouched and still pass, which is what says the check was
narrowed and not switched off. `LocalRewrite.post` (`return x++;`, the return position) is still
refused — the whole-method quote is unchanged there, and the file's `the_producer_behind_a_cast…`
test still passes.

## `crates/jarde-reader/src/classfile.rs`

The fixture-population census is a **re-measure**, not a relaxation: the pinned tuple moves from
`(765, 3179, 286, 2009, 8)` to `(783, 3275, 286, 2023, 8)` — +18 class files (nine per leg), +96
bodies, +14 branch targets, with the handler-record count unchanged. The comment above the tuple
gains this change's paragraph naming the eight classes, the two legs and the counts, exactly as the
preceding changes' paragraphs do.

## `tests/fixtures/corpus-fingerprint.json`

Regenerated with the test's own documented command
(`cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`).
The diff is **130 added lines and zero removed lines**: the 26 new files (8 sources + 18 class
files), five lines each. No existing entry moved.

## Budget and billing pins: not updated

`crates/jarde-java/tests/p3_patterns.rs`'s `straight_finally_reuses_its_guard_verdict_with_a_tight_budget`
(pinned at 52 `analysis_steps`) and `tests/p5_bulk_corpus.rs`'s two pinned `Billing` rows both
passed **without any change**, because the snapshot proof bills a candidate only after its cheap
identity checks — the same order the return-position proof takes. The first draft charged every
`dup`/`dup_x1`/`iinc` in the body and moved those pins (+97 on `flat-mixed`, +1389 corpus-wide); the
charge was moved behind the identity checks, which is both the precedent's shape and the honest
reading (no proof work happens for an instruction that is not a candidate).
