# Parameterized root-method refusals (`recover-anonymous-parameterized-root`)

Each subdirectory freezes one refusal shape for the widened root-method gate. All are compiled
with `javac --release 8 -g:none -d . <Root>.java`, all run clean under `java -Xverify:all`, and
all keep physical class text (the `$1` spelling javac refuses) — byte-identical before and after
the slice, archived under
`openspec/evidence/java-syntax-2026-10-04/recover-anonymous-parameterized-root/negatives/`.
The CI guard is `tests/anonymous_parameterized_root.rs`
(`the_parameterized_root_refusals_keep_their_physical_text`).

| Directory | Shape | Refusal landing |
| --- | --- | --- |
| `multiple-parameters/` | the root method declares two parameters (`create(String, int)`); `extra` is deliberately uncaptured so the child still carries exactly one `val$` field and the refusal lands on the widened gate itself | `anonymous_super_return_type_unproved` — the parameter table widens to exactly `(P)Lparent;`, never to several parameters |
| `capture-from-local/` | the descriptor is the single-parameter form but the allocation's capture argument is a root **local** (`val$local`), not the parameter | `anonymous_capture_argument_unproved` — the value-flow proof requires the unmodified parameter slot 0; the descriptor match alone proves nothing. (Before the slice this shape was refused by the same code at the gate; the landing moves into the proof, the physical text does not move.) |
| `parameter-also-consumed/` | the capture parameter is consumed a second time inside the root method (the `echo` copy) beside the allocation argument | `anonymous_capture_argument_unproved` ("consumed beyond the allocation argument") — design Open Question (b) resolved as default-refuse: a second consumption has no proof it re-spells losslessly |
| `instance-method/` | the root method is an instance method, so the child carries the enclosing instance (`this$0`) beside the capture field — a two-field child the shape gate never accepted | `anonymous_super_child_shape_unproved` (pre-existing gate); the projection's own `ACC_STATIC` requirement (precedent parity) stays as defense in depth for shapes javac cannot mint — design Open Question (a) resolved as default-refuse |
| `supertype-return/` | the root method declares the supertype `Renderer` while the allocation's direct superclass is `Base` | `anonymous_super_return_type_unproved` — the return part stays exactly the superclass type; this slice widens only the parameter table (ring 2 is a separate, unproven slice) |
