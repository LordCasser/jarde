#!/bin/sh
# Regenerates this directory's Tf3 compiled comparisons: the original transcription and this
# slice's fresh Jarde class-source, each recompiled as a full Java 8 class and run under
# `java -Xverify:all` on the five behavior paths. The support classes (Runner/Support) stay
# the original compilation's own on every side's runtime classpath, so the two sides differ
# only in the probe class under test. The behavioral oracle is original == Jarde.
# The frozen JADX java-input is recovered and its recompile attempted for the record, but it
# is reference-only here: its own recovery of this shape reads an unassigned local inside the
# finally on the early-return path (`close(inputStream)` after `return null;`), which javac's
# definite-assignment rule rejects, beside the `inputStream2` copy the patrol registered.
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/evidence/java-syntax-2026-09-30/testfinally-patrol/probe
SRC=$HERE/src-tf3
JARDE=${1:?usage: run-behavior-tf3.sh /path/to/jarde-cli}
JADX=${2:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}

rm -rf "$HERE/original-tf3" "$HERE/jadx-tf3" "$HERE/jarde-tf3" "$HERE/jadx-out-tf3"
mkdir -p "$HERE/original-tf3" "$HERE/jadx-tf3" "$HERE/jarde-tf3"

javac --release 8 -g:none -Xlint:-options -d "$HERE/original-tf3" "$SRC"/*.java
2> "$HERE/original-tf3.javac.stderr"

"$JARDE" class-source --input "$HERE/original-tf3/Tf3Probe.class" --policy single-class \
--class Tf3Probe 2> "$HERE/jarde-recover-tf3.stderr" > "$HERE/jarde-tf3/Tf3Probe.java"

javac --release 8 -g:none -Xlint:-options -cp "$HERE/original-tf3" -d "$HERE/jarde-tf3" \
"$HERE/jarde-tf3/Tf3Probe.java" 2> "$HERE/jarde-tf3.javac.stderr"

if [ -x "$JADX" ]; then
  "$JADX" -d "$HERE/jadx-out-tf3" --no-res --output-format java \
"$HERE/original-tf3/Tf3Probe.class" 2> "$HERE/jadx-tf3.stderr" || true
  sed 's/^package defpackage;//' "$HERE/jadx-out-tf3/sources/defpackage/Tf3Probe.java" \
> "$HERE/jadx-tf3/Tf3Probe.java"
  if javac --release 8 -g:none -Xlint:-options -cp "$HERE/original-tf3" -d "$HERE/jadx-tf3" \
"$HERE/jadx-tf3/Tf3Probe.java" 2> "$HERE/jadx-tf3.javac.stderr"; then
    java -Xverify:all -cp "$HERE/jadx-tf3:$HERE/original-tf3" Runner Tf3Probe \
> "$HERE/run-jadx-tf3.txt" 2> "$HERE/verify-jadx-tf3.txt"
  else
    printf 'jadx java-input recompile refused (definite-assignment on the early-return path): see jadx-tf3.javac.stderr\n' \
> "$HERE/run-jadx-tf3.txt"
  fi
else
  printf 'jadx CLI not available\n' > "$HERE/run-jadx-tf3.txt"
fi

for side in original jarde; do
java -Xverify:all -cp "$HERE/$side-tf3:$HERE/original-tf3" Runner Tf3Probe \
> "$HERE/run-$side-tf3.txt" 2> "$HERE/verify-$side-tf3.txt"
cmp -s /dev/null "$HERE/verify-$side-tf3.txt"
done
cmp "$HERE/run-original-tf3.txt" "$HERE/run-jarde-tf3.txt"

(cd "$HERE" && shasum -a 256 src-tf3/*.java jarde-tf3/Tf3Probe.java jadx-tf3/Tf3Probe.java \
run-original-tf3.txt run-jarde-tf3.txt run-jadx-tf3.txt > behavior-tf3-sha256.txt && \
printf 'paths: %s\n' "$(grep -c 'outcome=' run-original-tf3.txt)")
