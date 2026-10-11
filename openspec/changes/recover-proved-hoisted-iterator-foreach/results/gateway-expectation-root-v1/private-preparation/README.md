# Iterator foreach and loop gateway regression candidate

Fresh workspace `command 85` failed only `crates/jarde-java/tests/p3_loop_exit_gateways.rs::one_arm_straight_prefix_loop_and_optional_tail_are_closed` (14 passed, 1 failed), at the old `contains("while (")` assertion. The raw report for `VariablePostfixLoop.countEmpty(Ljava/util/List;)I` is now one null guard around `for (java.lang.Object iteratorElement23 : arg0)`, followed by the original `String` cast, `isEmpty()` condition, counter increment, and return. The raw failure's other source/map and region assertions had not yet run.

The candidate removes only the three obsolete iterator-local/while/hasNext/next spelling checks. It replaces them with exact header and preserved cast/body-condition/body-effect/return assertions. The existing default-vs-all text and source-map equality checks remain. The one-owner check still requires exactly blocks `[6, 13, 22, 39, 42]` in the null-guard arm; its source anchor now requires the exact foreach header to carry method BCIs 6/7. Other BCI, owner, transfer, update-slot, budget and cancellation assertions are untouched. In particular, the changed-for-update controls remain required to emit `while`, and no-backedge controls remain required to emit neither loop form.

Static scan of the remaining Rust unit/integration test sources found no other `Iterator.hasNext` / `.iterator()` loop-shape positive assertion. The closest while-only tests are intentionally different:

- `p3_loop_exit_gateways.rs::no_prefix_while_latch_keeps_physical_source_and_one_owner` (around line 980): independent plain `while` latch control.
- `p3_loop_exit_gateways.rs::rejected_for_update_slot_keeps_the_independent_while_latch` (around line 1246): local-slot mutation must keep the existing `while`.
- `p3_loop_exit_gateways.rs::non_backedge_transfer_does_not_gain_a_loop_latch_origin` (around line 1309): byte-mutated non-backedge must not acquire a loop.
- `p3_inner_assignment.rs::the_loop_condition_control_presents_the_assignment_in_place` (around line 249): assignment in an effectful method-call condition, not an iterator loop.
- `p3_java_recovery.rs`'s explicit `while (local1 != 0)`/`do-while` assertions (around lines 923, 972): primitive decrement/counting forms.
- `p3_effectful_exits.rs`, `p3_shared_join_finally.rs`, and `p3_void_loop_finally.rs` assert `while(true)` exit/finally structure or loop counts, not iterator-header spellings.

The scan covered Rust tests under `crates/jarde-java`, `crates/jarde-jvm`, `crates/jarde-reader`, and `crates/jarde-query`; there were no matching iterator-foreach while-only assertions in the latter three crates. This is a static candidate only. It has not been compiled or run. The root's planned full `VariablePostfixLoop` oracle run is still required before deciding to apply it.
