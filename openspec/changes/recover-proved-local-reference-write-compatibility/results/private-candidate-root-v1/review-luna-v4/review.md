# Local reference-write private candidate v4 delta review

Read-only review of the frozen v4 private candidate and its single-line delta from reviewed v3. Candidate `build.rs` SHA-256: `e45b9a8e1c6e8d32e82b65098853e5323101b4a4e2a400cebb545d950479bb52`; patch SHA-256: `aa2f2a949f0eeceb39bc7c64ba3f3ecbc551899f8d0310fd8dab39af227457d0`; v3 base candidate SHA-256: `5d42d8fcf1fd8210a2e394a9742888cfde82357cd8c32ca73239862456fcbd67`.

The entire candidate-to-candidate diff is exactly one condition in `local_reference_write_compatibility`: the cached Object-store trigger now skips `bci <= first.at` rather than only `bci == first.at`. This fixes the v3 false-trigger case: an earlier unresolved same-slot Object store can no longer itself start a “later Object write” candidate or supply its refusal BCI. Once a legitimate post-first Object trigger exists, the subsequent physical-slot census still sees every read/write occurrence, rejects any unresolved identity, preserves multiplicity, and isolates a proven different identity. Thus the correction does not weaken the completeness gate or promote an ambiguous access.

No new adverse issue was introduced by this delta. Earlier v3 review findings still apply: the v2 return-type mismatch is fixed; exact Object Store trigger, complete slot occurrence census, reference/null-only writes, array/header/catch/handler exclusions, exact Load-to-Return stack use, charged scans, and accurate refusal text remain as reviewed. No `consumer.slot` check is needed because the consumer is the loaded Stack value's `Return`; no extra opcode gate is needed beyond the established typed-frame category constraints for this narrow case. This does not claim full JVM verification.

**Disposition:** v4 closes the identified static trigger-boundary blocker. This is private static review only; it does not establish compilation, formatting, tests, generated-source behavior, control outcomes, full-class parity, or product acceptance. Those guarded checks remain outstanding.

No shared edit, Git mutation, build, or test was performed.
