# Task 2.4 Verification

`tests/p3_shift_expressions.rs` compares essential and all-evidence output for the frozen `callTarget` member. The text is identical; all-evidence maps helper calls at BCIs 1 and 5 and `ishl` at BCI 8 to the physical `callTarget` member, and the mapped shift text contains `<<`. Each helper invocation appears once in the emitted member text, so evidence replay does not duplicate physical emission or invent another call.

The same test exercises low `ir_items`, `analysis_steps`, and output-byte limits, source-map exhaustion with retained partial evidence, and cancellation before work. The resource stops are reported through the existing class-source outcomes and do not certify incomplete shift text as recovered. Validation used a dedicated Cargo target directory, then removed it to avoid retaining build artifacts.

The recursion depth limit exists inside expression recovery (`jre_recursion_bound`) but is not configurable through the class-source budget. The existing deep-expression test generates source with runtime `javac` and is ignored by default; the frozen shift input has no expression deep enough to reach the internal bound. Task 2.4 therefore records this boundary rather than adding an ungrounded depth assertion or changing the public budget surface.

Verification: `CARGO_TARGET_DIR=/tmp/jarde-task24-target cargo test --test p3_shift_expressions -- --nocapture` passed (5 tests); `openspec validate recover-shift-expressions --strict` passed.
