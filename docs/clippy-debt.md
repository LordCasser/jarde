# Clippy baseline debt

The CI Clippy gate denies warnings except for the lint names listed in `.github/workflows/ci.yml`.
This exception list records existing style and API-shape debt; it does not lower compiler warnings,
Clippy's correctness or suspicious groups, or any other Clippy lint. The exception list is global, so
new instances of one of these exact lint names will also be allowed until that lint is removed from
the list. Reduce this baseline incrementally and delete each exception when its count reaches zero.

## Observed baseline

On 2026-09-28, `rustup run stable cargo clippy --workspace --all-targets --all-features --locked
--message-format=json` with Rust/Clippy 1.98.1 reported 322 warnings. The failing GitHub run
36365687753 used Clippy 1.98.0 and stopped in `jarde-java` after 85 warnings; those were the first
module-level failures, not the entire workspace baseline.

| Source file | Warnings |
| --- | ---: |
| `crates/jarde-java/src/build.rs` | 45 |
| `crates/jarde-java/src/concat.rs` | 4 |
| `crates/jarde-java/src/emit.rs` | 4 |
| `crates/jarde-java/src/enumswitch.rs` | 8 |
| `crates/jarde-java/src/field.rs` | 2 |
| `crates/jarde-java/src/guard.rs` | 5 |
| `crates/jarde-java/src/init.rs` | 4 |
| `crates/jarde-java/src/region.rs` | 66 |
| `crates/jarde-java/src/report.rs` | 33 |
| `crates/jarde-java/src/reuse.rs` | 6 |
| `crates/jarde-java/tests/p3_carried_conditional_arguments.rs` | 1 |
| `crates/jarde-java/tests/p3_conditional_field_writes.rs` | 1 |
| `crates/jarde-java/tests/p3_conditional_values.rs` | 1 |
| `crates/jarde-java/tests/p3_effectful_exits.rs` | 1 |
| `crates/jarde-java/tests/p3_exception_only_catchall.rs` | 1 |
| `crates/jarde-java/tests/p3_inner_assignment.rs` | 1 |
| `crates/jarde-java/tests/p3_intermediate_join.rs` | 1 |
| `crates/jarde-java/tests/p3_loop_exit_gateways.rs` | 1 |
| `crates/jarde-java/tests/p3_loop_terminal_return.rs` | 1 |
| `crates/jarde-java/tests/p3_shared_join_finally.rs` | 1 |
| `crates/jarde-java/tests/p3_shared_tail.rs` | 1 |
| `crates/jarde-java/tests/p3_short_circuit_chain_controls.rs` | 1 |
| `crates/jarde-reader/src/classfile.rs` | 1 |
| `src/class_source.rs` | 28 |
| `src/enum_constants.rs` | 3 |
| `src/facade.rs` | 84 |
| `src/member_inner.rs` | 3 |
| `tests/class_source.rs` | 5 |
| `tests/ordinary_generic_projection.rs` | 1 |
| `tests/p3_bitwise.rs` | 1 |
| `tests/p3_boolean_short_circuit_return.rs` | 1 |
| `tests/p3_loop_boolean_do.rs` | 1 |
| `tests/p3_loop_boolean_exit.rs` | 1 |
| `tests/p3_multi_resource_twr_geometry.rs` | 2 |
| `tests/p3_numeric_comparison.rs` | 1 |
| `tests/p3_postfix_handler_boundary.rs` | 1 |

| Clippy lint | Warnings |
| --- | ---: |
| `too_many_arguments` | 80 |
| `cloned_ref_to_slice_refs` | 56 |
| `collapsible_if` | 38 |
| `type_complexity` | 31 |
| `len_zero` | 26 |
| `needless_option_as_deref` | 14 |
| `needless_borrow` | 13 |
| `useless_conversion` | 12 |
| `large_enum_variant` | 8 |
| `question_mark` | 6 |
| `comparison_to_empty` | 4 |
| `op_ref` | 4 |
| `manual_range_patterns` | 3 |
| `if_same_then_else` | 3 |
| `filter_map_bool_then` | 2 |
| `filter_next` | 2 |
| `unneeded_struct_pattern` | 2 |
| `redundant_guards` | 2 |
| `map_identity` | 2 |
| `redundant_slicing` | 2 |
| `unnecessary_get_then_check` | 2 |
| `unnecessary_unwrap` | 2 |
| `redundant_locals` | 2 |
| `replace_box` | 1 |
| `map_clone` | 1 |
| `unnecessary_mut_passed` | 1 |
| `single_element_loop` | 1 |
| `unnecessary_to_owned` | 1 |
| `needless_lifetimes` | 1 |

## Follow-up

Pay down these warnings with focused changes in their owning modules. Start with mechanical
single-site fixes; review `too_many_arguments`, `type_complexity`, and `large_enum_variant` against
the intended API and proof-state design before changing those shapes. This baseline is a CI bridge,
not a conclusion that every warning should be fixed mechanically.
