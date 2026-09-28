#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-multi-return-loop-finally/verification/neighbors
FIX=$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test5-multi-return
TMP=$(mktemp -d /tmp/jarde-cf16-neighbor-verify.XXXXXX)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
javac --release 8 -g:none -Xlint:-options -d "$TMP" "$HERE/Verify.java"
for path in "$HERE"/*.class; do
  name=$(basename "$path" .class)
  case "$name" in different-target-D) continue;; esac
  mkdir -p "$TMP/jadx/tests/integration/trycatch"
  cp "$FIX/fixed-aux/"*.class "$TMP/jadx/tests/integration/trycatch/"
  if [ "$name" = different-target ]; then
    cp "$HERE/different-target-D.class" "$TMP/jadx/tests/integration/trycatch/TestTryCatchFinally5\$TestCls\$D.class"
  fi
  cp "$path" "$TMP/jadx/tests/integration/trycatch/TestTryCatchFinally5\$TestCls.class"
  printf '%s ' "$name"
  java -Xverify:all -cp "$TMP" Verify
  digest=$(shasum -a 256 "$path" | cut -d ' ' -f 1)
  printf '%s  %s.class\n' "$digest" "$name"
done
