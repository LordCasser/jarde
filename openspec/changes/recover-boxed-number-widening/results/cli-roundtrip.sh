#!/bin/sh
# The acceptance roundtrip through the shipped entry point: the CLI renders each anchor class, the
# render is stripped of its `//` lines, compiled by the installed `javac --release 8` and by a real
# javac 8, and both runs are compared with the fixture's own class files under `-Xverify:all`.
#
# The counts are only trusted after the render's own self-header is asserted (`// jarde:
# presentation of`): an error document would otherwise be counted as a render of zero refusals.
set -eu

CLI=${CLI:-target/debug/jarde-cli}
JAVAC8=${JAVAC8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac}
FIXTURES=tests/fixtures/recover-boxed-number-widening
WORK=${WORK:-/tmp/bn-render/cli-roundtrip}
rm -rf "$WORK"
mkdir -p "$WORK/v8" "$WORK/v8j8"

render_and_check() {
    leg=$1
    class=$2
    out=$3
    "$CLI" class-source --policy single-class --input "$FIXTURES/$leg/$class.class" --class "$class" \
        --format text >"$out" 2>"$out.err"
    head -1 "$out" | grep -q '^// jarde: presentation of' ||
        { echo "SELF-HEADER MISSING in $out (not a render):"; head -3 "$out"; exit 1; }
    refusals=$(grep -c 'no safe reference conversion evidence' "$out" || true)
    echo "render $leg/$class: $(wc -c <"$out" | tr -d ' ') bytes, $refusals refusal sentence(s)"
}

for leg in v8 v8-javac8; do
    mkdir -p "$WORK/stripped-$leg"
    for class in C8 BN; do
        render_and_check "$leg" "$class" "$WORK/$leg-$class.txt"
        grep -v '^[[:space:]]*//' "$WORK/$leg-$class.txt" >"$WORK/stripped-$leg/$class.java"
    done
done

# The negatives are rendered too, but not replayed: their refused members carry the marker body no
# javac accepts, exactly as the sister changes' negatives do. The assertion is that both refusal
# sentences are still there, verbatim.
for leg in v8 v8-javac8; do
    render_and_check "$leg" BNX "$WORK/$leg-BNX.txt"
    [ "$(grep -c 'no safe reference conversion evidence' "$WORK/$leg-BNX.txt")" = 2 ] ||
        { echo "BNX does not keep exactly its two refusals"; exit 1; }
done

javac --release 8 -Xlint:-options -d "$WORK/v8" "$WORK"/stripped-v8/*.java 2>"$WORK/v8.compile.log"
echo "installed javac --release 8: exit 0"
"$JAVAC8" -d "$WORK/v8j8" "$WORK"/stripped-v8-javac8/*.java 2>"$WORK/v8j8.compile.log"
echo "real javac 8 ($JAVAC8): exit 0"

answers() {
    classpath=$1
    class=$2
    java -Xverify:all -cp "$classpath" "$class"
}

for leg in v8 v8-javac8; do
    mkdir -p "$WORK/original-$leg"
    cp "$FIXTURES/$leg"/*.class "$WORK/original-$leg/"
done

for class in C8 BN; do
    original_v8=$(answers "$WORK/original-v8" "$class")
    original_j8=$(answers "$WORK/original-v8-javac8" "$class")
    [ "$original_v8" = "$original_j8" ] || { echo "$class: the two legs' own classes diverge"; exit 1; }
    stripped_v8=$(answers "$WORK/v8" "$class")
    stripped_j8=$(answers "$WORK/v8j8" "$class")
    printf '%s: original | installed-javac | real-javac8\n' "$class"
    printf '  %s\n' "$(printf '%s' "$original_v8" | tr '\n' '|')"
    printf '  %s\n' "$(printf '%s' "$stripped_v8" | tr '\n' '|')"
    printf '  %s\n' "$(printf '%s' "$stripped_j8" | tr '\n' '|')"
    [ "$original_v8" = "$stripped_v8" ] || { echo "DIVERGES (installed javac)"; exit 1; }
    [ "$original_v8" = "$stripped_j8" ] || { echo "DIVERGES (real javac 8)"; exit 1; }
done
echo "ROUNDTRIP OK: both stripped legs answer exactly what the fixture's own classes answer"
