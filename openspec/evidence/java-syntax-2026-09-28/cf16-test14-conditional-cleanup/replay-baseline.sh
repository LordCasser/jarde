#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 JARDE_CLI [OUTPUT_DIR]" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
OUT="${2:-$(mktemp -d /tmp/jarde-cf16-test14-output.XXXXXX)}"
WORK="$(mktemp -d /tmp/jarde-cf16-test14-work.XXXXXX)"
trap 'find "$WORK" -depth -delete' EXIT
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
EXPECTED_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_CLASS=8857b84944f1a0c8ec0d8805d2ba0e7430dfadfda7629b4a14ea4064eb1d4ece
CLASS='jadx.tests.integration.trycatch.TestTryCatchFinally14$TestCls'
RUNNER=jadx.tests.integration.trycatch.Runner

if [[ ! -x "$CLI" || ! -x "$JADX" ]] || [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_HEAD" ]]; then
  echo "fresh Jarde CLI or pinned JADX checkout unavailable" >&2
  exit 3
fi
mkdir -p "$OUT" "$WORK/original" "$WORK/jadx-classes"
javac --release 8 -g -Xlint:-options -d "$WORK/original" "$HERE/TestTryCatchFinally14.java" "$HERE/Runner.java"
INPUT="$WORK/original/jadx/tests/integration/trycatch/TestTryCatchFinally14\$TestCls.class"
cmp "$INPUT" "$HERE/TestTryCatchFinally14\$TestCls.class"
if [[ "$(shasum -a 256 "$INPUT" | awk '{print $1}')" != "$EXPECTED_CLASS" ]]; then
  echo "Test14 target class SHA changed" >&2
  exit 4
fi
javap -classpath "$WORK/original" -c -v "$CLASS" > "$OUT/test14.javap.txt"
java -Xverify:all -cp "$WORK/original" "$RUNNER" > "$OUT/original.run.txt"
jar --create --file "$WORK/test14.jar" -C "$WORK/original" .
"$JADX" --no-res -d "$WORK/jadx" "$WORK/test14.jar" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
JADX_SOURCE="$WORK/jadx/sources/jadx/tests/integration/trycatch/TestTryCatchFinally14.java"
cp "$JADX_SOURCE" "$OUT/jadx.java.txt"
javac --release 8 -g -Xlint:-options -d "$WORK/jadx-classes" "$JADX_SOURCE" "$HERE/Runner.java"
java -Xverify:all -cp "$WORK/jadx-classes" "$RUNNER" > "$OUT/jadx.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jadx.run.txt"
"$CLI" class-source --input "$INPUT" --class "$CLASS" --policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
python3 - "$OUT/jarde.java.txt" "$OUT/jadx.java.txt" <<'PY'
import sys
from pathlib import Path
jarde, jadx = (Path(path).read_text() for path in sys.argv[1:])
target = jarde[jarde.index('public void test() {'):jarde.index('static jadx.tests.integration.trycatch', jarde.index('public void test() {'))]
if 'not recovered' not in target or '@bytecode 0 1 4' not in target:
    raise SystemExit('baseline Jarde Test14 refusal changed')
if 'finally {' not in jadx or jadx.count('.doFinally();') != 1:
    raise SystemExit('pinned JADX did not emit one conditional finally')
PY
printf 'target_class_sha256=%s\noriginal_equals_jadx_seven_paths=true\njarde_test_method_refused=true\n' "$EXPECTED_CLASS" > "$OUT/results.txt"
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
