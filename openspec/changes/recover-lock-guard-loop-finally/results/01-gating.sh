#!/bin/sh
# The slice's gating experiment: render the anchor and every control with the baseline binary (the
# parent commit) and with the current one, and compare the two byte for byte.
#
# The claim this measures: the certificate alone flips the anchor's three members, and nothing else
# moves — the negatives, the io-wrapping probe and the whole committed fixture corpus keep their
# answers. Both binaries are built from the same checkout family: the baseline from a detached
# worktree at the parent commit, the current one from this worktree.
#
# Usage: 01-gating.sh [baseline-cli] [current-cli]
#   defaults: /tmp/lk/baseline-target/debug/jarde-cli and <worktree>/target/debug/jarde-cli
#
# Self-test before counting: every render must start with the layer's own header, or the script
# stops. Output: `01-gating.out` (the table) and `gating/<kind>.<binary>.java` (the renders), both
# beside this script's directory.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BASELINE=${1:-/tmp/lk/baseline-target/debug/jarde-cli}
CURRENT=${2:-$ROOT/target/debug/jarde-cli}
OUT="$HERE/01-gating.out"
RENDERS="$HERE/gating"

[ -x "$BASELINE" ] || { echo "no baseline binary at $BASELINE" >&2; exit 2; }
[ -x "$CURRENT" ] || { echo "no current binary at $CURRENT" >&2; exit 2; }
mkdir -p "$RENDERS"

render() {
    # $1 = binary, $2 = input, $3 = policy, $4 = class
    "$1" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null
}

{
    echo "# The gating experiment (baseline = the parent commit, current = this slice)"
    echo "# columns: input | baseline render sha256 | current render sha256 | verdict"
    echo "input|baseline|current|verdict"
} > "$OUT"
for spec in \
    "patrol-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar|plain-jar|LK" \
    "v8-LK|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LK.class|single-class|LK" \
    "v8-javac8-LK|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LK.class|single-class|LK" \
    "v8-negatives|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LockGuardNegatives.class|single-class|LockGuardNegatives" \
    "v8-javac8-negatives|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LockGuardNegatives.class|single-class|LockGuardNegatives" \
    "v8-probe|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LockGuardProbe.class|single-class|LockGuardProbe" \
    "v8-javac8-probe|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LockGuardProbe.class|single-class|LockGuardProbe" \
    "p3-handlers|$ROOT/tests/fixtures/p3-handlers/v8/Guarded.class|single-class|Guarded" \
    "local-scope-crossing|$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlanCrossing.class|single-class|ScopePlanCrossing" \
    "local-scope-positive|$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlan.class|single-class|ScopePlan" \
    "io-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar|plain-jar|IO"
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
