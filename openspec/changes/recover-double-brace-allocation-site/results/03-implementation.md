# Tasks 2.1/2.2 — the implementation and the suites

## What changed

| file | change |
| --- | --- |
| `src/facade.rs` | `project_class_source_double_brace` (the pass, called before the companion-body dispatch) and `prove_double_brace_allocation` (one allocation's four-criteria proof and its staged projection); `spellable_source_name`; `error_ends_the_request`; a third `AnonymousOwnerCensusPath` discriminant |
| `crates/jarde-java/src/report.rs` | `emit_class_source_anonymous_instance_block` (the block reading: the call, the certified store, the block, the guards), `class_source_allocation_argument_local` (the site argument at any statement position), `class_source_anonymous_child_name`, `allocates_an_anonymous_child` (the retention's third reason), `CaptureReadAnchor` (`Converted` over the field) and `project_class_source_converted_parameter_reads` |
| `crates/jarde-java/src/emit.rs` | the anonymous re-emission sets the emitter's `initializer` flag for `<clinit>` (measured correction (f)) |
| `src/class_source.rs` | `projected_statements_method_text`: the write-back under the member's own physical prefix (markers kept byte for byte) with the derived range |
| `src/member_inner.rs` | `CaptureReadReceiver` (`EntryThis`/`ConstructorThis`) threaded through `prove_anonymous_val_capture` and `scan_capture_method_uses`; `value_from_constructor_this`/`constructor_this_write` |

Every existing certificate passes the **narrow** reading (`EntryThis`, `CaptureReadAnchor::Field`),
so no other slice's acceptance set moves; this slice's own entry points pass the widened ones. The
corpus scans (`04-gates.md`) show the resulting delta surface: the anchors' renders, and nothing
else.

The pass claims the text exactly as a companion-body projection does — it sets
`anonymous_interface_projection = Projected { derived }` with one
`MemberFamilyDerivedProjection { kind: NestedAnonymousExpression, .. }` per anchor (the anchors are
the root class, the child class, the allocation's method point and the companion constructor's) —
which is what keeps the interface/superclass dispatch and the folds after it from re-presenting the
text. A shape the criteria do not prove writes nothing: no diagnostic, no execution stop, the
presentation the class already had. A stop that *ends the shared request* (a cancellation, an
exhausted dimension) is propagated to the caller's own error handling.

## The test faces

* `tests/recover_double_brace_allocation_site.rs` (new, four tests): the control's double-brace
  form (both legs, both anchors' own shape), the three negatives' unchanged text, the control's
  recompiled run under `javac --release 8` **and** the real javac 8 (`java -Xverify:all` → the
  fixture's own answer), the physical companion staying queryable, and the fixture directory's own
  README/`freeze.py` presence.
* `tests/double_brace_capture.rs` (**semantic update**): the host-form assertion now reads the
  double-brace form at both anchors and asserts that no companion name and no synthetic capture
  survives; every companion-side assertion is unchanged, and the two probes' pins are untouched.
  The file's module documentation states the replacement.
* `tests/fixtures/proved-java-structure/double-brace-allocation-site/` (new): the control and the
  three negatives, both legs, with `README.md`, `freeze.py` and `SHA256SUMS`.
* `crates/jarde-reader/src/classfile.rs`: the fixture census tuple and its paragraph.

## The invariants this slice must keep, and where they are checked

| invariant | check |
| --- | --- |
| the double-brace body's evaluation order is the constructor's | the three order-sensitive control suites (`ctor_reorder_dispatch_guard`, `fixture_behavior_guards` incl. `anonymous-super-dispatch`, `p3_ordinary_new_invokes`), green |
| every other shape keeps path-A's presentation byte for byte | the negatives' text assertions, and the two-leg corpus scans (0 render deltas beyond the anchors) |
| path A's criteria are untouched | its code is unmodified; the two contained readings are new parameters whose existing callers pass the old value |
| the recompiled strip is the original program | the fixture's own answer (`2/z`) on both compilers, in this slice's suite and in path A's |
| the corpus does not move silently | the three two-leg scans, the render fingerprint, the fingerprint manifest, the reader census, the oracle leg |
