#!/bin/sh
# Regenerates this directory's Tf2 compiled comparisons: the original transcription, the frozen
# JADX java-input, and this slice's fresh Jarde class-source, each recompiled as a full Java 8
# class and run under `java -Xverify:all`. The support classes (Result/Support/Runner) stay the
# original compilation's own on every side's runtime classpath, so the three sides differ only
# in the probe class under test. The behavioral oracle is original == Jarde; the JADX java-input
# is recorded for reference (its recovered Tf2 renames the body's assignment, which the patrol
# registered as reference-only).
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/probe
SRC=$HERE/src-tf2
JARDE=${1:?usage: run-behavior-tf2.sh /path/to/jarde-cli}
JADX=${2:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}

rm -rf "$HERE/original-tf2" "$HERE/jadx-tf2" "$HERE/jarde-tf2" "$HERE/jadx-out-tf2"
mkdir -p "$HERE/original-tf2" "$HERE/jadx-tf2" "$HERE/jarde-tf2"

javac --release 8 -g:none -Xlint:-options -d "$HERE/original-tf2" "$SRC"/*.java
2> "$HERE/original-tf2.javac.stderr"

"$JADX" -d "$HERE/jadx-out-tf2" --no-res --output-format java \
"$HERE/original-tf2/Tf2Probe.class" 2> "$HERE/jadx-tf2.stderr"
sed 's/^package defpackage;//' "$HERE/jadx-out-tf2/sources/defpackage/Tf2Probe.java" \
> "$HERE/jadx-tf2/Tf2Probe.java"

"$JARDE" class-source --input "$HERE/original-tf2/Tf2Probe.class" --policy single-class \
--class Tf2Probe 2> "$HERE/jarde-recover-tf2.stderr" > "$HERE/jarde-tf2/Tf2Probe.java"

javac --release 8 -g:none -Xlint:-options -cp "$HERE/original-tf2" -d "$HERE/jadx-tf2" \
"$HERE/jadx-tf2/Tf2Probe.java" 2> "$HERE/jadx-tf2.javac.stderr"
javac --release 8 -g:none -Xlint:-options -cp "$HERE/original-tf2" -d "$HERE/jarde-tf2" \
"$HERE/jarde-tf2/Tf2Probe.java" 2> "$HERE/jarde-tf2.javac.stderr"

for side in original jadx jarde; do
java -Xverify:all -cp "$HERE/$side-tf2:$HERE/original-tf2" Runner Tf2Probe \
> "$HERE/run-$side-tf2.txt" 2> "$HERE/verify-$side-tf2.txt"
cmp -s /dev/null "$HERE/verify-$side-tf2.txt"
done
cmp "$HERE/run-original-tf2.txt" "$HERE/run-jarde-tf2.txt"

(cd "$HERE" && shasum -a 256 src-tf2/*.java jarde-tf2/Tf2Probe.java jadx-tf2/Tf2Probe.java \
run-original-tf2.txt run-jarde-tf2.txt run-jadx-tf2.txt) > behavior-tf2-sha256.txt
printf 'paths: %s\n' "$(grep -c 'outcome=' run-original-tf2.txt)"
