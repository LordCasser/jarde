#!/bin/sh
# Task 1.1's gating experiment: the branch arm alone, measured against HEAD.
#
# The claim this measures: adding the branch arm to `proves_boolean_local_store`'s consumer
# whitelist — and nothing else — flips the ternary and statement condition positions (OP2's
# `condAssignOld`, `BranchReads.ternaryRead`/`ifStatement`/`midChain`, the `ifne` control) and
# moves nothing else: the loop condition position (refused by the same gate's cross-region
# criterion *before* the whitelist), the catch-crossing form (refused before the gate, at region
# ownership), the numeric-branch control, the three existing consumer positions (putstatic-Z,
# boolean-`ireturn`, `append(Z)`) and the whole short-circuit family's own fixtures keep their
# answers byte for byte.
#
# Both binaries are built from this worktree: the baseline from the parent commit's gate (this
# worktree is at it), the current one from the same source plus the twelve-line arm.
#
# Usage: 01-gating.sh [baseline-cli] [current-cli]
#   defaults: /tmp/brslice/bin/jarde-cli.baseline and <worktree>/target/debug/jarde-cli
#
# Self-test before counting: every render must start with the layer's own header, or the script
# stops. Output: `01-gating.out` (the table) and `gating/<kind>.<binary>.java` (the renders).

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BASELINE=${1:-/tmp/brslice/bin/jarde-cli.baseline}
CURRENT=${2:-$ROOT/target/debug/jarde-cli}
OUT="$HERE/01-gating.out"
RENDERS="$HERE/gating"
FIXTURES="$ROOT/tests/fixtures"

[ -x "$BASELINE" ] || { echo "no baseline binary at $BASELINE" >&2; exit 2; }
[ -x "$CURRENT" ] || { echo "no current binary at $CURRENT" >&2; exit 2; }
mkdir -p "$RENDERS"

render() {
    # $1 = binary, $2 = input, $3 = policy, $4 = class
    "$1" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null
}

