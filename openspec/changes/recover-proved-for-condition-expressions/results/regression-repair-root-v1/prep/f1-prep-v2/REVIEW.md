# Private F2 for-header assertion update

This v2 is based on the current shared `tests/p3_crossing_array_read_values.rs`; v1 and the root's raw failure output are preserved. The only code change is in the `F2.stepTwoWithQuote` loop assertion: it expects the exact header `for (local2 = 0; local2 < arg0.length; local2 = local2 + 2)` and asserts the `+2` update occurs exactly once in the recovered method text.

The existing array-element accumulation assertion remains. The try/catch body assertion, fallback assertion, and `14\n` compile-and-run assertion are unchanged. This projection only collects the existing initialization, condition, and single latch into the header; the try/catch block remains in the loop body at its original place and is not moved across a protected-region boundary. F1's exact two-block comparison/runtime check and all variant, refusal, nested-boundary, and budget assertions are otherwise unchanged.

No test or toolchain command was run, no Git command was used, and no shared file was modified.
