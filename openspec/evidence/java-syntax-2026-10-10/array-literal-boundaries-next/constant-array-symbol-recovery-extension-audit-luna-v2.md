# Exact name ranges through Emitter replay

Supplement to `constant-array-symbol-recovery-extension-audit-luna.md`. This is a read-only design clarification from the current `emit.rs`, `evidence.rs`, and `source_map.rs`; no product files or tests were changed and no toolchain, Git, or verifier was run.

## Finding

The generic replay is a viable exact-range source for the existing private class-source projection, but the facade cannot call it directly: `Emitter`, `Emitter::replay`, `Emitter::stmts`, and `finish_replay` are private to `emit.rs`. The minimum seam is one crate-private body-only replay adapter in `emit.rs`. It can replay the already committed class-source statement text with the same formatter and return the usual `SourceMap`; it does not need `RecoveryFacts`, a declaration envelope, or a second mandatory collector in the commit pass.

The returned map does not label spans with AST node kinds. Therefore, a matching substring alone is insufficient. Pair each projected use `(name, primary BCI)` with recorded spans whose **primary** origin BCI equals that BCI and whose `Segment::text(body)` equals the full name. Accept only one match per use and one-to-one distinct ranges; reject missing matches, duplicate use keys, multiple matching segments, or a reused range. For repeated array elements, their bytecode origins should distinguish use sites; the exact-count checks make any same-BCI ambiguity a refusal instead of guessing. This uses byte ranges written by the AST replay, not offsets reconstructed with `match_indices`.

## Why replay has the needed information

- [`emit_class_source_statements`](../../../../crates/jarde-java/src/emit.rs#L194-L217) writes just the body statements at the supplied indentation and returns their exact text. Its `Emitter::commit` stores no segments, consistent with the module's D3 contract.
- [`Emitter::replay`](../../../../crates/jarde-java/src/emit.rs#L482-L515) already takes the budget, member, class/nested-class context, exact artifact text, `SegmentPublication`, and `EvidencePhase`. The helper can call it with the same context and `SegmentPublication::Whole`, then call `stmts(statements, indentation)` directly. It does not call `envelope` or `body`, so it needs no `RecoveryFacts` and does not introduce an envelope into the body artifact.
- [`Emitter::put`](../../../../crates/jarde-java/src/emit.rs#L1535-L1570) checks every replay write against the artifact at that byte offset. [`Emitter::node`](../../../../crates/jarde-java/src/emit.rs#L572-L610) records the start/end offsets from those writes and the node's `OriginSet`; `expr` routes `IntegerConstantName` through its own `node` at [`emit.rs:1163-1181`](../../../../crates/jarde-java/src/emit.rs). Thus the leaf's segment is the exact name token range in the already emitted body.
- [`SourceMap::segments`](../../../../crates/jarde-java/src/source_map.rs#L320-L353) preserves every selected span, including nested spans, and [`Segment::text`](../../../../crates/jarde-java/src/source_map.rs#L259-L307) slices that exact span from the replayed artifact. `covering` is deliberately not appropriate here because nested statement/expression nodes hide the inner leaf; scan `segments()` and filter exact primary BCI plus exact text instead.
- [`SegmentPublication::Whole`](../../../../crates/jarde-java/src/evidence.rs#L582-L620) records all anchored spans. `EvidencePhase` starts with `new()` and charges `IrItems` for each anchored replay span, stopping without claiming a complete result if the allowance ends ([`evidence.rs:663-757`](../../../../crates/jarde-java/src/evidence.rs)). The adapter must require a complete replay and propagate cancellation/budget/gate stops before returning ranges. This work occurs only for the requested projection; it does not allocate segments on ordinary commit runs.

## Matching and refusal rule

Use the existing projection handoff's `IntegerConstantNameUse { name, bci, ... }` as the expected use set, extended only as required to express array leaves if the current boolean discriminator is removed. For each use:

1. Reject repeated expected `(name, bci)` keys; a single source instruction must not silently claim two output leaves.
2. Find source-map segments with `segment.origin().primary().bci() == use.bci` and `segment.text(body) == use.name`.
3. Require exactly one segment and require its half-open range not to have been assigned to another use.
4. Require every expected use to resolve. Store the returned `start..end` directly in the existing derived projection record with its field and method/BCI anchors.

Outer nodes may share the leaf BCI, but their spans include statement or array syntax and so do not match the entire identifier text. If any additional node does have that exact text and primary BCI, the match count becomes ambiguous and the projection is refused. This avoids assuming that BCI filtering alone identifies an AST leaf. A custom `ExprKind` tag collector inside `expr` could be more explicit, but is not necessary for this constrained projection unless a real fixture demonstrates a same-BCI, same-text collision that the exact-count rule cannot conservatively reject.

## Transaction and budget boundary

The adapter should return ranges only when the replay reached the end and `written == artifact.len()`. A replay mismatch, phase stop, or incomplete range set leaves the staged class-source projection unpublished. The current facade already stages projected method text and derived records before assigning the root text ([`src/facade.rs:19220-19313`](../../../../src/facade.rs)); keep this publication boundary.

This does add a replay pass and source-map-sized temporary segment table when the projection runs, and it charges the existing `EvidencePhase` budget per anchored node. That is explicit, request-scoped work. It avoids a per-commit `Vec`, a new public report type, and any range-from-text reconstruction. A specialized replay that stores only named leaves would require branching at `Emitter::expr` and a new private range result path; do not add it unless measured memory or budget pressure justifies the extra mechanism.

## Minimal implementation seam

Add a crate-private helper beside `emit_class_source_statements` that accepts the same statements/member/class context/indentation and the exact committed body text. It should instantiate `Emitter::replay(..., SegmentPublication::Whole, &mut EvidencePhase::new())`, set the same initializer flag as `emit_class_source_statements` (`member.name.0 == b"<clinit>"`), invoke only `stmts`, call `finish_replay`, and return the complete `SourceMap` (or exact ranges after applying the matching rule above). Setting that flag keeps the helper's replay semantics aligned with the body-only commit helper; the narrow array-return MVP itself is an ordinary method. Map replay's `Halt::PhaseStopped` to the phase's stop reason, propagate a real stop, and treat a gate mismatch or uncovered suffix as the existing source-map mismatch stop. Keep the helper private to the crate and leave `emit_class_source_statements`'s commit behavior unchanged.

The next minimal acceptance case remains `ConstantIntArray`: one named initializer leaf, exact body replay, one unique segment whose text is `CONST_INT`, and a derived record covering exactly that token. A repeated-name fixture is useful only after the one-leaf seam works; it must demonstrate distinct BCIs/ranges or a conservative refusal, not string-search behavior.
