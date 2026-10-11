# Combined computed-for task-1.1 diagnostic candidate v3

This private, unexecuted candidate combines the prior same-IR SSA/CFG diagnostic with the Builder statement/materialization diagnostic. It has no production behavior changes; the build-side probe is `#[cfg(test)]` and gated to the frozen `lastIndexOf([IIII)I` member.

V3 fixes the two compile errors root reported for Builder v1: `ArrayInitializers` is referenced as `crate::build::ArrayInitializers`, matching the `Inputs` field's declared type in `build.rs`; `Operation::Store.slot` is matched against integer slot `4`, matching its `u16` declaration in `facts.rs`. The Store check therefore uses the actual Operation API, while the SSA slot checks continue to use `Slot::Local(4)`.

Static review also checked every field in the `Inputs` literal against the live `Inputs<'a>` definition. The real-IR and Builder diagnostics still run in the same frozen reader/API test and consume the exact same analyzed artifacts. No assertion or production gate was added. Root-reported Builder-v1 compile failure raw is pinned below for provenance; this candidate has not been compiled or run.

Both bases match the requested/current source pins: `region.rs` `acdb5d22195ce0b5d283fe1b7a3003f35830c85811b18e2e713b7f4728e93168`; `build.rs` `f4660755cf2ebc699dccf8ee7f7c984a94f6695a22a62cf649d4b728c76a1ade`. Apply only after review, run the exact test once, and restore both source files byte-for-byte.

Exact test: `region::tests::cf07_return_arm_latch_proof_stops_at_shape_and_edges_from_real_ir`.
