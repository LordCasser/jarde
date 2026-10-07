#!/bin/sh
# The gating experiment (`recover-boolean-int-bitwise-operands`): the parent commit's binary and this
# change's render the same three classes, and the two texts are diffed.
#
# Self-test first: the baseline must show the two refusals on the patrol's frozen `BW.class` (a
# baseline that already recovered them would make every later comparison meaningless), and the
# patched binary must show none.
set -eu

ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11405-eb09-78f1-babc-dbe140bcaad2
BASE=/tmp/jarde-baseline-target/debug/jarde-cli
PATCHED=$ROOT/target/debug/jarde-cli
FIX=$ROOT/tests/fixtures/recover-boolean-int-bitwise-operands
PATROL=$ROOT/openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.class
WORK=${1:-/tmp/boolean-int-bitwise/gating}
rm -rf "$WORK"
mkdir -p "$WORK"

render() { # binary class-file class-name out
    "$1" class-source --policy single-class --input "$2" --class "$3" --format text >"$4" 2>/dev/null
    head -1 "$4" | grep -q '// jarde: presentation of' || { echo "NO SELF-HEADER: $2 on $1"; exit 1; }
}

render "$BASE" "$PATROL" BW "$WORK/base-BW.txt"
render "$PATCHED" "$PATROL" BW "$WORK/pat-BW.txt"
base_refusals=$(grep -c 'which no Java integral or boolean bitwise expression accepts' "$WORK/base-BW.txt" || true)
patched_refusals=$(grep -c 'which no Java integral or boolean bitwise expression accepts' "$WORK/pat-BW.txt" || true)
if [ "$base_refusals" -ne 2 ] || [ "$patched_refusals" -ne 0 ]; then
    echo "SELF-TEST FAILED: BW refusals base=$base_refusals patched=$patched_refusals (want 2 then 0)"
    exit 1
fi

for class in BW BWR BWN; do
    for leg in v8 v8-javac8; do
        [ "$class" = BW ] && [ "$leg" = v8 ] && continue
        render "$BASE" "$FIX/$leg/$class.class" "$class" "$WORK/base-$class-$leg.txt"
        render "$PATCHED" "$FIX/$leg/$class.class" "$class" "$WORK/pat-$class-$leg.txt"
    done
done

{
    echo "=== the change's own gating: the anchors and the negatives, baseline -> patched"
    echo
    echo "--- the patrol's frozen BW.class: refusals"
    echo "baseline: $base_refusals"
    echo "patched:  $patched_refusals"
    echo
    echo "--- BW diff"
    diff "$WORK/base-BW.txt" "$WORK/pat-BW.txt" || true
    for class in BWR BWN; do
        for leg in v8 v8-javac8; do
            echo
            echo "--- $class ($leg) diff"
            diff "$WORK/base-$class-$leg.txt" "$WORK/pat-$class-$leg.txt" || true
            echo "(end of $class $leg diff)"
        done
    done
} >"$WORK/gating-experiment.txt"
echo "GATING OK: BW refusals $base_refusals -> $patched_refusals; transcript in $WORK/gating-experiment.txt"
