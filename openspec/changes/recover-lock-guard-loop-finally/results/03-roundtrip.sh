#!/bin/sh
# The anchor's roundtrip, outside the test suite: render the patrol's own jar, strip the layer's
# `//` envelope lines, compile the text on both javac legs, and run every class under
# `-Xverify:all` beside the original. The recorded line is the patrol's (`1/0/true`), and its value
# is the lock/unlock ordering itself.
#
# Usage: 03-roundtrip.sh [cli]
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
CLI=${1:-$ROOT/target/debug/jarde-cli}
WORK=${WORK:-/tmp/lk/roundtrip}
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java
JAR=$ROOT/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar
OUT="$HERE/03-roundtrip.out"

rm -rf "$WORK"
mkdir -p "$WORK/rendered-v23" "$WORK/rendered-v8" "$WORK/rendered"

{
    echo "# render"
    "$CLI" class-source --input "$JAR" --class LK --format text >"$WORK/LK.java" 2>/dev/null
    head -1 "$WORK/LK.java" | grep -q '^// jarde: presentation of `LK`' ||
        { echo "SELF-TEST FAILED: the render has no header"; exit 1; }
    if grep -q '@bytecode' "$WORK/LK.java"; then
        echo "SELF-TEST FAILED: the render still carries a refusal"
        exit 1
    fi
    echo "header ok, no refusals"
    echo "# original"
    "$JAVA8" -Xverify:all -cp "$JAR" LK | tee "$WORK/original.out"
    echo "# stripped text compiles and runs"
    grep -v '^[[:space:]]*//' "$WORK/LK.java" >"$WORK/rendered/LK.java"
    javac --release 8 -d "$WORK/rendered-v23" "$WORK/rendered/LK.java"
    "$JAVA8" -Xverify:all -cp "$WORK/rendered-v23" LK | tee "$WORK/rendered-v23.out"
    "$JAVAC8" -d "$WORK/rendered-v8" "$WORK/rendered/LK.java"
    "$JAVA8" -Xverify:all -cp "$WORK/rendered-v8" LK | tee "$WORK/rendered-v8.out"
    echo "# comparison"
    cmp "$WORK/original.out" "$WORK/rendered-v23.out" &&
        echo "javac 23 leg: identical to the original"
    cmp "$WORK/original.out" "$WORK/rendered-v8.out" &&
        echo "javac 8 leg: identical to the original"
} >"$OUT" 2>&1
cat "$OUT"
