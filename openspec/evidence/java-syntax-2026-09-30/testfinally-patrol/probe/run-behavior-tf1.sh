#!/bin/sh
# Regenerates this directory's Tf1 compiled comparisons: the original transcription, the frozen
# JADX java-input, and this slice's fresh Jarde class-source, each recompiled as a full Java 8
# class and run under `java -Xverify:all`. The support classes (Context/Cursor/Throwables) stay
# the original compilation's own on every side's runtime classpath, so the three sides differ
# only in the probe class under test. The behavioral oracle is original == Jarde; the JADX
# java-input is recorded for reference (its recovered Tf1 carries the dead null-check artifact
# the patrol registered, which is no behavior oracle).
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/probe
SRC=$HERE/src-tf1
JARDE=${1:?usage: run-behavior-tf1.sh /path/to/jarde-cli}
JADX=${2:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}

rm -rf "$HERE/original-tf1" "$HERE/jadx-tf1" "$HERE/jarde-tf1" "$HERE/jadx-out-tf1"
mkdir -p "$HERE/original-tf1" "$HERE/jadx-tf1" "$HERE/jarde-tf1"

javac --release 8 -g:none -Xlint:-options -d "$HERE/original-tf1" "$SRC"/*.java
2> "$HERE/original-tf1.javac.stderr"

"$JADX" -d "$HERE/jadx-out-tf1" --no-res --output-format java \
"$HERE/original-tf1/Tf1Probe.class" 2> "$HERE/jadx-tf1.stderr"
sed 's/^package defpackage;//' "$HERE/jadx-out-tf1/sources/defpackage/Tf1Probe.java" \
> "$HERE/jadx-tf1/Tf1Probe.java"

"$JARDE" class-source --input "$HERE/original-tf1/Tf1Probe.class" --policy single-class \
--class Tf1Probe 2> "$HERE/jarde-recover-tf1.stderr" > "$HERE/jarde-tf1/Tf1Probe.java"

javac --release 8 -g:none -Xlint:-options -cp "$HERE/original-tf1" -d "$HERE/jadx-tf1" \
"$HERE/jadx-tf1/Tf1Probe.java" 2> "$HERE/jadx-tf1.javac.stderr"
javac --release 8 -g:none -Xlint:-options -cp "$HERE/original-tf1" -d "$HERE/jarde-tf1" \
"$HERE/jarde-tf1/Tf1Probe.java" 2> "$HERE/jarde-tf1.javac.stderr"

for side in original jadx jarde; do
java -Xverify:all -cp "$HERE/$side-tf1:$HERE/original-tf1" Runner Tf1Probe \
> "$HERE/run-$side-tf1.txt" 2> "$HERE/verify-$side-tf1.txt"
cmp -s /dev/null "$HERE/verify-$side-tf1.txt"
done
cmp "$HERE/run-original-tf1.txt" "$HERE/run-jarde-tf1.txt"

(cd "$HERE" && shasum -a 256 src-tf1/*.java jarde-tf1/Tf1Probe.java jadx-tf1/Tf1Probe.java \
run-original-tf1.txt run-jarde-tf1.txt run-jadx-tf1.txt) > behavior-tf1-sha256.txt
printf 'paths: %s\n' "$(grep -c 'outcome=' run-original-tf1.txt)"
