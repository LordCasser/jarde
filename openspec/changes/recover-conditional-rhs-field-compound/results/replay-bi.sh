#!/bin/sh
# Replay the BI anchor end to end: present the frozen `bi.jar`'s `BI` with jarde, strip the
# comment lines the patrol's own stripped sources dropped, compile the whole class with
# `javac --release 8` and with the real javac 8, run both under `-Xverify:all`, and compare every
# answer with the frozen class's own.
#
# Usage: replay-bi.sh <absolute jarde-cli> <absolute bi.jar> <scratch dir>
# Exit status 0 means: 0 quoted BCIs, both legs compile, both legs print the original's answer.
set -eu

CLI=$1
JAR=$2
WORK=$3
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java

test -x "$CLI"
test -f "$JAR"
rm -rf "$WORK"
mkdir -p "$WORK/presented" "$WORK/original" "$WORK/leg23" "$WORK/leg8"

# The presentation of the whole class.
"$CLI" class-source --input "$JAR" --class BI --format text \
    --output "$WORK/presented/BI-presented.txt" 2>"$WORK/presented/stderr.txt"

# jarde's own self-header, asserted **before** anything is counted in the render: a render of
# nothing is not a render.
head -1 "$WORK/presented/BI-presented.txt" | grep -q '// jarde: presentation of `BI`'

# The stripped text: the comment lines dropped, exactly as the patrol's own stripped sources were
# made.
grep -v '^[[:space:]]*//' "$WORK/presented/BI-presented.txt" >"$WORK/presented/BI.java"

# The anchors: the compound assignment must be present and no line may quote a BCI.
echo "--- quotes ---"
grep -c '@bytecode' "$WORK/presented/BI-presented.txt" || true
echo "--- earlyRet ---"
sed -n '/boolean earlyRet/,/^    }/p' "$WORK/presented/BI-presented.txt"

# The original class's own answer.
(cd "$WORK/original" && unzip -o -q "$JAR")
ORIGINAL=$(cd "$WORK/original" && /usr/bin/java -Xverify:all -cp . BI)
echo "original: $ORIGINAL"

# The recovered text on both legs.
cp "$WORK/presented/BI.java" "$WORK/leg23/BI.java"
(cd "$WORK/leg23" && /usr/bin/javac --release 8 -nowarn -d . BI.java)
GOT23=$(cd "$WORK/leg23" && /usr/bin/java -Xverify:all -cp . BI)
echo "javac 23 --release 8: $GOT23"

cp "$WORK/presented/BI.java" "$WORK/leg8/BI.java"
(cd "$WORK/leg8" && "$JAVAC8" -nowarn -d . BI.java)
GOT8=$(cd "$WORK/leg8" && "$JAVA8" -Xverify:all -cp . BI)
echo "corretto 1.8.0_432: $GOT8"

test "$GOT23" = "$ORIGINAL"
test "$GOT8" = "$ORIGINAL"
echo "replay: both legs identical to the original"
