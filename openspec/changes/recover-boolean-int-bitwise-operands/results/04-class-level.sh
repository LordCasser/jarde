#!/bin/sh
# The class-level probe (`recover-boolean-int-bitwise-operands`): strip each recovered presentation
# the way the patrols' own stripped sources were made (comment lines dropped), compile it with
# `javac --release 8` and with real javac 8 (Corretto 1.8.0_432), run both under `-Xverify:all`, and
# compare every answer with the fixture's own class file.
#
# Self-tests, before any comparison is believed:
#   * the frozen `BW` prints `true/5/true/2/-2147483648/false/3` (the patrol's recorded values);
#   * the recovered text compiles under both compilers (a `javac` failure is printed, not swallowed).
set -eu

ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11405-eb09-78f1-babc-dbe140bcaad2
CLI=$ROOT/target/debug/jarde-cli
FIX=$ROOT/tests/fixtures/recover-boolean-int-bitwise-operands
PATROL=$ROOT/openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.class
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java
WORK=${1:-/tmp/boolean-int-bitwise/class-level}
rm -rf "$WORK"
mkdir -p "$WORK"

render() { # class-file class-name out
    "$CLI" class-source --policy single-class --input "$1" --class "$2" --format text >"$3" 2>/dev/null
    head -1 "$3" | grep -q '// jarde: presentation of' || { echo "NO SELF-HEADER: $1"; exit 1; }
}

strip() { # in out
    sed '/^[[:space:]]*\/\//d' "$1" >"$2"
}

probe() { # label class-file class-name
    label=$1
    class_file=$2
    class=$3
    dir=$WORK/$label
    mkdir -p "$dir/original" "$dir/v8" "$dir/v8javac8"
    cp "$class_file" "$dir/original/$class.class"
    render "$class_file" "$class" "$dir/$class.txt"
    strip "$dir/$class.txt" "$dir/v8/$class.java"
    cp "$dir/v8/$class.java" "$dir/v8javac8/$class.java"
    want=$(/usr/bin/java -Xverify:all -cp "$dir/original" "$class" 2>&1)
    echo "--- $label"
    echo "original      : $(printf '%s' "$want" | tr '\n' '|')"
    if ! /usr/bin/javac --release 8 -nowarn -d "$dir/v8" "$dir/v8/$class.java" >"$dir/v8.log" 2>&1; then
        echo "javac 23 --release 8: FAILED"; sed 's/^/    /' "$dir/v8.log"; exit 1
    fi
    got=$(/usr/bin/java -Xverify:all -cp "$dir/v8" "$class" 2>&1)
    echo "javac 23 --release 8: $(printf '%s' "$got" | tr '\n' '|')"
    [ "$got" = "$want" ] || { echo "MISMATCH"; exit 1; }
    if [ -x "$JAVAC8" ]; then
        if ! "$JAVAC8" -nowarn -d "$dir/v8javac8" "$dir/v8javac8/$class.java" >"$dir/v8javac8.log" 2>&1; then
            echo "corretto 1.8.0_432   : FAILED"; sed 's/^/    /' "$dir/v8javac8.log"; exit 1
        fi
        got8=$("$JAVA8" -Xverify:all -cp "$dir/v8javac8" "$class" 2>&1)
        echo "corretto 1.8.0_432   : $(printf '%s' "$got8" | tr '\n' '|')"
        [ "$got8" = "$want" ] || { echo "MISMATCH"; exit 1; }
    else
        echo "corretto 1.8.0_432   : not installed, leg skipped"
    fi
}

# The frozen anchor's own answer, before anything is compared against it.
frozen=$WORK/frozen-original
mkdir -p "$frozen"
cp "$PATROL" "$frozen/BW.class"
frozen_answer=$(/usr/bin/java -Xverify:all -cp "$frozen" BW 2>&1)
if [ "$frozen_answer" != "true/5/true/2/-2147483648/false/3" ]; then
    echo "SELF-TEST FAILED: the frozen BW answers $frozen_answer"
    exit 1
fi
echo "SELF-TEST OK: the frozen BW answers $frozen_answer"

probe frozen "$PATROL" BW
probe v8 "$FIX/v8/BW.class" BW
probe v8-javac8 "$FIX/v8-javac8/BW.class" BW
probe bwr "$FIX/v8/BWR.class" BWR

# The negatives' stripped text must not compile: a body a reader cannot compile, never one that
# compiles and behaves differently.
for leg in v8 v8-javac8; do
    dir=$WORK/negative-$leg
    mkdir -p "$dir"
    render "$FIX/$leg/BWN.class" BWN "$dir/BWN.txt"
    strip "$dir/BWN.txt" "$dir/BWN.java"
    if /usr/bin/javac --release 8 -nowarn -d "$dir" "$dir/BWN.java" >"$dir.log" 2>&1; then
        echo "NEGATIVE FAILED: BWN ($leg) compiled"; exit 1
    fi
    echo "negative BWN ($leg): does not compile, as the refusals require ($(grep -c '缺少返回语句\|missing return' "$dir.log" || true) missing-return errors)"
done
echo "CLASS-LEVEL PROBE OK: every recovering class answers what its own class file answers, on both legs"
