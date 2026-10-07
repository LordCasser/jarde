#!/bin/sh
# 1.2/1.3 re-verification: the anchors 12/13 the planning slice pinned as refused are re-rendered
# with this worktree's CLI, and the two legs of the declaration-plan fixture are compared.
#
# Self-test before counting: every render must start with the layer's own header, or the script
# stops. Usage: 01-reverify-1.2-1.3.sh [cli]
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../../.." && pwd)
CLI=${1:-$ROOT/target/debug/jarde-cli}
OUT="$HERE/01-reverify-1.2-1.3.out"
RENDERS="$HERE/renders"
[ -x "$CLI" ] || { echo "no CLI at $CLI" >&2; exit 2; }
mkdir -p "$RENDERS"

render() {
    "$CLI" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null
}

{
    echo "# anchor/leg | render sha256 | verdict"
    echo "input|sha256"
} > "$OUT"
for spec in \
    "anchor12-patrol-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar|plain-jar|LK" \
    "anchor12-v8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LK.class|single-class|LK" \
    "anchor12-v8-javac8|$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LK.class|single-class|LK" \
    "anchor13-patrol-jar|$ROOT/openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar|plain-jar|IO" \
    "anchor13-v8|$ROOT/tests/fixtures/recover-io-resource-finally/v8/IO.class|single-class|IO" \
    "anchor13-v8-javac8|$ROOT/tests/fixtures/recover-io-resource-finally/v8-javac8/IO.class|single-class|IO" \
    "plan-positive-v8|$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlan.class|single-class|ScopePlan" \
    "plan-positive-v8-javac8|$ROOT/tests/fixtures/preserve-local-scope-plan/v8-javac8/ScopePlan.class|single-class|ScopePlan" \
    "plan-crossing-v8|$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlanCrossing.class|single-class|ScopePlanCrossing" \
    "plan-crossing-v8-javac8|$ROOT/tests/fixtures/preserve-local-scope-plan/v8-javac8/ScopePlanCrossing.class|single-class|ScopePlanCrossing" \
    "refusals-v8|$ROOT/tests/fixtures/preserve-local-scope-refusals/v8/ScopeRefusals.class|single-class|ScopeRefusals" \
    "refusals-v8-javac8|$ROOT/tests/fixtures/preserve-local-scope-refusals/v8-javac8/ScopeRefusals.class|single-class|ScopeRefusals"
do
    kind=${spec%%|*}
    rest=${spec#*|}
    input=${rest%%|*}
    rest=${rest#*|}
    policy=${rest%%|*}
    class=${rest#*|}
    text=$(render "$CLI" "$input" "$policy" "$class")
    case "$text" in
        "// jarde: presentation of"*) ;;
        *) echo "SELF-TEST FAILED: $kind has no header" >&2; exit 3 ;;
    esac
    printf '%s\n' "$text" > "$RENDERS/$kind.java"
    digest=$(printf '%s' "$text" | shasum -a 256 | cut -d' ' -f1)
    printf '%s|%s\n' "$kind" "$digest" >> "$OUT"
done
cat "$OUT"
