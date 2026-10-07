#!/bin/sh
# The three-way comparison of task 3.2 (`recover-statement-position-news`).
#
# Every anchor is read three ways and the three answers are compared, with the output SHA-256
# recorded for each leg:
#
#   1. 原 class      — the frozen class files, run under `-Xverify:all`;
#   2. 固定 Java 输入 — the fixture's own frozen `.java` source (the input the class was built from),
#                      compiled with `javac --release 8` and run under `-Xverify:all`;
#   3. Jarde 重编     — the recovered text, comment lines stripped, compiled the same way and run
#                      under `-Xverify:all`.
#
# `B5` is read from the patrol's committed fixture; `SP`/`SB` from this change's `v8` leg; `SPC`
# from this change's assembled class plus its support family (the original class is the first leg,
# and the recovered text is quoted, so its third leg is the non-compilation assertion the ignored
# test already states — this script prints what javac says).
set -u

ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11450-b786-7b93-b480-e65b9cc75b09
CLI=$ROOT/target/debug/jarde-cli
JAVAC=${JAVAC:-/usr/bin/javac}
JAVA=${JAVA:-/usr/bin/java}
WORK=${1:-/tmp/statement-position-news/three-way}
PATROL=$ROOT/openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture
FIX=$ROOT/tests/fixtures/recover-statement-position-news
rm -rf "$WORK"
mkdir -p "$WORK"

sha() { shasum -a 256 "$1" | awk '{print $1}'; }

# one class: the class directory, the class name, the frozen source, and the recovered text.
one() {
    label=$1
    dir=$2
    name=$3
    source=$4
    case=$WORK/$label
    mkdir -p "$case/original" "$case/input" "$case/recovered"

    # 1. the frozen class files.
    cp "$dir/$name.class" "$case/original/" 2>/dev/null
    for sibling in "$dir/$name\$"*.class; do
        [ -e "$sibling" ] || continue
        cp "$sibling" "$case/original/"
    done
    ( cd "$case/original" && "$JAVA" -Xverify:all -cp . "$name" >out.txt 2>&1; echo $? >status )

    # 2. the frozen Java input.
    cp "$source" "$case/input/$name.java"
    ( cd "$case/input" && "$JAVAC" --release 8 -nowarn -cp "$dir" -d out "$name.java" >javac.log 2>&1;
      echo $? >javac.status; "$JAVA" -Xverify:all -cp out:"$dir" "$name" >out.txt 2>&1; echo $? >status )

    # 3. Jarde's recovered text.
    "$CLI" class-source --policy single-class --input "$dir/$name.class" --class "$name" \
        --format text 2>/dev/null | grep -v '^[[:space:]]*//' >"$case/recovered/$name.java"
    ( cd "$case/recovered" && "$JAVAC" --release 8 -nowarn -cp "$dir" -d out "$name.java" >javac.log 2>&1;
      echo $? >javac.status )
    # A text that did not compile must not be "run": the classpath holds the original class, and a
    # JVM run there would report the original's answer as if the recovered text had produced it.
    if [ "$(cat "$case/recovered/javac.status")" != 0 ]; then
        echo "=== $label"
        echo "    original  run-exit=$(cat "$case/original/status") output-sha256=$(sha "$case/original/out.txt")"
        echo "    recovered javac-exit=1 — a refusal keeps this text incomplete (the safe form)"
        echo "    recovered javac says: $(head -1 "$case/recovered/javac.log")"
        return
    fi
    ( cd "$case/recovered" && "$JAVA" -Xverify:all -cp out:"$dir" "$name" >out.txt 2>&1; echo $? >status )

    echo "=== $label"
    for leg in original input recovered; do
        printf '    %-10s javac-exit=%s run-exit=%s output-sha256=%s\n' "$leg" \
            "$(cat "$case/$leg/javac.status" 2>/dev/null || echo n/a)" \
            "$(cat "$case/$leg/status")" "$(sha "$case/$leg/out.txt")"
    done
    if diff -q "$case/original/out.txt" "$case/recovered/out.txt" >/dev/null; then
        echo "    original vs recovered: IDENTICAL"
    else
        echo "    original vs recovered: DIFFERS"
        diff "$case/original/out.txt" "$case/recovered/out.txt" | sed -n '1,6p' | sed 's/^/        /'
    fi
    if diff -q "$case/input/out.txt" "$case/recovered/out.txt" >/dev/null; then
        echo "    frozen input vs recovered: IDENTICAL"
    else
        echo "    frozen input vs recovered: DIFFERS"
        diff "$case/input/out.txt" "$case/recovered/out.txt" | sed -n '1,6p' | sed 's/^/        /'
    fi
}

one B5 "$PATROL" B5 "$PATROL/B5.java"
one B6 "$PATROL" B6 "$PATROL/B6.java"
one SP "$FIX/v8" SP "$FIX/SP.java"
one SB "$FIX/v8" SB "$FIX/SB.java"

# The statement-position counterexample: the original class answers its class-initialization order,
# and the recovered text is quoted (its third leg is the non-compilation assertion).
case=$WORK/SPC
mkdir -p "$case/original" "$case/recovered"
cp "$FIX/SPC.class" "$case/original/"
cp "$FIX/spc/Side.java" "$FIX/spc/Target.java" "$FIX/spc/Trace.java" "$FIX/spc/SPCRunner.java" "$case/original/"
( cd "$case/original" && "$JAVAC" --release 8 -nowarn -cp . -d . Side.java Target.java Trace.java SPCRunner.java >javac.log 2>&1;
  echo $? >javac.status; "$JAVA" -Xverify:all -cp . SPCRunner >out.txt 2>&1; echo $? >status )
"$CLI" class-source --policy single-class --input "$FIX/SPC.class" --class SPC --format text 2>/dev/null \
    | grep -v '^[[:space:]]*//' >"$case/recovered/SPC.java"
( cd "$case/recovered" && "$JAVAC" --release 8 -nowarn -d out SPC.java >javac.log 2>&1; echo $? >javac.status )
echo "=== SPC (the statement-position counterexample)"
echo "    original run-exit=$(cat "$case/original/status") output=$(tr -d '\n' <"$case/original/out.txt") output-sha256=$(sha "$case/original/out.txt")"
echo "    recovered javac-exit=$(cat "$case/recovered/javac.status") (the text keeps its refusal)"
echo "    recovered javac says: $(head -1 "$case/recovered/javac.log")"
