#!/bin/sh
# The dual-leg replay of `recover-array-initializer-value-positions` (evidence run, mirroring the
# integration test `tests/recover_array_initializer_value_positions.rs`).
#
# For every fixture class: strip the presentation's comment lines, compile the result with
# `javac --release 8` and with the real javac 8, run both under `-Xverify:all`, and compare the
# standard output with the fixture's own class file.
set -eu
WORK=${1:-/tmp/array-replay}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
FIXTURES=$ROOT/tests/fixtures/recover-array-initializer-value-positions
CLI=$ROOT/target/debug/jarde-cli
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java
rm -rf "$WORK"
mkdir -p "$WORK"

for leg in v8 v8-javac8; do
    (cd "$FIXTURES/$leg" && jar cf "$WORK/$leg.jar" MD.class MD2.class MD3.class AV.class)
    mkdir -p "$WORK/original.$leg"
    cp "$FIXTURES/$leg"/*.class "$WORK/original.$leg/"
    for class in MD MD2 MD3 AV; do
        "$CLI" class-source --policy plain-jar --input "$WORK/$leg.jar" --class "$class" \
            --format text >"$WORK/$class.$leg.txt" 2>/dev/null
        head -1 "$WORK/$class.$leg.txt" | grep -q '// jarde: presentation of' || {
            echo "FAILED: $class on $leg renders nothing"; exit 1; }
        if grep -q 'not recovered\|@bytecode' "$WORK/$class.$leg.txt"; then
            echo "FAILED: $class on $leg still quotes bytecode"; exit 1; fi
        grep -v '^[[:space:]]*//' "$WORK/$class.$leg.txt" >"$WORK/$class.$leg.java"
        for tool in "release:/usr/bin/javac" "javac8:$JAVAC8"; do
            name=${tool%%:*}
            compiler=${tool#*:}
            dir=$WORK/$class.$leg.$name
            mkdir -p "$dir"
            cp "$WORK/$class.$leg.java" "$dir/$class.java"
            if [ "$name" = release ]; then
                "$compiler" --release 8 -nowarn -d "$dir" "$dir/$class.java"
            else
                "$compiler" -nowarn -d "$dir" "$dir/$class.java"
            fi
            got=$(/usr/bin/java -Xverify:all -cp "$dir" "$class")
            want=$(/usr/bin/java -Xverify:all -cp "$WORK/original.$leg" "$class")
            if [ "$got" != "$want" ]; then
                echo "FAILED: $class on $leg via $name: got=$got want=$want"; exit 1
            fi
            echo "OK: $class on $leg via $name -> $got"
        done
    done
done
echo "REPLAY OK: every fixture compiles on both compilers and answers what its own class answers"
