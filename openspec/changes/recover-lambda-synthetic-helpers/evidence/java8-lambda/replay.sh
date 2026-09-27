#!/bin/sh
set -eu
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
JADX=/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx
JARDE=/tmp/jarde-dt21-audit-target/debug/jarde-cli
TMP=$(mktemp -d "${TMPDIR:-/tmp}/dt25-lambda.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
mkdir -p "$TMP/original" "$TMP/jadx" "$TMP/jarde" "$HERE/original-classes"
cp "$HERE/LambdaFixture.java" "$HERE/Runner.java" "$TMP/original/"
(
  cd "$TMP/original"
  javac --release 8 LambdaFixture.java Runner.java
  java -Xverify:all Runner > "$HERE/original-run.txt"
  javap -p -c -v LambdaFixture > "$HERE/javap.txt"
  cp LambdaFixture.class Runner.class "$HERE/original-classes/"
)
"$JADX" -d "$TMP/jadx" "$TMP/original/LambdaFixture.class" > "$HERE/jadx-decompile.txt" 2>&1
JADX_SOURCE=$(find "$TMP/jadx" -name LambdaFixture.java -print -quit)
JADX_DIR=$(dirname "$JADX_SOURCE")
sed '1i\
package defpackage;
' "$HERE/Runner.java" > "$JADX_DIR/Runner.java"
cp "$JADX_SOURCE" "$HERE/jadx-LambdaFixture.java"
(
  cd "$JADX_DIR"
  mkdir classes
  javac --release 8 -d classes LambdaFixture.java Runner.java
  java -Xverify:all -cp classes defpackage.Runner > "$HERE/jadx-run.txt"
)
"$JARDE" class-source --input "$TMP/original/LambdaFixture.class" --class LambdaFixture --policy single-class --release 8 --format text > "$HERE/jarde-class-source.txt" 2> "$HERE/jarde-bookkeeping.txt"
cp "$HERE/Runner.java" "$TMP/jarde/"
cp "$HERE/jarde-class-source.txt" "$TMP/jarde/LambdaFixture.java"
(
  cd "$TMP/jarde"
  mkdir classes
  javac --release 8 -d classes LambdaFixture.java Runner.java > "$HERE/jarde-javac.txt" 2>&1 || echo "javac-exit=$?" >> "$HERE/jarde-javac.txt"
  if test -f classes/Runner.class; then
    java -Xverify:all -cp classes Runner > "$HERE/jarde-run.txt" 2>&1 || echo "java-exit=$?" >> "$HERE/jarde-run.txt"
  else
    echo "not-run: javac did not produce Runner.class" > "$HERE/jarde-run.txt"
  fi
)
(
  cd "$HERE"
  shasum -a 256 LambdaFixture.java Runner.java > sha256.txt
  shasum -a 256 jadx-LambdaFixture.java jarde-class-source.txt > output-sha256.txt
  shasum -a 256 "$JARDE" > jarde-cli.sha256
  shasum -a 256 "$JADX" > jadx-launcher.sha256
)
(
  cd "$HERE"
  shasum -a 256 original-classes/LambdaFixture.class original-classes/Runner.class >> "$HERE/sha256.txt"
)
