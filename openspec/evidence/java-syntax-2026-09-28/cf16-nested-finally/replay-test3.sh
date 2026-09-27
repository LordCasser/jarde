#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$2"
"$HERE/replay.sh" "$1" "$OUT"
TMP="$(mktemp -d "${TMPDIR:-/tmp}/cf16-test3.XXXXXX")"
trap 'find "$TMP" -depth -delete' EXIT
mkdir -p "$TMP/source-jadx" "$TMP/source-jarde" "$TMP/original" "$TMP/jadx" "$TMP/jarde" "$TMP/verify"
cp "$OUT/FinallyMinimalProbe.jadx.java" "$TMP/source-jadx/FinallyMinimalProbe.java"
cp "$OUT/jarde.java.txt" "$TMP/source-jarde/FinallyMinimalProbe.java"
for pair in "original:$HERE/FinallyMinimalProbe.java" "jadx:$TMP/source-jadx/FinallyMinimalProbe.java" "jarde:$TMP/source-jarde/FinallyMinimalProbe.java"; do
  kind="${pair%%:*}"
  source="${pair#*:}"
  javac --release 8 -Xlint:-options -d "$TMP/$kind" "$source" "$HERE/Test3Runner.java"
  java -Xverify:all -cp "$TMP/$kind" jadx.tests.integration.trycatch.Test3Runner > "$OUT/test3-$kind-run.txt"
done
cmp "$OUT/test3-original-run.txt" "$OUT/test3-jadx-run.txt"
cmp "$OUT/test3-original-run.txt" "$OUT/test3-jarde-run.txt"

python3 "$HERE/test3-mutants.py"
javac --release 8 -Xlint:-options -d "$TMP/verify" "$HERE/VerifyTest3.java"
for variant in 'TestTryCatchFinally12$TestCls' Test3MatchingOtherConstant Test3DifferentConstant Test3DifferentField Test3DifferentTarget Test3StoredResult Test3WidenedRow; do
  mkdir -p "$TMP/verify/jadx/tests/integration/trycatch"
  cp "$HERE/$variant.class" "$TMP/verify/jadx/tests/integration/trycatch/TestTryCatchFinally12\$TestCls.class"
  expected=call-finally
  if [[ "$variant" == Test3DifferentConstant ]]; then expected=call-catch; fi
  npe=call-npe-catch-finally
  iae=call-iae-finally
  if [[ "$variant" == Test3MatchingOtherConstant ]]; then
    expected=call-catch
    npe=call-npe-catch-catch
    iae=call-iae-catch
  fi
  java -Xverify:all -cp "$TMP/verify" VerifyTest3 "$expected" "$npe" "$iae" > "$OUT/$variant.run.txt"
done
printf 'test3_original_jadx_jarde_equal=true\nfixed_and_six_variants_java_Xverify_all=true\n' > "$OUT/test3-results.txt"
cat "$OUT/test3-results.txt"
