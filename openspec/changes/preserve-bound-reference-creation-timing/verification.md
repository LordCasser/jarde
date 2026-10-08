# Verification

## Implemented

- Reused the existing `receiver_nonnull` gate for direct and adapted bound method references. A direct reference without a same-run proof is refused.
- Builder proves entry `this` only when the member facts say it has a receiver, uses the existing completed-allocation/stable-move proof, and recognizes String constant type/non-null facts from the existing constant producer reader. Static local 0 remains an ordinary nullable parameter.
- For a refused bound reference inside a constructor argument, source mapping now retains the exact SSA producer's closed construction interval. Refused lambda capture producers are included only on that refusal path. Additional interval BCIs are counted in the existing bounded quote walk.
- `constant_of_value` is reused for capture producer types and follows only direct String/Class constants through stores and `dup`; `written_type` retains its previous unknown-frame guard. The Class literal `dup; getClass; pop` source rendering remains conservatively refused and is recorded as a deferred duplicate-shape limitation.

## Targeted checks

Using `CARGO_TARGET_DIR=/Users/lordcasser/workspace/projects/jarde/target CARGO_BUILD_JOBS=1 CARGO_INCREMENTAL=0`:

- `cargo test -p jarde-java --test p3_java_recovery a_nullable_direct_bound_reference_keeps_creation_and_consumer_producers_quoted -- --nocapture` — passed. Exact origin sets are `{0,3,4,5,10,13}` for the constructor and `{0,1,6}` for the standalone factory; `arg0::start` is absent. The existing allocation-backed receiver site remains accepted.
- `cargo test -p jarde-java --test p3_java_recovery javac_bound_reference_nonnull_proofs_preserve_safe_forms_and_reject_class_dup -- --nocapture` — passed. Frozen Corretto 8 KnownBound recovers `this::value` and `"value"::length`; the javac Class literal duplicate shape remains quoted/refused.
- `cargo test -p jarde-java --test p3_java_recovery proved_entry_this_keeps_bound_method_reference -- --nocapture` — passed.
- `cargo test -p jarde-java --lib plan_accepts_boxed_array_length_but_keeps_instance_handle_arity_and_captures_exact -- --nocapture` — passed. The no-proof direct-reference plan refuses; the same plan accepts with an explicit non-null proof.
- `cargo fmt --all` — passed.

The frozen `NoCheck` and `NoStand` originals were independently verified by root on OpenJDK 23 and Corretto 8 as `creation=ok`, then `invocation=NPE`; this implementation intentionally emits no bound reference for them. Full workspace tests, clippy, both-JDK full-class replay, and OpenSpec strict validation remain with root (tasks 2.1–2.2).

## Artifacts and limits

No new Cargo target directory was created; builds used the existing shared target above. The positive source/class fixture is under `openspec/evidence/java-syntax-2026-10-08/bound-reference-creation-timing/positive/`. No temporary build fixture was created in `/tmp` by this implementation. Class-literal duplicate-shape recovery remains deferred and must not be inferred from this change.
