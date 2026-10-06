# Task 3.3 — the gates, verbatim

Baseline: the merged state (`119ddc0d`), which is this branch's parent commit. Every command below
was run on the **final** file state of this change (after `cargo fmt`), in the isolated worktree.

## fmt

```text
$ cargo fmt --all -- --check
fmt: clean
```

## clippy — the CI-exact invocation

```text
$ sed -n '46,76p' .github/workflows/ci.yml | sed 's/^ *//' | grep -E "^cargo|^-A" | tr '\n' ' ' > /tmp/ci-clippy.sh
$ sh /tmp/ci-clippy.sh
cargo clippy --workspace --all-targets --all-features --locked -- -A clippy::too_many_arguments -A
clippy::cloned_ref_to_slice_refs -A clippy::collapsible_if -A clippy::type_complexity -A clippy::len_zero
-A clippy::needless_option_as_deref -A clippy::needless_borrow -A clippy::useless_conversion -A
clippy::large_enum_variant -A clippy::question_mark -A clippy::comparison_to_empty -A clippy::op_ref -A
clippy::manual_range_patterns -A clippy::if_same_then_else -A clippy::filter_map_bool_then -A
clippy::filter_next -A clippy::unneeded_struct_pattern -A clippy::redundant_guards -A clippy::map_identity
-A clippy::redundant_slicing -A clippy::unnecessary_get_then_check -A clippy::unnecessary_unwrap -A
clippy::redundant_locals -A clippy::replace_box -A clippy::map_clone -A clippy::unnecessary_mut_passed -A
clippy::single_element_loop -A clippy::unnecessary_to_owned -A clippy::needless_lifetimes
    Finished `dev` profile [unoptimized + debuginfo] target(s) in 53.58s
clippy exit=0
clippy warning+error lines: 0
```

No warning line of any kind: the new code carries its own `#[allow]`s nowhere, and the dead-code
warning the first draft produced (`FieldCopyShape::Chain::stores` never read) was closed by reading
it — the plan's own store set is what makes a chain read from a later copy of another chain its
*own* tail rather than a second shape.

## The workspace suite — authoritative totals

```text
$ cargo test --workspace --all-targets --all-features --locked --no-fail-fast
tests exit=0
ok lines: 326
FAILED lines: 0
```

The three counts are the whole accounting: the exit code, the number of `test result: ok` lines, and
zero `test result: FAILED` lines. (A broad `awk`/`grep` over the log is not the criterion this file
uses.)

### The tests this change re-measured

Five suites pin **exact** budget arithmetic, and this change adds one bounded phase that charges the
same `IrItems` dimension — so their numbers moved, with the assertion's meaning kept:

| test | before | after | why |
| --- | --- | --- | --- |
| `p3_array_initializers::a_method_without_array_allocations_keeps_its_tight_ir_budget` | 84 | 87 | the field-copy proof's scan of a body with no copy |
| `p3_array_initializers::a_partial_dimension_allocation_…` | 105 | 110 | the same |
| `p3_hoisted_boolean::the_type_decision_is_billed_…` (`RELAYED_IR_ITEMS`, `IR_ITEMS_BEFORE_THE_PLAN`) | 464 / 374 | 479 / 389 | the same, and the cutoff still lands on `x`'s write at BCI 1 |
| `p3_required_conversions::the_conversion_costs_no_ir_item_…` | 165 | 181 | the same (the conversion itself still costs nothing) |
| `p3_shift_expressions::shift_recovery_stops_cleanly_at_resource_limits_…` | 1 000 | 1 180 | the last method's analysis (1 150) still completes and its recovery (1 218) still runs out |
| `p5_bulk_corpus` (six rows + the two arms) | 2 372 / 4 112 / 1 975 / 20 327 / 1 975 / 3 019; 33 780 ×2 | 2 473 / 4 298 / 2 058 / 21 278 / 2 058 / 3 213; 35 378 ×2 | the same, re-read with `record_the_billing_table` |

The fixture census and the fingerprint moved with the two new fixture classes (both compiler legs):

```text
(801, 3363, 288, 2095, 8)  ->  (805, 3405, 288, 2095, 8)
$ cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
+30 lines in tests/fixtures/corpus-fingerprint.json (the six new files)
```

## The oracle leg (the corpus-moving discipline)

```text
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
running 3 tests
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 45.10s
```

No expectation was stale: every leg compiles and executes the recovered bodies and compares their
answers, and this change's corpus deltas are all recoveries except `BI`, whose differing method is
byte-identical before and after (see `results/03-finding-bi-anchor15.txt`).

## The change's own replay

```text
$ cargo test -p jarde --test recover_chained_field_assignment --all-features --locked
test result: ok. 3 passed; 0 failed; 1 ignored; 0 measured; 0 filtered out

$ cargo test -p jarde --test recover_chained_field_assignment --all-features --locked -- --ignored
test the_recovered_text_compiles_and_runs_identically_on_both_legs ... ok
```

## openspec

```text
$ openspec validate --all --strict
✓ change/recover-chained-field-assignment
Totals: 305 passed, 0 failed (305 items)
```

## git diff --check

```text
$ git diff --check
clean
```

## Known flakes

None observed in this change's runs: the workspace suite was run twice end-to-end (before and after
`cargo fmt`) and both runs were green with zero `FAILED` lines, and no test in this change's own file
is timing-dependent. No single-test re-run was needed.
