# Private test-only compound-condition expectation update

This private candidate changes only the existing test in `tests/p3_reference_slot_lifetimes.rs`. It was based on the live source pinned in `pins.txt`. No repository file was changed, and no Git, Cargo, JDK, formatter, or other toolchain command was run.

The test is renamed to `null_first_exact_builder_writes_refine_local_type_and_compound_condition`, and its comment now states the two expected changes. It keeps the frozen `NullThenBuilder.baseline.java` and `.class` inputs intact. Expected output is derived in the requested order: first replace the unique `Object local2;` declaration with `java.lang.StringBuilder local2;`, then assert and replace exactly one five-line nested-if fragment with `if (arg0 && local2 == null)`. The existing whole-class exact equality for essential/all, method count and identity, per-method text equality, source-map existence and physical-method-origin checks remain unchanged. All other tests, including the seven frozen negatives, are untouched.

Files:

- `p3_reference_slot_lifetimes.rs`: complete candidate source.
- `p3_reference_slot_lifetimes.rs.patch`: one-hunk unified diff against the pinned live test source.
- `pins.txt`: baseline, candidate, patch, frozen Java baseline, and frozen class SHA-256 pins.

The candidate has not been compiled or executed. Root's independent whole-class JVM/JDK matrix is the intended validation after review and application.
