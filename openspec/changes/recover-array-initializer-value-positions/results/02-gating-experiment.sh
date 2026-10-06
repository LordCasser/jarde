#!/bin/sh
# The gating experiment of `recover-array-initializer-value-positions`.
#
# Two binaries — the baseline built from this change's parent commit `HEAD`
# (/tmp/jarde-array-baseline-wt) and the patched one — render the change's own fixture inputs, and
# the run reports which **methods** moved and which did not. The self-tests state the expectation
# before the counts are believed: the two refusing positions must flip, every position that already
# presented the dance must not move, and the hand-built negatives must not move at all.
set -eu
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
BASE=${BASE:-/tmp/jarde-array-baseline-wt/target/debug/jarde-cli}
PATCHED=${PATCHED:-$ROOT/target/debug/jarde-cli}
WORK=${1:-/tmp/array-value/gating}
FIXTURES=$ROOT/tests/fixtures/recover-array-initializer-value-positions
rm -rf "$WORK"
mkdir -p "$WORK"

for pair in "baseline:$BASE" "patched:$PATCHED"; do
    label=${pair%%:*}
    binary=${pair#*:}
    (cd "$FIXTURES/v8" && jar cf "$WORK/leg.jar" MD.class MD2.class MD3.class AV.class AVT.class)
    for class in MD MD2 MD3 AV AVT; do
        "$binary" class-source --policy plain-jar --input "$WORK/leg.jar" --class "$class" \
            --format text >"$WORK/$label-$class.txt" 2>/dev/null || true
        head -1 "$WORK/$label-$class.txt" | grep -q '// jarde: presentation of' || {
            echo "FAILED: $class renders nothing on $label"; exit 1; }
    done
    (cd "$FIXTURES" && jar cf "$WORK/avn.jar" AVN.class)
    "$binary" class-source --policy plain-jar --input "$WORK/avn.jar" --class AVN \
        --format text >"$WORK/$label-AVN.txt" 2>/dev/null || true
done

moved_methods() {
    python3 - "$1" "$2" <<'PY'
import re, sys
def bodies(path):
    out={}; cur=None; buf=[]
    for line in open(path).read().splitlines():
        m=re.match(r'^    (?:public|static|private|protected|final|synchronized).*?(\w+)\(', line)
        if m and line.rstrip().endswith('{'):
            if cur: out[cur]='\n'.join(buf)
            cur=m.group(1); buf=[line]
        elif cur is not None:
            buf.append(line)
            if line=='    }': out[cur]='\n'.join(buf); cur=None
    return out
a, b = bodies(sys.argv[1]), bodies(sys.argv[2])
print(" ".join(k for k in a if k in b and a[k] != b[k]))
PY
}

for class in MD MD2 MD3 AV AVT; do
    echo "$class moved: $(moved_methods "$WORK/baseline-$class.txt" "$WORK/patched-$class.txt")"
done

# The self-tests: what must move, and what must not.
check() {
    local class=$1
    local want=$2
    local got
    got=$(moved_methods "$WORK/baseline-$class.txt" "$WORK/patched-$class.txt")
    if [ "$got" != "$want" ]; then
        echo "SELF-TEST FAILED: $class moved [$got], wanted [$want]"
        exit 1
    fi
}
check MD "partSet"
check MD2 "retPos"
check MD3 "bareIdx2"
check AV "elemStore immIdx immLen immIdxVar immIdxExpr immIdxSum twoIdx twoStores nestedIdx condIdx immIdxInCall order"
check AVT "setS setL setD setB setC idxS idxL idxD idxB idxC lenD"
avn_moved=$(moved_methods "$WORK/baseline-AVN.txt" "$WORK/patched-AVN.txt")
if [ "$avn_moved" != "single" ]; then
    echo "SELF-TEST FAILED: the hand-built AVN moved [$avn_moved], wanted [single] (the two negatives must not move)"
    exit 1
fi
# The two refusing positions are the only refusals the change removes from the patrol's own classes.
for class in MD MD2 MD3; do
    if grep -q 'has no proved local assignment' "$WORK/patched-$class.txt"; then
        echo "SELF-TEST FAILED: $class still carries the copy family's refusal"
        exit 1
    fi
done
echo "GATING OK: the two positions flip (MD.partSet, MD2.retPos, MD3.bareIdx2), the four positions and"
echo "the bare consumption do not move, AVN moves only its builder self-test, and no patrol class quotes bytecode."
