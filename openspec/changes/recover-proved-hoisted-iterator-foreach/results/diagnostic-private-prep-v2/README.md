# Hoisted iterator same-IR Phi-use diagnostic candidate v2

Private, unapplied diagnostic delta derived from v1. It preserves the frozen class/method/path filter, inner-only owner-fact continuation, existing iterator proof predicates, and forced `Ok(None)` before projection. The current production `build.rs` remains pinned to the v1 base (`f5c0757…`); the facade harness is the unchanged v1 copy.

The added `phi_use_expansion` event starts from the iterator local value written at init store BCI 33, selected from that instruction's local-slot write. It follows SSA `bci=None` uses to their containing blocks, records every Phi in those blocks with block/slot/output/inputs and output `SsaValue`, and recursively follows Phi values, inputs, and `replaced_by` links. For each traced value it records its SSA facts and each instruction consumer's opcode/read/write facts plus same-block effect facts. This is a snapshot of the frozen SSA graph. It neither classifies a value as dead/live nor changes the refusal at `iterator_value_and_phi_use_ownership`.

Root's actual v1 diagnostic execution is pinned in `pins.json` (execution SHA `40eee52f…`; raw stderr SHA `4a24afb4…`). It observed inner owner `Elevated { owner: [1] }`, init/test/next all `LocalVariable { slot: 5, index: 0 }`, and the first post-owner refusal at `iterator_value_and_phi_use_ownership`. The init value had three `bci=None` uses in block 97. The v2 downstream Phi expansion has not run, so no block-97 Phi structure or liveness conclusion is claimed.

Existing helper boundaries are reviewed in [`helper-review.md`](</private/tmp/jarde-hoisted-iterator-diagnostic-prep-luna-v2/helper-review.md>). In particular, `reuse.rs::representative` and `trivial_phi_outputs` are private to `reuse.rs`; their bounded trivial-phi forwarding serves a reference lifetime-boundary proof, not a generic Phi liveness query.

Static review only: lexical delimiter and logger-call-arity checks are recorded in `syntax-review.json`. No Rust parser/compiler, Cargo, JDK, JADX, CLI, or Git command was run. No shared file was edited. The guarded runner remains the root's responsibility.