{
    echo "# The gating experiment (baseline = the parent commit's gate, current = + the branch arm)"
    echo "# columns: input | baseline render sha256 | current render sha256 | verdict"
    echo "input|baseline|current|verdict"
} > "$OUT"
for spec in \
    "op2-patrol-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar|plain-jar|OP2" \
    "branch-reads-v8|$FIXTURES/recover-short-circuit-local-branch-reads/v8/BranchReads.class|single-class|BranchReads" \
    "branch-reads-v8-javac8|$FIXTURES/recover-short-circuit-local-branch-reads/v8-javac8/BranchReads.class|single-class|BranchReads" \
    "branch-read-negatives-v8|$FIXTURES/recover-short-circuit-local-branch-reads/v8/BranchReadNegatives.class|single-class|BranchReadNegatives" \
    "branch-read-negatives-v8-javac8|$FIXTURES/recover-short-circuit-local-branch-reads/v8-javac8/BranchReadNegatives.class|single-class|BranchReadNegatives" \
    "control-numeric-branch|$FIXTURES/recover-short-circuit-local-branch-reads/controls/NumericBranch.class|single-class|BranchReads" \
    "control-not-zero-branch|$FIXTURES/recover-short-circuit-local-branch-reads/controls/NotZeroBranch.class|single-class|BranchReads" \
    "anchor-putstatic-ireturn-z|$FIXTURES/p3-conditional-values/mixed-short-circuit-local/MixedBooleanLocal.class|single-class|MixedBooleanLocal" \
    "anchor-append-z|$FIXTURES/p3-conditional-values/scv-concat-consumers/ScvConcatConsumers.class|single-class|ScvConcatConsumers" \
    "scv-concat-controls|$FIXTURES/p3-conditional-values/scv-concat-consumers/ScvConcatReReadControls.class|single-class|ScvConcatReReadControls" \
    "scv-local-controls|$FIXTURES/p3-conditional-values/mixed-short-circuit-local-controls/MixedLocalControls.class|single-class|MixedLocalControls" \
    "scv-local-short-circuit|$FIXTURES/p3-conditional-values/short-circuit-local/LocalShortCircuit.class|single-class|LocalShortCircuit" \
    "scv-shared-true|$FIXTURES/p3-conditional-values/short-circuit-shared-true/SharedTrueShortCircuit.class|single-class|SharedTrueShortCircuit" \
    "scv-shared-true-controls|$FIXTURES/p3-conditional-values/short-circuit-shared-true-controls/SharedTrueDuplicatePhi.class|single-class|SharedTrueDuplicatePhi" \
    "scv-shared-true-non-boolean|$FIXTURES/p3-conditional-values/short-circuit-shared-true-controls/SharedTrueNonBoolean.class|single-class|SharedTrueShortCircuit" \
    "scv-chain-shared-true|$FIXTURES/p3-conditional-values/short-circuit-chain-shared-true/ChainOrField.class|single-class|ChainOrField" \
    "scv-chain-shared-true-controls|$FIXTURES/p3-conditional-values/short-circuit-chain-shared-true-controls/ChainOrFieldDuplicatePhi.class|single-class|ChainOrFieldDuplicatePhi" \
    "scv-exception-chain|$FIXTURES/p3-conditional-values/short-circuit-exception-chain/ExceptionChain.class|single-class|ExceptionChain" \
    "scv-exception-edge|$FIXTURES/p3-conditional-values/short-circuit-exception-edge/ExceptionShortCircuit.class|single-class|ExceptionShortCircuit" \
    "scv-transfer-gateway-controls|$FIXTURES/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundary.class|single-class|ChainExtraBoundary" \
    "scv-transfer-gateway-exception|$FIXTURES/p3-conditional-values/short-circuit-transfer-gateway-controls/ChainExtraBoundaryException.class|single-class|ChainExtraBoundary" \
    "scv-mixed-argument|$FIXTURES/p3-conditional-values/mixed-short-circuit-argument/MixedBooleanArgument.class|single-class|MixedBooleanArgument" \
    "scv-mixed-argument-controls|$FIXTURES/p3-conditional-values/mixed-short-circuit-argument-controls/MixedArgumentControls.class|single-class|MixedArgumentControls" \
    "scv-mixed-field|$FIXTURES/p3-conditional-values/mixed-short-circuit-field/MixedBooleanField.class|single-class|MixedBooleanField" \
    "scv-mixed-instance-field|$FIXTURES/p3-conditional-values/mixed-short-circuit-instance-field/MixedShortCircuitField.class|single-class|MixedShortCircuitField" \
    "scv-mixed-instance-controls|$FIXTURES/p3-conditional-values/mixed-short-circuit-instance-field/MixedInstanceControls.class|single-class|MixedInstanceControls" \
    "scv-mixed-int-return|$FIXTURES/p3-conditional-values/mixed-short-circuit-int-return/MixedIntReturn.class|single-class|MixedIntReturn" \
    "scv-mixed-local-return|$FIXTURES/p3-conditional-values/mixed-short-circuit-return/MixedLocalReturn.class|single-class|MixedLocalReturn" \
    "p3-boolean-short-circuit-return|$FIXTURES/p3-boolean-short-circuit-return/v8/BoolValue.class|single-class|BoolValue" \
    "p3-boolean-contexts|$FIXTURES/p3-boolean-contexts/v8/BooleanContexts.class|single-class|BooleanContexts" \
    "p3-hoisted-boolean|$FIXTURES/p3-hoisted-boolean/v8/HoistedBoolean.class|single-class|HoistedBoolean" \
    "p3-loop-boolean-exit|$FIXTURES/p3-loop-boolean-exit/v8/LoopBool.class|single-class|LoopBool" \
    "p3-loop-boolean-do|$FIXTURES/proved-java-structure/loop-boolean-do/DoLoopBool.class|single-class|DoLoopBool" \
    "p3-short-circuit-left-false|$FIXTURES/proved-java-structure/short-circuit-left-false/ShortCircuitFalse.class|single-class|ShortCircuitFalse" \
    "control-local-scope-crossing|$FIXTURES/preserve-local-scope-plan/v8/ScopePlanCrossing.class|single-class|ScopePlanCrossing" \
    "control-local-scope-positive|$FIXTURES/preserve-local-scope-plan/v8/ScopePlan.class|single-class|ScopePlan" \
    "control-p3-handlers|$FIXTURES/p3-handlers/v8/Guarded.class|single-class|Guarded"
do
    kind=${spec%%|*}
    rest=${spec#*|}
    input=${rest%%|*}
    rest=${rest#*|}
    policy=${rest%%|*}
    class=${rest#*|}
    baseline_text=$(render "$BASELINE" "$input" "$policy" "$class" || true)
    current_text=$(render "$CURRENT" "$input" "$policy" "$class" || true)
    case "$baseline_text" in
        "// jarde: presentation of"*) ;;
        *) echo "SELF-TEST FAILED: baseline render of $kind has no header" >&2; exit 3 ;;
    esac
    case "$current_text" in
        "// jarde: presentation of"*) ;;
        *) echo "SELF-TEST FAILED: current render of $kind has no header" >&2; exit 3 ;;
    esac
    printf '%s\n' "$baseline_text" > "$RENDERS/$kind.baseline.java"
    printf '%s\n' "$current_text" > "$RENDERS/$kind.current.java"
    baseline_digest=$(printf '%s' "$baseline_text" | shasum -a 256 | cut -d' ' -f1)
    current_digest=$(printf '%s' "$current_text" | shasum -a 256 | cut -d' ' -f1)
    verdict=identical
    [ "$baseline_digest" = "$current_digest" ] || verdict=moved
    printf '%s|%s|%s|%s\n' "$kind" "$baseline_digest" "$current_digest" "$verdict" >> "$OUT"
done
cat "$OUT"
