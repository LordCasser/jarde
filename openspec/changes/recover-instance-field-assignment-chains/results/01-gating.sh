#!/bin/sh
# Task 1.1's gating experiment: the instance form's admission against the anchors it must not move.
#
# For every named anchor (the static chain, the compound receiver copies, the concatenation
# receiver copy, this change's own CP and its negatives) this renders the class with both binaries —
# the parent commit's (`/tmp/jarde-base-target`) and the patched one — and diffs the two texts.
# `CP` must move (its chain is the one admission this change makes); every other anchor must be
# byte-identical.
set -eu

BASE=${BASE:-/tmp/jarde-base-target/debug/jarde-cli}
PATCHED=${PATCHED:-/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a117e3-4f68-7671-9841-01ed39785e09/target/debug/jarde-cli}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
OUT=$(cd "$(dirname "$0")/renders" && pwd)
ANCHORS=$OUT/../01-anchors.txt

render() {
    local binary=$1
    local policy=$2
    local path=$3
    local name=$4
    local out=$5
    "$binary" class-source --policy "$policy" --input "$path" --class "$name" \
        --format text >"$out" 2>"$out.err" || true
    head -1 "$out" | grep -q '// jarde: presentation of' || {
        echo "UNRENDERED: $path under $name"
        return 1
    }
}

cat >"$ANCHORS" <<EOF
CH single-class $ROOT/openspec/evidence/java-syntax-2026-10-05/chained-assign-sideeffect-patrol/fixture/CH.class CH
SC plain-jar $ROOT/openspec/evidence/java-syntax-2026-10-05/field-string-compound-patrol/fixture/sc.jar SC
BF plain-jar $ROOT/openspec/evidence/java-syntax-2026-10-05/field-compound-soundness-patrol/fixture/bf.jar BF
BG plain-jar $ROOT/openspec/evidence/java-syntax-2026-10-05/field-compound-soundness-patrol/fixture/bg.jar BG
CA2 plain-jar $ROOT/openspec/evidence/java-syntax-2026-10-05/assign-chain-soundness-patrol/fixture/ca2.jar CA2
CF single-class $ROOT/tests/fixtures/recover-chained-field-assignment/v8/CF.class CF
CP single-class $ROOT/tests/fixtures/recover-instance-field-assignment-chains/v8/CP.class CP
MX single-class $ROOT/tests/fixtures/recover-instance-field-assignment-chains/v8/MX.class MX
NEG single-class $ROOT/tests/fixtures/recover-instance-field-assignment-chains/v8/NEG.class NEG
EOF

while read -r label policy input class; do
    [ -n "$label" ] || continue
    render "$BASE" "$policy" "$input" "$class" "$OUT/$label-before.txt"
    render "$PATCHED" "$policy" "$input" "$class" "$OUT/$label-after.txt"
    if diff -q "$OUT/$label-before.txt" "$OUT/$label-after.txt" >/dev/null; then
        echo "IDENTICAL: $label"
    else
        lines=$(diff "$OUT/$label-before.txt" "$OUT/$label-after.txt" | grep -c '^[<>]' || true)
        echo "MOVED:     $label ($lines differing lines)"
    fi
done <"$ANCHORS"
