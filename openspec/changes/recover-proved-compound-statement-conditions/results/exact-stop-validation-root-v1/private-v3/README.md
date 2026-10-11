# Compound Stop exact pins with FT2 switch-tail coverage

Private test-only patch based on the clean root backups.

Base SHA-256 pins:

- `region.rs.base`: `26e3646f032894135e44306db7ebcd00fa7d56b28ff4062a4fbfcfb7fb788766`
- `build.rs.base`: `34c8d100ba8dcae10b0c1561874ca082312ab49a0f6c3662cb2357d3da1d586d`

The region patch keeps the reviewed exact Stop tests and adds the FT2 switch straight-tail late-budget and pre-cancel checks from the private FT2 candidate. Its observed full-tail result is 54 analysis steps from next BCI 175 to boundary BCI 197. A limit of 53 stops with `AnalysisSteps`, written 0, usage 53, at BCI 197. Pre-cancellation stops at BCI 175 with usage 0. The successful case verifies the complete run claims BCI 175 once and leaves BCI 197 outside; late Stop and cancellation compare the full run and Walker-owned mutable state.

No added `eprintln!` calls, no production logic changes, and no Reader diagnostics. No clone test was added or copied. No compiler, Cargo, JDK, Git, or CLI was run, and the repository was not modified. Apply only after reviewing the unified diffs against the pinned bases.

Patch files: `region.rs.patch`, `build.rs.patch`. All candidate and patch hashes are recorded in `pins.json`.
