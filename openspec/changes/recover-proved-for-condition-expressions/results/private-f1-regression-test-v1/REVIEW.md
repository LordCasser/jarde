# Private F1 two-header test candidate

The candidate changes only `tests/p3_crossing_array_read_values.rs` in this private directory. The failing frozen test output shows two source differences in `F1`: `stepTwo` and `stepTwoWithCall` each project the same local-bound, array-length condition and `+2` update into a `for` header.

The test keeps `F1_VERBATIM` as the baseline and constructs its only accepted expected text by replacing those two complete historical blocks, each required to occur exactly once. The full class text remains byte-for-byte strict everywhere else. It adds explicit source assertions for exactly two `for` headers, the exact step-two header in each method, and exactly one `+2` update per method in the header; the existing loop bodies must remain present and `while` must be absent from those two methods.

The recovered full F1 source is compiled and run through the existing `compile_and_run` helper. A small helper writes the frozen `F1.class` bytes directly and runs them with `java -Xverify:all`; stdout is compared between the two runs. No output is hardcoded. The F2 assertions in the same test and all variant/refusal/nested/budget tests remain as written. No historical OpenSpec evidence file is changed.

No tests, Rust/Cargo, Java, JADX, CLI, or Git commands were run. This is a private candidate, not a verified test result.
