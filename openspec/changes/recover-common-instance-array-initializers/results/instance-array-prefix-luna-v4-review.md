# Instance-array initializer projection: private v4 review

This is an unapplied review patch against the current product sources. It is a v3-derived candidate with the corrections below; the v3 patch and review remain unchanged.

Patch SHA-256: `20f9febf1e8cbc23c409f858455b96154554dc5e2707fbed361909f1e896af36`.

## Changes in v4

- Constructor census counts the complete method-header slice first and charges the selected-constructor vector before allocation. AST, field, field-header, and static-call target joins use one-match `Option` scans, avoiding the temporary match vectors. Each scan polls and charges analysis work.
- The constructor body is composed with `ClassSourceMethod::assert_projection_text(&body)`. This keeps the original artifact envelope, marker lines, and annotations without parsing/rebuilding the artifact format.
- The projection stages only field declarations and composed member texts. It does not add a fabricated `staged_member_emissions` record for a body that has no real anchor table. Existing member-text and staged-member occupancy checks refuse a competing later rewrite. The façade test checks `emission == None` and that physical method text still contains the original writes.
- All candidate fields and constructor texts remain local until all proof, output charges, and text composition succeed; the final loop commits the field declarations and member texts together. No late-budget rollback claim is made by the zero-budget/cancellation façade tests: those tests cover public entry/early-stop behavior only.

The opaque `jarde-java` sidecar retains only same-run claimed nonstatic field evidence for complete constructors. The projection still requires exact owner/name/descriptor joins, complete constructor census, direct-super common contiguous writes, matching typed literal/cast/call RHS facts, unique same-class static call targets, and source field order. It does not compare rendered RHS text.

## Added no-`<clinit>` control source

`results/no-clinit-super-args-v1/` contains a complete child class, superclass, and Runner. The child has two constructors with distinct `super(...)` arguments and distinct suffix writes; both have the same two-field final byte-array prefix and same-class static `mark`/`run` helpers. The static `trace` has no explicit initializer, so these sources should not require a `<clinit>`. Runner output is observation-based and no expected values are presented as an oracle.

These Java sources have not been compiled or run. The existing façade fixture positives still use the earlier classfile fixture with `<clinit>`; the new no-`<clinit>` path is therefore prepared but not yet exercised by a Rust fixture test. Root should first capture the original classfile/raw oracle, then wire its class bytes into the positive test if accepted.

## Validation boundary

No Git, Cargo, rustfmt, JDK, JADX, CLI, or product-source edits were performed. The patch was not applied, compiled, or tested. Root review and the separately owned Java oracle/replay remain outstanding.
