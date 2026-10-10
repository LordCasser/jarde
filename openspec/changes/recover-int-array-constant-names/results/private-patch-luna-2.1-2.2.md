# 2.1/2.2 private patch handoff

Patch: `private-patch-luna-2.1-2.2.patch`
SHA-256: `cff8532daf70de060fcad9c7ee4ed09cd398c47057b0b956be3b9ba54c740e21`

This is an unapplied, untested patch prepared against the current four source files. It does not edit `tasks.md` or product files in the checkout.

The report transformer adds only top-level direct `Return(NewArray)` admission for descriptor-approved `)[I` methods, one-dimensional initialized `int[]`, and direct typed-`int` literal elements. It keeps switch labels and selected-arm returns as distinct use kinds; those retain their existing text-needle range path. Array uses receive exact body ranges from a body-only `Emitter::replay` with `SegmentPublication::Whole`, full byte comparison, `EvidencePhase` accounting, and unique primary-BCI/name/physical-method matching. Missing or ambiguous ranges refuse the local projection. The commit emitter remains text-only.

The facade requires complete physical executions, exact `)[I` admission, and no earlier `projection_inputs.member_texts`. `ClassSourceMethod` maps body ranges through the original envelope and placement, checking current member text still matches the original recovered artifact before replacement. A final poll precedes publication.

The added tests cover repeated names at distinct BCIs, exact token-only ranges and Field/MethodPoint anchors, unchanged physical method text, preserved switch-label behavior, same-BCI range ambiguity refusal, and body-replay `IrItems` stop propagation.

No `cargo`, Rust compiler, JDK, CLI, Git, or test command was run. Root review should confirm the four-file patch applies cleanly on the intended base after the independent instance patch, then run formatting and focused Rust tests. Tasks remain 2/8 complete; 2.1/2.2 are not marked done.
