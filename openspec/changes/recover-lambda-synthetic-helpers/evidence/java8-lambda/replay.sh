#!/bin/sh
set -eu
MODE=${1:-fixed}
case "$MODE" in baseline|fixed) ;; *) echo "usage: $0 [baseline|fixed]" >&2; exit 2;; esac
: "${JARDE_CLI:?set JARDE_CLI to the CLI binary for the selected run}"
HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
JADX_REPO=${JADX_REPO:-/Users/lordcasser/workspace/testzone/jadx}
JADX=${JADX_CLI:-$JADX_REPO/jadx-cli/build/install/jadx/bin/jadx}
JADX_REV=2fb1b16386941660fda07e9017285aec40fcb37f
ACTUAL_REV=$(git -C "$JADX_REPO" rev-parse HEAD)
test "$ACTUAL_REV" = "$JADX_REV" || { echo "expected JADX $JADX_REV, found $ACTUAL_REV" >&2; exit 1; }
OUT="$HERE/$MODE"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/dt25-lambda.XXXXXX")
trap 'rm -rf "$TMP"' EXIT
STAGE="$TMP/output"
mkdir -p "$OUT" "$STAGE" "$TMP/original" "$TMP/jadx" "$TMP/jarde"
cp "$HERE/LambdaFixture.java" "$HERE/Runner.java" "$TMP/original/"
cp "$HERE/LambdaFixture.java" "$HERE/Runner.java" "$TMP/"
(
  cd "$TMP/original"
  javac --release 8 LambdaFixture.java Runner.java
  java -Xverify:all Runner > "$STAGE/original-run.txt"
  javap -p -c LambdaFixture > "$STAGE/original-javap.txt"
  cp LambdaFixture.class "$STAGE/LambdaFixture.class"
)
"$JADX" -d "$TMP/jadx" "$TMP/original/LambdaFixture.class" > "$STAGE/jadx-decompile.txt" 2>&1
JADX_SOURCE=$(find "$TMP/jadx" -name LambdaFixture.java -print -quit)
JADX_DIR=$(dirname "$JADX_SOURCE")
sed '1i\
package defpackage;
' "$HERE/Runner.java" > "$JADX_DIR/Runner.java"
cp "$JADX_SOURCE" "$STAGE/jadx-LambdaFixture.java"
cp "$JADX_DIR/Runner.java" "$STAGE/jadx-Runner.java"
(
  cd "$JADX_DIR"
  mkdir classes
  javac --release 8 -d classes LambdaFixture.java Runner.java
  java -Xverify:all -cp classes defpackage.Runner > "$STAGE/jadx-run.txt"
)
"$JARDE_CLI" class-source --input "$TMP/original/LambdaFixture.class" --class LambdaFixture --policy single-class --release 8 --format text > "$TMP/jarde-LambdaFixture.java" 2> "$TMP/jarde-bookkeeping.txt"
sed -E 's/(usage\.elapsed_millis = )[0-9]+/\1<normalized>/' "$TMP/jarde-bookkeeping.txt" > "$STAGE/jarde-bookkeeping.txt"
cp "$TMP/jarde-LambdaFixture.java" "$STAGE/jarde-LambdaFixture.java"
cp "$HERE/Runner.java" "$STAGE/jarde-Runner.java"
cp "$HERE/Runner.java" "$TMP/jarde/Runner.java"
cp "$TMP/jarde-LambdaFixture.java" "$TMP/jarde/LambdaFixture.java"
(
  cd "$TMP/jarde"
  mkdir classes
  if javac --release 8 -d classes LambdaFixture.java Runner.java > "$TMP/jarde-javac.txt" 2>&1; then
    java -Xverify:all -cp classes Runner > "$STAGE/jarde-run.txt"
    cp "$TMP/jarde-javac.txt" "$STAGE/jarde-javac.txt"
    JARDE_STATUS=PASS
  else
    cp "$TMP/jarde-javac.txt" "$STAGE/jarde-javac.txt"
    if test "$MODE" = fixed; then cat "$OUT/jarde-javac.txt" >&2; exit 1; fi
    printf '%s\n' 'not-run: baseline source did not compile' > "$STAGE/jarde-run.txt"
    JARDE_STATUS=EXPECTED_BASELINE_FAILURE
  fi
)
if grep -q '^not-run:' "$STAGE/jarde-run.txt"; then JARDE_STATUS=EXPECTED_BASELINE_FAILURE; else JARDE_STATUS=PASS; fi
cmp "$STAGE/original-run.txt" "$STAGE/jadx-run.txt"
if test "$MODE" = fixed; then
  cmp "$STAGE/original-run.txt" "$STAGE/jarde-run.txt"
  if grep -E '^[[:space:]]*(private|static).*lambda\$|return.*lambda\$' "$STAGE/jarde-LambdaFixture.java"; then
    echo "fixed source still declares or calls a compiler helper" >&2
    exit 1
  fi
fi
(
  cd "$STAGE"
  shasum -a 256 LambdaFixture.class ../LambdaFixture.java ../Runner.java > source-sha256.txt
  shasum -a 256 jadx-LambdaFixture.java jadx-Runner.java jarde-LambdaFixture.java jarde-Runner.java > output-sha256.txt
)
printf '%s\n' "mode=$MODE" "JADX revision=$ACTUAL_REV" "javac --release 8: original/JADX PASS; Jarde=$JARDE_STATUS" "java -Xverify:all: original/JADX PASS; Jarde=$JARDE_STATUS" "stdout: $(paste -sd, "$STAGE/original-run.txt")" > "$STAGE/results.txt"
cp "$STAGE"/* "$OUT/"
