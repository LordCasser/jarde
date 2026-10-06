#!/bin/sh
# The acceptance roundtrip through the shipped entry point: the CLI renders each anchor jar, the
# render is stripped of its `//` lines, compiled by the installed `javac --release 8` and by a real
# javac 8, and both are run under `-Xverify:all` beside the fixture's own class files.
#
# The counts are only trusted after the render's own self-header is asserted (`// jarde:
# presentation of`): an error document would otherwise be counted as a render of zero refusals.
set -eu

CLI=${CLI:-target/debug/jarde-cli}
JAVAC8=${JAVAC8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac}
EVIDENCE=openspec/evidence/java-syntax-2026-10-05
WORK=/tmp/tw-render/cli-roundtrip
rm -rf "$WORK"
mkdir -p "$WORK/stripped"

render_and_check() {
    jar=$1
    class=$2
    out=$3
    "$CLI" class-source --input "$jar" --class "$class" --format text >"$out" 2>"$out.err"
    head -1 "$out" | grep -q '^// jarde: presentation of' ||
        { echo "SELF-HEADER MISSING in $out (not a render):"; head -3 "$out"; exit 1; }
    refusals=$(grep -c 'no safe reference conversion evidence' "$out" || true)
    echo "render $class: $(wc -c <"$out" | tr -d ' ') bytes, $refusals refusal sentence(s)"
}

render_and_check "$EVIDENCE/java-time-temporal-patrol/fixture/jt.jar" JT "$WORK/JT.txt"
render_and_check "$EVIDENCE/java-time-temporal-patrol/fixture/dt.jar" DT "$WORK/DT.txt"
render_and_check "$EVIDENCE/completable-future-patrol/fixture/cf.jar" CF "$WORK/CF.txt"

for class in JT DT CF; do
    grep -v '^[[:space:]]*//' "$WORK/$class.txt" >"$WORK/stripped/$class.java"
done

mkdir -p "$WORK/v8" "$WORK/v8j8"
javac --release 8 -Xlint:-options -d "$WORK/v8" "$WORK"/stripped/*.java 2>"$WORK/v8.compile.log"
echo "installed javac --release 8: exit 0"
"$JAVAC8" -d "$WORK/v8j8" "$WORK"/stripped/*.java 2>"$WORK/v8j8.compile.log"
echo "real javac 8 ($JAVAC8): exit 0"

answers() {
    classpath=$1
    class=$2
    java -Xverify:all -cp "$classpath" "$class"
}

# The fixture's own class files, from the committed `v8` leg (byte-identical to the patrol jars).
for class in JT DT CF; do
    cp "tests/fixtures/recover-temporal-argument-widening/v8/$class.class" "$WORK/original-$class.class"
done
mkdir -p "$WORK/original"
for class in JT DT CF; do
    cp "tests/fixtures/recover-temporal-argument-widening/v8/$class.class" "$WORK/original/"
done

for class in JT DT CF; do
    original=$(java -Xverify:all -cp "$WORK/original" "$class")
    stripped=$(answers "$WORK/v8" "$class")
    real=$(answers "$WORK/v8j8" "$class")
    printf '%s: original | installed-javac | real-javac8\n' "$class"
    printf '  %s\n' "$(printf '%s' "$original" | tr '\n' '|')"
    printf '  %s\n' "$(printf '%s' "$stripped" | tr '\n' '|')"
    printf '  %s\n' "$(printf '%s' "$real" | tr '\n' '|')"
    [ "$original" = "$stripped" ] || { echo "DIVERGES (installed javac)"; exit 1; }
    [ "$original" = "$real" ] || { echo "DIVERGES (real javac 8)"; exit 1; }
done
echo "ROUNDTRIP OK: both stripped legs answer exactly what the fixture's own classes answer"
