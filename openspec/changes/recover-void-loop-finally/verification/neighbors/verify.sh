#!/bin/sh
# Every neighbor must pass `java -Xverify:all` on its own: the refusals below are choices of the
# certificate, never of a class the JVM would reject.
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-void-loop-finally/verification/neighbors
FIX=$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally
TMP=$(mktemp -d /tmp/jarde-cf16t2-neighbor-verify.XXXXXX)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
javac --release 8 -g:none -Xlint:-options -d "$TMP" "$HERE/Verify.java"
for path in "$HERE"/*.class; do
  name=$(basename "$path" .class)
  mkdir -p "$TMP/jadx/tests/integration/trycatch"
  cp "$FIX/support/jadx/core/clsp/ClspClass.java" "$TMP/"
  cp "$FIX/support/jadx/core/dex/instructions/args/ArgType.java" "$TMP/"
  javac --release 8 -g:none -Xlint:-options -d "$TMP" \
    "$TMP/ClspClass.java" "$TMP/ArgType.java"
  cp "$path" "$TMP/jadx/tests/integration/trycatch/TestTryCatchFinally2\$TestCls.class"
  printf '%s ' "$name"
  java -Xverify:all -cp "$TMP" Verify
  digest=$(shasum -a 256 "$path" | cut -d ' ' -f 1)
  printf '%s  %s.class\n' "$digest" "$name"
done
