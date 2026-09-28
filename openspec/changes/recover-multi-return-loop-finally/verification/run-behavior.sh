#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-multi-return-loop-finally/verification
FIX=$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test5-multi-return
TMP=$(mktemp -d /tmp/jarde-cf16-test5-behavior.XXXXXX)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
mkdir -p "$TMP"/{fixed,original,jadx,jarde}/jadx/tests/integration/trycatch
javac --release 8 -g:none -Xlint:-options -d "$TMP/fixed" "$HERE/Runner.java"
cp "$FIX/TestTryCatchFinally5\$TestCls.class" "$FIX/fixed-aux/"*.class "$TMP/fixed/jadx/tests/integration/trycatch/"
javac --release 8 -g:none -Xlint:-options -d "$TMP/original" "$FIX/original/TestTryCatchFinally5.java" "$HERE/Runner.java"
javac --release 8 -g:none -Xlint:-options -d "$TMP/jadx" "$FIX/jadx/TestTryCatchFinally5\$TestCls.java" "$HERE/Runner.java"
javac --release 8 -g:none -Xlint:-options -d "$TMP/jarde" "$FIX/jarde/Support.java" "$HERE/TestTryCatchFinally5\$TestCls.java" "$HERE/Runner.java"
cp "$FIX/fixed-aux/"*.class "$TMP/jarde/jadx/tests/integration/trycatch/"
for profile in fixed original jadx jarde; do
  java -Xverify:all -cp "$TMP/$profile" jadx.tests.integration.trycatch.Runner > "$HERE/$profile.run.txt"
done
cmp "$HERE/fixed.run.txt" "$HERE/original.run.txt"
cmp "$HERE/fixed.run.txt" "$HERE/jadx.run.txt"
cmp "$HERE/fixed.run.txt" "$HERE/jarde.run.txt"
