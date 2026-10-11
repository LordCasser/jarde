# Review

Changes from v2 are narrowly scoped: selected class arguments now validate the observed dotted Java name (`jadx.tests.integration.switches.<binary-name>`), and the same-class Inner constant projection is applied to its full source plus `f1` method source when comparing against old output. Root substitutions remain unchanged.

The v3 report verifier was run against the actual eight old/new default/all rows from v4. It passed all eight report comparisons and the per-class default/all semantic checks. It bound each CLI binary by its observed SHA-256 and validated report stdout captures against command records.

This only establishes report-level equivalence and expected projections for the captured rows. The v4 execution has a later JDK manifest shape failure (`legs` is a list where the runner expected a mapping), so full runtime/dual-JDK acceptance remains pending v5. No CLI, JDK, Rust, Cargo, or Git command was run here.
