# Existing SSA consumer helper review

Read-only review of the frozen production sources.

- `crates/jarde-jvm/src/ssa.rs:206-215` exposes `SsaUse::block/bci`; `:233-258` exposes `SsaValue::uses/replaced_by`; `:273-291` exposes `SsaPhi::block/slot/value/inputs`; `:477-518` exposes the table's values, `value`, `phis`, blocks, effects, and block lookup. These are sufficient to enumerate Phi records and their downstream facts without adding a helper.
- `crates/jarde-java/src/reuse.rs:319-327` has `representative`, which follows `replaced_by` with a bound. It is private to `reuse.rs`.
- `reuse.rs:765-835` (`reference_values_stay_before_boundaries`) follows None-use phi records only when the helper below proves trivial forwarding. It also imposes a next-write boundary and checks only reference-preserving operations. It is a lifetime-boundary proof, not general liveness.
- `reuse.rs:837-886` (`trivial_phi_outputs`) scans every Phi in one block, canonicalizes operands, and forwards only when all operands represent the same source and the Phi has a replacement. It is private to `reuse.rs`; it does not mark nontrivial Phi outputs dead.
- The current foreach proof is `crates/jarde-java/src/build.rs:21224` onward. The private v2 candidate labels its current gate `iterator_value_and_phi_use_ownership` at line 21958: iterator-value uses must be the header reads or the narrowly recognized local header Phi. The block-97 None uses fail that predicate. The neighboring array foreach proof begins at current production `build.rs:21720` and similarly binds its local Phi to a specific induction/length role.
- `build.rs:32090` `consumed` belongs to a purpose-specific proof context; it traverses allowed value consumers for that proof and is not a generic use/liveness service.

The minimal logger uses public SSA fact accessors to expose all Phi rows in blocks reached from the iterator value's None-use chain, then walks replacement links and Phi inputs. It records ordinary consumers and their instruction/effect facts but makes no dead/live claim. Any product proof for block 97 still needs separate evidence for every incoming arm, alias/replacement chain, and consumer.
