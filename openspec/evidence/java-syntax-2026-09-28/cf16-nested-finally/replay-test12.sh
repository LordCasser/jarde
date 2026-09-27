#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$2"
"$HERE/replay.sh" "$1" "$OUT"
grep -qx 'jarde_equals_original=true' "$OUT/results.txt"
grep -qx 'jarde_javac_exit=0' "$OUT/results.txt"
grep -qx 'jarde_java_Xverify_all_exit=0' "$OUT/results.txt"
cmp "$OUT/original-run.txt" "$OUT/jadx-run.txt"
cmp "$OUT/original-run.txt" "$OUT/jarde-run.txt"

TMP="$(mktemp -d "${TMPDIR:-/tmp}/cf16-test12.XXXXXX")"
trap 'find "$TMP" -depth -delete' EXIT
mkdir -p "$TMP/jadx/tests/integration/trycatch"
python3 "$HERE/test12-mutants.py"
javac --release 8 -Xlint:-options -d "$TMP" "$HERE/VerifyTest12.java"
: > "$OUT/test12-class-sha256.txt"

for variant in 'TestTryCatchFinally12$TestCls' \
  Test1DifferentConstant Test1WidenedRow Test1BypassCleanup Test1ChangedRethrow \
  Test2DifferentConstant Test2WidenedRow Test2BypassCleanup Test2ChangedRethrow; do
  cp "$HERE/$variant.class" "$TMP/jadx/tests/integration/trycatch/TestTryCatchFinally12\$TestCls.class"
  java -Xverify:all -cp "$TMP" VerifyTest12 > "$OUT/$variant.test12-run.txt"
  shasum -a 256 "$HERE/$variant.class" >> "$OUT/test12-class-sha256.txt"
done
printf 'original_jadx_jarde_nine_paths_equal=true\nfixed_and_eight_test12_variants_java_Xverify_all=true\n' > "$OUT/test12-results.txt"
cat "$OUT/test12-results.txt"
