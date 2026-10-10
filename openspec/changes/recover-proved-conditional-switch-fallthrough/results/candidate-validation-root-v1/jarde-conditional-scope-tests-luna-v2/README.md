# Conditional switch scope tests

This private v2 draft is ready for root to copy into the repository tests. Its fixture is the complete javac 8 `ConditionalSwitchBoundaries.class` at `tests/fixtures/p3-conditional-switch-boundaries/javac8/original/ConditionalSwitchBoundaries.class`. The recorded SHA-256 is `27dea02e6dc3003a822ef7db2f447a6d21a508d1fb83c83622f86c64f4ff1b8b`; the test pins the runtime BLAKE3 identity `49795c66605066655f48ac37b862ccd132e43b74d290653afd51762ab1a409b7` and the 1,494-byte size.

The test uses the real `ArtifactSnapshot` → `analyze_method_ir` → `RecoveryRequest` → `recover` path. It pins all four methods' canonical graph sizes and pathless physical block identities, the `innerLoopBreak` back-edge 57 → 38, and the three caught-method Exception edges from 40, 45, and 47 to handler entry 60 with handler ordinal 0.

The three boundary outcomes are the ones observed in root's applied-candidate run:

- `innerLoopBreak`: `ExplanationOnly`, with `jre_region_uncovered_blocks` at BCI 38 owning `[38, 43, 63, 54, 57]`.
- `innerSwitchBreak`: `ExplanationOnly`, with `jre_region_arms_do_not_meet` at BCI 0.
- `caughtExceptionThenFallthrough`: `ExplanationOnly`, with `jre_region_uncovered_blocks` at BCI 47 owning `[47]`.

These are earlier region/normal-flow refusals. They do not show that `Walker`'s nearest-switch `branch_bci`/`join` or loop/switch-depth guard rejected a `SwitchBreak`: the observed region tree did not hand those paths to the Builder break visitor. The tests assert only that these runs emitted no `break;` statement. That prevents a false unlabeled break while keeping the claimed evidence at the stage the observer actually reached.

`terminalCase` remains fully structured. The test pins the complete observed body under essential/default, all-evidence, and explicit source-map evidence requests; it checks the three physical terminators (`areturn` at 47 and 88, `athrow` at 66), direct source-map spans for the exact return/throw statements, and source-map coverage for every decoded physical instruction BCI. It does not infer any scope-guard behavior from this method.

The draft is uncompiled by request. Root owns toolchain execution and any resulting adjustment to source-map span assertions.
