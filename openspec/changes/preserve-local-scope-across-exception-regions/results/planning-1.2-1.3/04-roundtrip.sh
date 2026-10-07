#!/bin/sh
# The fixture's acceptance roundtrip: the class the positive fixture is, stripped of its `//`
# envelope lines, is compiled by the installed `javac --release 8` and by the real javac 8
# (Corretto 1.8.0_432), and both recompiled classes are run under `-Xverify:all` beside the
# fixture's own class files. The two legs' own bytes are the baseline: the recovered text has to
# answer exactly what they answer.
#
# The counts are only trusted after the render's own self-header is asserted (`// jarde:
# presentation of`): an error document would otherwise be counted as a render of zero refusals.
set -eu

ROOT=$(cd "$(dirname "$0")/../../../../.." && pwd)
CLI=${CLI:-$ROOT/target/debug/jarde-cli}
JAVAC8=${JAVAC8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac}
JAVA8=${JAVA8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java}
FIX=$ROOT/tests/fixtures/preserve-local-scope-plan
WORK=${WORK:-/tmp/local-scope-plan-roundtrip}
rm -rf "$WORK"
mkdir -p "$WORK/stripped" "$WORK/original-v8" "$WORK/original-v8-javac8" "$WORK/installed" "$WORK/real8"

render() {
    leg=$1
    class=$2
    out=$3
    "$CLI" class-source --policy single-class --input "$FIX/$leg/$class.class" --class "$class" \
        --format text >"$out" 2>"$out.err"
    head -1 "$out" | grep -q '^// jarde: presentation of' ||
        { echo "SELF-HEADER MISSING in $out (not a render):"; head -3 "$out"; exit 1; }
}

for leg in v8 v8-javac8; do
    render "$leg" ScopePlan "$WORK/$leg-ScopePlan.txt"
    grep -v '^[[:space:]]*//' "$WORK/$leg-ScopePlan.txt" >"$WORK/stripped/ScopePlan-$leg.java"
    render "$leg" ScopePlanCrossing "$WORK/$leg-ScopePlanCrossing.txt"
    cp "$FIX/$leg"/*.class "$WORK/original-$leg/"
done

# The two legs' recovered text is one text (the classification reads control flow, not a compiler's
# lowering): the stripped sources must be byte-identical, and each is compiled on its own anyway.
cmp "$WORK/stripped/ScopePlan-v8.java" "$WORK/stripped/ScopePlan-v8-javac8.java" ||
    { echo "the two legs' stripped texts differ"; exit 1; }
cp "$WORK/stripped/ScopePlan-v8.java" "$WORK/stripped/ScopePlan.java"

# The negatives keep their two refusals verbatim; they are not replayed (their refused members
# carry the marker body no javac accepts), exactly as the sister changes' negatives do.
for leg in v8 v8-javac8; do
    [ "$(grep -c 'crosses a quoted fallback region' "$WORK/$leg-ScopePlanCrossing.txt")" = 2 ] ||
        { echo "$leg/ScopePlanCrossing does not keep exactly its two refusals"; exit 1; }
done

javac --release 8 -Xlint:-options -d "$WORK/installed" "$WORK/stripped/ScopePlan.java" \
    2>"$WORK/installed.log"
echo "installed javac --release 8: exit 0"
"$JAVAC8" -d "$WORK/real8" "$WORK/stripped/ScopePlan.java" 2>"$WORK/real8.log"
echo "real javac 8 ($JAVAC8): exit 0"

for leg in v8 v8-javac8; do
    echo "original $leg: $(java -Xverify:all -cp "$WORK/original-$leg" ScopePlan)"
done

baseline=$(java -Xverify:all -cp "$WORK/original-v8" ScopePlan)
for run in installed real8; do
    if [ "$run" = installed ]; then
        recovered=$(java -Xverify:all -cp "$WORK/installed" ScopePlan)
    else
        recovered=$("$JAVA8" -Xverify:all -cp "$WORK/real8" ScopePlan)
    fi
    echo "recovered ($run): $recovered"
    [ "$baseline" = "$recovered" ] || { echo "DIVERGES ($run): $baseline vs $recovered"; exit 1; }
done
echo "ROUNDTRIP OK: the recovered class answers exactly what the fixture's own bytes answer"
