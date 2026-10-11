# recover-proved-switch-continuations

## Why

The frozen CF12 evidence now covers all ten upstream switch fixtures, but two real methods still expose one shared structural gap: `TestSwitchWithFallThroughCase2.test(IZZ)Ljava/lang/String;` refuses the switch tail between its internal join 175 and the outer-if join 197, while `TestSwitch2.test(I)V` refuses a switch whose paths either reach a shared continuation at BCI 164 or terminate in early returns. The accepted remaining-five replay records both as Jarde compile failures. The whole-class observations are inputs for this change, not proof that either structure is safe to emit.

## What Changes

- After first observing each method's actual rejection path, extend the existing switch continuation consumer for a fully proved switch followed by a bounded straight tail in its enclosing arm.
- Extend the existing forward-join proof only for a unique, bounded finite DAG whose complete paths reach one candidate join or a physically proved return/throw terminal; allow the candidate to be a direct case target only when canonical ownership proves it.
- Keep structured output, ownership, canonical edge closure, source origins, and budget/Stop behavior tied to the existing Java 8 recovery pipeline. Unsupported shapes remain conservative.
- Verify both structures against the frozen full-class inputs and then run one complete, newly frozen comparison across all ten CF12 fixtures.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `java8-recovery`: recover only the two bounded switch continuation shapes after proof of their physical paths and ownership.

## Impact

The implementation is expected to remain local to the existing `jarde-java` switch/region recovery in `region.rs` and its private tests. It reuses the current `Region`, sequence, switch-arm, terminal, canonical CFG, and budget mechanisms. It adds no Region variant, public API, graph framework, JVM IR/SSA type change, or cross-class constant-name behavior.

This work starts only after the conditional-switch change's own CI acceptance and clean delivery are closed. The root owns product application, full frozen-CLI validation, CI, and clean delivery. Two independent Luna agents may prepare private implementation candidates for the two shapes; the root reviews and merges them serially into the same `region.rs`, without branches or worktrees.

The existing frozen Jarde failures are not positive examples. JADX's TestSwitch4 failure (`2234` instead of `1234`) and its FallThroughCase2 duplicated-code warning with matching runtime behavior are comparison controls; neither is a completion claim for this change or for the 71-unit ledger. The nested constant-name gate remains separately queued.
