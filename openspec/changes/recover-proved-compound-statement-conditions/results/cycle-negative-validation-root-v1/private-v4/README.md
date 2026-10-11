# Compound cycle negative test candidate v4

This private delta is based on the currently formatted `crates/jarde-java/src/region.rs` at SHA-256 `f4167bef0479c51c0e045fd50685a616fde5e987069bfe65ed10a4ec581682c3`. It changes only the reader-backed test `real_compound_non_tree_conditions_refuse_without_walker_state_leaks`.

The failing lookup's generic panic did not identify which fixture and BCI was missing. The lookup now reports the variant label, requested original BCI, every published canonical identity `(id BCI, jsr path, original block starts)`, and `CanonicalCfg::unreachable()` identities. This preserves the assertion that method entry and each expected branch block have an exact pathless canonical identity.

The shared physical return at BCI 83 is different: normalization can fuse a straight-line block into its sole predecessor. The irreducible bytecode mutation changes the old `62 -> 83` transfer to `62 -> 40`; if that leaves 74 as the only predecessor of 83, canonical normalization is permitted to represent original starts `[74, 83]` under identity BCI 74. `block_covering(83)` locates the pathless canonical identity whose `CanonicalBlock::blocks()` includes original start 83, without assuming the normalized identity starts there. It still fails with the full identities if no canonical block covers 83, and fails separately if that identity is missing from the normal-flow view. The compound proof itself still receives the canonical join node and all existing refusal/ownership assertions remain intact.

Normalization facts checked in source:

- `crates/jarde-jvm/src/canonical.rs:70-104` defines canonical identity as original start plus jsr path; `CanonicalBlock::blocks()` gives original block starts represented by a possibly fused node (`:147-189`).
- `crates/jarde-jvm/src/canonical.rs:330-356` states that published-but-entry-unreachable IDs remain in `blocks()` and are separately listed by `unreachable()`; absence from both can mean normalization never created a node.
- `crates/jarde-java/src/normal_flow.rs:96-106,122-171` builds the normal-flow projection over every canonical block and preserves its IDs; it filters edge kinds, not nodes.
- In this fixture there is no jsr/ret, so every expected root/branch identity is pathless. BCI 0 is the method-entry identity; physical branch instruction BCI 26 is obtained from the last SSA instruction in that block. Cycle targets and sources 29, 40, 51, 62, and 74 are raw block starts from the frozen fixture graph. The shared exit lookup alone permits a fused id whose `blocks()` includes 83.

This is a test-diagnostic/normalization-identity correction, not a production algorithm change and not an assertion waiver. Root must run the targeted test to learn which canonical BCI was actually absent before accepting this explanation. No Cargo, JDK, CLI, Git, or formatter was run here.
