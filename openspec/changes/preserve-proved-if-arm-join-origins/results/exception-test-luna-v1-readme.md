# Private observer test patch: canonical exception edge on an if-arm transfer

This is a private patch only. It changes no repository file, and no Cargo, JDK, rustfmt, CLI, or runtime command was run.

The test reuses `fixture_value_attempts_with_tree`'s existing real class-byte -> `analyze_method_ir` -> canonical/SSA -> `region::recover` setup. The old helper keeps its arguments and return tuple; a narrowly-scoped observer sibling exposes the real recovered tree, canonical CFG, SSA, Operations, and existing budget only to this test. All existing callers delegate with a no-op observer.

The fixture path is the root-provided compiled sample `openspec/changes/preserve-proved-if-arm-join-origins/results/exception-join-case-root-v1/classes/ifjoin/ExceptionIfJoin.class`. The test does not synthesize a graph or Region. It prints the recovered tree, requires exactly one recovered If at branch BCI 12 with the javap-pinned join BCI 27, takes the actual final direct Straight block of that If's then arm, and checks its real SSA BCIs `[15, 18, 21]` plus decoded `Invoke`, `Increment`, and `Transfer` operations. It then filters the full canonical edge list for that exact block and requires exactly a Normal edge to the recovered join and an Exception edge to handler BCI 36. Finally, it calls `nonempty_if_join_transfer` on that actual arm and requires `None`.

The test is intended to fail if real recovery does not expose the target If shape. The fixture's javap layout and bytes are root-reported evidence; this patch has not been compiled or run, so compileability and recovered-region shape remain unverified. Additional ordinary or exceptional outgoing-edge cases beyond this single real sample remain outside this patch.

Proposed focused test name: `jarde_java::build::tests::exception_if_join_transfer_rejects_a_canonical_exception_edge`.
