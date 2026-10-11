# Private clone-path proof test candidate

This candidate appends one focused reader-to-proof test to the pinned `region.rs` source. It uses the exact `CompoundStatementConditions.class` fixture that root reports accepted by both Java 8 and Java 23 verifiers.

The test does not synthesize block IDs or edges. It runs the existing method-IR reader, finds the BCI 11 comparison by examining canonical block membership and the SSA terminal instruction, then verifies the reader's two clone contexts (`[0]` and `[3]`). It locates the BCI 3 caller continuation from the same canonical graph and verifies the actual `Return { call_site: 0 }` edge from the sole BCI 35 `ret` clone. The proof is asked to close clone path `[0]` at the pathless BCI 3 continuation; it must refuse before scanning and preserve visited ownership, catch joins, tail state, and fragment state.

Pins are in `metadata.json`. The fixture copy matches the repository fixture SHA. The candidate is append-only against the exact base pin. No Cargo, JDK, Git, or formatter was run; Rust compilation and test execution remain for root's guarded runner.
