#!/bin/sh
# The slice's gating experiment: render every anchor, control and negative with the baseline binary
# (the parent commit, `c99e26a5`) and with the current one, and compare the two byte for byte.
#
# The claims this measures:
#
#   * the row-set resource guard flips the io-wrapping anchor's `countLines` — the patrol's jar and
#     both fixture legs move together — and the same shape's probes with it
#     (`LockGuardProbe.countLines`, `ScopePlanCrossing.resourceAcrossFinally`, `IOMidRead`);
#   * the lock guard's own three methods (`LK`) stay byte-identical on all three of their legs, and
#     the two-copy non-lock `finally` shapes (`Guarded`) stay where they were;
#   * the registered boundaries stay refused verbatim: `IO.readAll`, the two `IONegatives` members,
#     the CF-16 void-loop fixture (a two-row shape my certificate refuses) and the lock-guard
#     negatives;
#   * the `new@1` depth boundary moves: `NestedDepth.threeLayer` and the patrol's `X4.threeLayer`
#     present, `NestedDepth.fourLayer` keeps its refusal;
#   * the two `java.io` widening rows present their own argument positions (`WideningProbe`).
#
# Usage: 02-gating.sh [baseline-cli] [current-cli]
#   defaults: /tmp/io-baseline-target/debug/jarde-cli and <worktree>/target/debug/jarde-cli
#
# Self-test before counting: every render must start with the layer's own header, or the script
# stops. Output: `02-gating.out` (the table) and `gating/<kind>.<binary>.java` (the renders), both
# beside this script's directory.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BASELINE=${1:-/tmp/io-baseline-target/debug/jarde-cli}
CURRENT=${2:-$ROOT/target/debug/jarde-cli}
OUT="$HERE/02-gating.out"
RENDERS="$HERE/gating"

[ -x "$BASELINE" ] || { echo "no baseline binary at $BASELINE" >&2; exit 2; }
[ -x "$CURRENT" ] || { echo "no current binary at $CURRENT" >&2; exit 2; }
mkdir -p "$RENDERS"

render() {
    # $1 = binary, $2 = input, $3 = policy, $4 = class
    "$1" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null
}

{
    echo "# The gating experiment (baseline = the parent commit c99e26a5, current = this slice)"
    echo "# columns: input | baseline render sha256 | current render sha256 | verdict"
    echo "input|baseline|current|verdict"
} > "$OUT"
for spec in \
    "io-patrol-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar|plain-jar|IO" \
    "io-v8|$ROOT/tests/fixtures/recover-io-resource-finally/v8/IO.class|single-class|IO" \
    "io-v8-javac8|$ROOT/tests/fixtures/recover-io-resource-finally/v8-javac8/IO.class|single-class|IO" \
    "mid-v8|$ROOT/tests/fixtures/recover-io-resource-finally/v8/IOMidRead.class|single-class|IOMidRead" \
    "mid-v8-javac8|$ROOT/tests/fixtures/recover-io-resource-finally/v8-javac8/IOMidRead.class|single-class|IOMidRead" \
    "negatives-v8|$ROOT/tests/fixtures/recover-io-resource-finally/v8/IONegatives.class|single-class|IONegatives" \
    "negatives-v8-javac8|$ROOT/tests/fixtures/recover-io-resource-finally/v8-javac8/IONegatives.class|single-class|IONegatives" \
    "depth-v8|$ROOT/tests/fixtures/recover-io-resource-finally/v8/NestedDepth.class|single-class|NestedDepth" \
    "depth-v8-javac8|$ROOT/tests/fixtures/recover-io-resource-finally/v8-javac8/NestedDepth.class|single-class|NestedDepth" \
    "widening-v8|$ROOT/tests/fixtures/recover-io-resource-finally/v8/WideningProbe.class|single-class|WideningProbe" \
    "widening-v8-javac8|$ROOT/tests/fixtures/recover-io-resource-finally/v8-javac8/WideningProbe.class|single-class|WideningProbe" \
    "lk-patrol-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar|plain-jar|LK" \
    "lk-v8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LK.class|single-class|LK" \
    "lk-v8-javac8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LK.class|single-class|LK" \
    "lk-negatives-v8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LockGuardNegatives.class|single-class|LockGuardNegatives" \
    "lk-negatives-v8-javac8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LockGuardNegatives.class|single-class|LockGuardNegatives" \
    "lk-probe-v8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LockGuardProbe.class|single-class|LockGuardProbe" \
    "lk-probe-v8-javac8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LockGuardProbe.class|single-class|LockGuardProbe" \
    "local-scope-crossing|$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlanCrossing.class|single-class|ScopePlanCrossing" \
    "local-scope-crossing-javac8|$ROOT/tests/fixtures/preserve-local-scope-plan/v8-javac8/ScopePlanCrossing.class|single-class|ScopePlanCrossing" \
    "p3-handlers|$ROOT/tests/fixtures/p3-handlers/v8/Guarded.class|single-class|Guarded" \
    "void-loop-fixed|$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/fixed/TestTryCatchFinally2\$TestCls.class|single-class|jadx.tests.integration.trycatch.TestTryCatchFinally2\$TestCls" \
    "nested-ctor-x4|$ROOT/openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X4.class|single-class|X4" \
    "nested-ctor-x3|$ROOT/openspec/evidence/java-syntax-2026-10-02/nested-ctor-argument-patrol/variants-nested/X3.class|single-class|X3"
do
    kind=${spec%%|*}
    rest=${spec#*|}
    input=${rest%%|*}
    rest=${rest#*|}
    policy=${rest%%|*}
    class=${rest#*|}
    baseline_text=$(render "$BASELINE" "$input" "$policy" "$class")
    current_text=$(render "$CURRENT" "$input" "$policy" "$class")
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
