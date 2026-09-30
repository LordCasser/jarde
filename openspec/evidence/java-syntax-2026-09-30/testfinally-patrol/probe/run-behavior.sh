#!/bin/sh
# Regenerates this directory's compiled comparisons: the original transcription, the frozen
# JADX java-input, and this slice's fresh Jarde class-source, each recompiled as a full Java 8
# class and run under `java -Xverify:all`. The call-failure probe compares three sides; the
# cleanup-failure probe's Jarde side is the recorded refusal (a throw inside the cleanup copy
# is a grammar addition the certificate must refuse), so its behavior comparison is the two
# compilable sides. The call probe's JADX side is the z/z2-distorted reference the patrol
# README registered; the behavioral oracle is original == Jarde.
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/probe
JARDE=${1:?usage: run-behavior.sh /path/to/jarde-cli}
JADX=${2:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}

rm -rf "$HERE/original" "$HERE/jadx" "$HERE/jarde" "$HERE/jadx-out"
mkdir -p "$HERE/original" "$HERE/jadx" "$HERE/jarde"

javac --release 8 -g:none -Xlint:-options -d "$HERE/original" "$HERE"/src/*.java
2> "$HERE/original.javac.stderr"

"$JADX" -d "$HERE/jadx-out" --no-res --output-format java \
	"$HERE/original/Tf4Probe.class" "$HERE/original/Tf4CleanupProbe.class" \
	2> "$HERE/jadx.stderr"
sed 's/^package defpackage;//' "$HERE/jadx-out/sources/defpackage/Tf4Probe.java" \
	> "$HERE/jadx/Tf4Probe.java"
sed 's/^package defpackage;//' "$HERE/jadx-out/sources/defpackage/Tf4CleanupProbe.java" \
	> "$HERE/jadx/Tf4CleanupProbe.java"

"$JARDE" class-source --input "$HERE/original/Tf4Probe.class" --policy single-class \
	--class Tf4Probe 2> "$HERE/jarde-recover.stderr" > "$HERE/jarde/Tf4Probe.java"
"$JARDE" class-source --input "$HERE/original/Tf4CleanupProbe.class" --policy single-class \
	--class Tf4CleanupProbe 2>> "$HERE/jarde-recover.stderr" > "$HERE/jarde/Tf4CleanupProbe.java"

javac --release 8 -g:none -Xlint:-options -d "$HERE/jadx" \
	"$HERE/jadx/Tf4Probe.java" "$HERE/jadx/Tf4CleanupProbe.java" "$HERE/src/Runner.java" \
	2> "$HERE/jadx.javac.stderr"
javac --release 8 -g:none -Xlint:-options -d "$HERE/jarde" \
	"$HERE/jarde/Tf4Probe.java" "$HERE/src/Runner.java" \
	2> "$HERE/jarde.javac.stderr"

for side in original jadx jarde; do
	java -Xverify:all -cp "$HERE/$side" Runner Tf4Probe \
		> "$HERE/run-$side-probe.txt" 2> "$HERE/verify-$side-probe.txt"
	cmp -s /dev/null "$HERE/verify-$side-probe.txt"
done
cmp "$HERE/run-original-probe.txt" "$HERE/run-jarde-probe.txt"

java -Xverify:all -cp "$HERE/original" Runner Tf4CleanupProbe true \
	> "$HERE/run-original-cleanup.txt" 2> "$HERE/verify-original-cleanup.txt"
java -Xverify:all -cp "$HERE/jadx" Runner Tf4CleanupProbe true \
	> "$HERE/run-jadx-cleanup.txt" 2> "$HERE/verify-jadx-cleanup.txt"
cmp "$HERE/run-original-cleanup.txt" "$HERE/run-jadx-cleanup.txt"

(cd "$HERE" && shasum -a 256 src/*.java jarde/Tf4Probe.java jadx/Tf4Probe.java \
	run-original-probe.txt run-jarde-probe.txt run-original-cleanup.txt) > behavior-sha256.txt
printf 'paths: %s + %s\n' "$(grep -c 'outcome=' run-original-probe.txt)" \
	"$(grep -c 'outcome=' run-original-cleanup.txt)"
