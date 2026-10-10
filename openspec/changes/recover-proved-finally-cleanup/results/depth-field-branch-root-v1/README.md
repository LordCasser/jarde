# CF16 field-entry nested-branch probe preparation

This probe copies the depth-branch baseline's complete class, Runner, pinned toolchain checks,
and observation flow. Its only bytecode-shape change is the outermost condition:
`if (trace + x > 1)`. That reads the static `trace` field before the argument and uses the same
field-read-at-protected-entry shape as the existing `ImplicitCleanup` path. The remaining nested
conditions still use `x > depthIndex`; both depths still run `run(0)` and `run(40)` with trace
reset between calls, so the expected outputs remain unchanged.

The purpose is to enter the existing finally-recovery route whose guard inspection needs a leaving
exception edge before asking whether the protected region is a finally. The earlier pure
argument/compare entry reached Produced/Fallback at both depths. This probe records what the
frozen CLI actually reports; it does not assume that depth 33 reaches a hard depth bound.

The preparation script verifies the same pinned returned-array manifest, CLI v2 identity, and
both JDK binary hashes; it compiles and verifies only the original complete classes and Runner,
then saves fresh `class-source all` JSON and all raw streams. It does not compile or execute
recovered text. No toolchain command was run while preparing these files.

After review, prepare observations with:

```sh
python3 openspec/changes/recover-proved-finally-cleanup/results/depth-field-branch-root-v1/prepare-depth-field-branch-root-v1.py
```
