#!/bin/sh
# The behaviour leg of the corpus delta (`recover-chained-field-assignment`, task 3.2).
#
# For every class the sweep moved, the patched render is stripped of comment lines, compiled with
# `javac --release 8` and with real javac 8, and run under `-Xverify:all`; its standard output and
# exit status are compared with the same run over the committed class file. A moved class whose text
# compiles and answers differently is the one outcome this discipline exists to catch.
set -eu

ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
PATCHED=${PATCHED:-$ROOT/target/debug/jarde-cli}
WORK=${1:-/tmp/fieldcopy/behavior}
JAVAC=/usr/bin/javac
JAVA=/usr/bin/java
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java
rm -rf "$WORK"
mkdir -p "$WORK"

# One moved class: the render, the jar it came from, the entry name and the class name.
check() {
    label=$1
    jar=$2
    entry=$3
    class=$4
    dir=$WORK/$label
    mkdir -p "$dir/original" "$dir/recovered" "$dir/recovered8"
    unzip -o -q "$jar" "$entry" -d "$dir/original"
    "$PATCHED" class-source --policy plain-jar --input "$jar" --class "$class" --format text \
        >"$dir/render.txt" 2>/dev/null
    run_legs "$label" "$dir" "$class"
}

# One moved class that is a **loose** class file: the single-class posture, and the file itself is
# the committed original.
check_loose() {
    label=$1
    file=$2
    class=$3
    dir=$WORK/$label
    mkdir -p "$dir/original" "$dir/recovered" "$dir/recovered8"
    cp "$file" "$dir/original/$class.class"
    "$PATCHED" class-source --policy single-class --input "$file" --class "$class" --format text \
        >"$dir/render.txt" 2>/dev/null
    run_legs "$label" "$dir" "$class"
}

run_legs() {
    label=$1
    dir=$2
    class=$3
    head -1 "$dir/render.txt" | grep -q '// jarde: presentation of' || {
        echo "SELF-TEST FAILED: $label renders under no name it states"
        exit 1
    }
    sed '/^[[:space:]]*\/\//d' "$dir/render.txt" >"$dir/recovered/$class.java"
    sed '/^[[:space:]]*\/\//d' "$dir/render.txt" >"$dir/recovered8/$class.java"
    # A class whose stripped text still holds a refused body marker is the *safe* form: it must not
    # compile at all, which is what the whole-method quote is for.
    if grep -q 'jarde_refused_body' "$dir/recovered/$class.java"; then
        if "$JAVAC" --release 8 -nowarn -d "$dir/recovered" "$dir/recovered/$class.java" \
            >"$dir/javac.log" 2>&1; then
            echo "$label: quoted whole and the stripped text COMPILES — the safe form is broken"
            return 1
        fi
        echo "$label: quoted whole; the stripped text does not compile (the safe form)"
        return 0
    fi
    ( cd "$dir/original" && "$JAVA" -Xverify:all -cp . "$class" >"$dir/original.out" 2>&1 ) || true
    "$JAVAC" --release 8 -nowarn -d "$dir/recovered" "$dir/recovered/$class.java" \
        >"$dir/javac.log" 2>&1 || { echo "$label: javac --release 8 FAILED"; cat "$dir/javac.log"; return 1; }
    ( cd "$dir/recovered" && "$JAVA" -Xverify:all -cp . "$class" >"$dir/recovered.out" 2>&1 ) || true
    "$JAVAC8" -nowarn -d "$dir/recovered8" "$dir/recovered8/$class.java" \
        >"$dir/javac8.log" 2>&1 || { echo "$label: real javac 8 FAILED"; cat "$dir/javac8.log"; return 1; }
    ( cd "$dir/recovered8" && "$JAVA8" -Xverify:all -cp . "$class" >"$dir/recovered8.out" 2>&1 ) || true
    if ! diff -q "$dir/original.out" "$dir/recovered.out" >/dev/null; then
        echo "$label: javac --release 8 leg ANSWERS DIFFERENTLY"
        diff "$dir/original.out" "$dir/recovered.out" | head -6
        return 1
    fi
    if ! diff -q "$dir/original.out" "$dir/recovered8.out" >/dev/null; then
        echo "$label: real javac 8 leg ANSWERS DIFFERENTLY"
        diff "$dir/original.out" "$dir/recovered8.out" | head -6
        return 1
    fi
    echo "$label: $(head -c 120 "$dir/original.out" | tr '\n' '|')  (both legs, -Xverify:all, identical)"
}

E=$ROOT/openspec/evidence/java-syntax-2026-10-05
F=$ROOT/tests/fixtures/recover-chained-field-assignment
check_loose CH "$E/chained-assign-sideeffect-patrol/fixture/CH.class" CH || true
check SC "$E/field-string-compound-patrol/fixture/sc.jar" "SC.class" SC || true
check BF "$E/field-compound-soundness-patrol/fixture/bf.jar" "BF.class" BF || true
check BG "$E/field-compound-soundness-patrol/fixture/bg.jar" "BG.class" BG || true
check BI "$E/boolean-loop-earlyret-patrol/fixture/bi.jar" "BI.class" BI || true
check CA2 "$E/assign-chain-soundness-patrol/fixture/ca2.jar" "CA2.class" CA2 || true
# The fixture legs are the class files themselves, not a jar: the entry is the file.
CFDIR=$WORK/cf
mkdir -p "$CFDIR/v8" "$CFDIR/v8-javac8"
( cd "$F/v8" && jar cf "$CFDIR/cf.jar" CF.class )
check CF-v8 "$CFDIR/cf.jar" "CF.class" CF || true
( cd "$F/v8-javac8" && jar cf "$CFDIR/cf8.jar" CF.class )
check CF-javac8 "$CFDIR/cf8.jar" "CF.class" CF || true
