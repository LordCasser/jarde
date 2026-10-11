# Private clone-path proof test candidate v2

This candidate is rebased on the current live `region.rs` pin and adds one reader-to-proof test. It uses the exact legacy clone fixture already present in the change results. The test discovers all block IDs and paths from Reader's canonical graph, verifies the two clone contexts and the real `ret@35` return edge to caller continuation BCI 3, then checks that closing a clone-local root at that pathless continuation refuses without changing Walker ownership/catch/tail state.

The class file's SHA-256 is pinned by the fixture artifact metadata; the in-test `PhysicalDefinitionId` uses BLAKE3 as required by its digest type, and its assertion now uses root's independently observed BLAKE3 value. The analysis-step assertion reads through `walker.budget`, avoiding a simultaneous borrow of the separate local budget while `Walker` retains its mutable reference.

`region.rs.patch` adds only this test to the current source. No production code changes. No Cargo, JDK, Git, or formatter was run; the Rust test is uncompiled and unexecuted.
