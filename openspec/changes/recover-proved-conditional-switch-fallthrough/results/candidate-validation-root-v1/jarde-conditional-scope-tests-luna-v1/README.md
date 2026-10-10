# Conditional switch scope test draft

`tests/p3_conditional_switch_scope_boundaries.rs` is a private draft for the public reader/recovery API. The embedded 1,494-byte class was copied from `boundary-baseline-root-v1/.../cases/javac8/original/classes/ConditionalSwitchBoundaries.class`; its recorded SHA-256 is `27dea02e6dc3003a822ef7db2f447a6d21a508d1fb83c83622f86c64f4ff1b8b`, and the public-IR record pins its BLAKE3 as `49795c66605066655f48ac37b862ccd132e43b74d290653afd51762ab1a409b7`.

The test reuses the real `ArtifactSnapshot` → `analyze_method_ir` → `RecoveryRequest` path from the existing physical-boundary draft. It checks the recorded javac 8 graph sizes, the observed loop back-edge, the three actual exception edges into handler entry 60, physical block identities, and the terminal method's `areturn`/`athrow` instructions. It deliberately prints the four report outcomes and region summaries without claiming that any candidate was admitted or refused.

After the applied implementation is run, add only observations confirmed by that run:

- `innerLoopBreak`: inspect direct source-map origins of emitted breaks against the actual loop/switch nesting. The loop back-edge is BCI 57 → 38. `build.rs` captures `loop_headers.len()` and switch depth when it pushes each active switch target. A `SwitchBreak` is accepted only when it matches the top target's branch BCI and join and both recorded depths still match. A `LoopBreak` resolves its exact loop header with the innermost matching loop position; when it crosses a switch it receives the loop label.
- `innerSwitchBreak`: inspect the inner switch break origins and verify they stay scoped to the inner switch; also inspect any outer conditional exit using its real source BCI and nearest switch target. The run must supply the exact region/segment assertions.
- `terminalCase`: inspect the all-evidence source-map spans for the physical return and throw BCIs and pin the exact text and origins from the run. The draft checks only that the physical `areturn` and `athrow` exist.
- `caughtExceptionThenFallthrough`: retain the three observed `Exception` edges `(40, 60)`, `(45, 60)`, and `(47, 60)`; the baseline record identifies handler ordinal 0. Inspect the generated try/catch and fallthrough spans before adding exact outcome assertions.

The canonical summaries come from `boundary-public-ir-root-v1/README.md`: inner loop 10 blocks/13 Normal edges, inner switch 8/10, terminal 6/6, caught method 9 blocks/14 total edges with three exception edges. The complete class's baseline source output is in `boundary-baseline-root-v1/.../jarde-default/ConditionalSwitchBoundaries.java`. Its refusal reasons describe the pre-candidate run and must not be copied as expected candidate reasons.

No repository file or toolchain was touched while preparing this draft.
