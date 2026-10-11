# Compound negative-control observation verifier v3

This private read-only verifier checks the frozen 21-command replay at `/private/tmp/jarde-compound-negative-comparison-root-v1/execution.json`. It rechecks the 74 pinned inputs, raw command streams and exact argv, full compiled class sets, both JDK profiles, original/JADX runtime outputs, default/all class-source text and maps, and the actual javap instruction BCI sets and effects.

The measured physical methods contain 36 unique BCIs in `prefixIncrement` and 39 in `localAssignment`. The javap evidence shows `iinc` at BCI 16, and `dup` at 20 followed by `istore 4` at 21. The original class returns the expected output on both JDKs. JADX executes but has only 4 of 8 expected lines correct on each JDK. Jarde default/all sources fail compilation with two missing-return diagnostics each and have no runtime; their target methods are whole-method explanatory fallback after ownership overlap at BCI 32/34. This is a negative observation, not evidence that a CompoundIf Builder guard ran.

Owner class BLAKE3 and snapshot are checked against the separately frozen BLAKE3 oracle execution, its pinned executable, and exact raw stdout for the same SHA-256-pinned class input. The verifier does not run the oracle.

The verifier was only syntax-checked with Python AST parsing by the authoring agent. It has not been executed; parent validation is still required. No workspace, Git, Cargo, JDK, or product CLI was used.
