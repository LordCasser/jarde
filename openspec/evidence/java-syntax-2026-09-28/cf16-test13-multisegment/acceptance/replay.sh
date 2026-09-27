#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 JARDE_CLI [OUTPUT_DIR]" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
BASE="$(cd "$HERE/.." && pwd)"
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
OUT="${2:-$(mktemp -d /tmp/cf16-test13-acceptance.XXXXXX)}"
JADX="${JADX:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
CLASS='jadx.tests.integration.trycatch.TestTryCatchFinally13$TestCls'
RUNNER=jadx.tests.integration.trycatch.ProbeRunner
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
WORK="$(mktemp -d /tmp/cf16-test13-acceptance-work.XXXXXX)"
trap 'find "$WORK" -depth -delete' EXIT
mkdir -p "$OUT" "$WORK/original" "$WORK/jadx" "$WORK/jadx-classes" "$WORK/jarde-classes"
if [[ ! -x "$CLI" || ! -x "$JADX" ]]; then
  echo "Jarde CLI or pinned JADX launcher is unavailable" >&2
  exit 3
fi
if [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_JADX_HEAD" ]]; then
  echo "pinned JADX HEAD mismatch" >&2
  exit 3
fi

javac --release 8 -g -d "$WORK/original" "$HERE/TestTryCatchFinally13\$TestCls.java" "$BASE/ProbeRunner.java" > "$OUT/original-javac.stdout" 2> "$OUT/original-javac.stderr"
ORIGINAL_CLASS="$WORK/original/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class"
cmp "$ORIGINAL_CLASS" "$HERE/TestTryCatchFinally13\$TestCls.class"
javap -classpath "$WORK/original" -c -v "$CLASS" > "$OUT/acceptance.javap.txt"
python3 - "$BASE/fixed-class.javap.txt" "$OUT/acceptance.javap.txt" > "$OUT/shape.txt" <<'PY'
import re, sys
from pathlib import Path

def method(path):
    lines = Path(path).read_text().splitlines()
    start = next(i for i, line in enumerate(lines) if re.match(r"\s+public void test\(int\);", line))
    code, table, in_table = [], [], False
    for line in lines[start:]:
        if "Exception table:" in line:
            in_table = True
            continue
        if in_table:
            match = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$", line)
            if match:
                table.append(tuple(match.groups()))
            elif table:
                break
            continue
        match = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line)
        if match:
            code.append((int(match.group(1)), match.group(2)))
    return code, table

fixed, acceptance = map(method, sys.argv[1:])
if fixed != acceptance:
    raise SystemExit(f"Test13 test(I)V BCI/opcode/rows differ: {fixed} != {acceptance}")
if acceptance[1] != [
    ('0', '10', '44', 'Class java/lang/Exception'),
    ('15', '37', '44', 'Class java/lang/Exception'),
    ('0', '10', '56', 'any'),
    ('15', '37', '56', 'any'),
    ('44', '49', '56', 'any'),
]:
    raise SystemExit(f"unexpected five-row table: {acceptance[1]}")
print("fixed_and_acceptance_bci_opcode_rows_equal=true")
PY

java -Xverify:all -cp "$WORK/original" "$RUNNER" > "$OUT/original.run.txt"
"$JADX" --no-res -d "$WORK/jadx" "$ORIGINAL_CLASS" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
JADX_SOURCE="$WORK/jadx/sources/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.java"
cp "$JADX_SOURCE" "$OUT/jadx.java.txt"
javac --release 8 -g -d "$WORK/jadx-classes" "$JADX_SOURCE" "$BASE/ProbeRunner.java" > "$OUT/jadx-javac.stdout" 2> "$OUT/jadx-javac.stderr"
java -Xverify:all -cp "$WORK/jadx-classes" "$RUNNER" > "$OUT/jadx.run.txt"

"$CLI" class-source --input "$ORIGINAL_CLASS" --class "$CLASS" --policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
cp "$OUT/jarde.java.txt" "$WORK/TestTryCatchFinally13\$TestCls.java"
javac --release 8 -g -d "$WORK/jarde-classes" "$WORK/TestTryCatchFinally13\$TestCls.java" "$BASE/ProbeRunner.java" > "$OUT/jarde-javac.stdout" 2> "$OUT/jarde-javac.stderr"
java -Xverify:all -cp "$WORK/jarde-classes" "$RUNNER" > "$OUT/jarde.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jadx.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jarde.run.txt"
python3 - "$OUT/jarde.java.txt" <<'PY'
import sys
from pathlib import Path
source = Path(sys.argv[1]).read_text()
start = source.index('public void test(int i) {')
end = source.index('void logError()', start)
method = source[start:end]
if method.count('finally {') != 1 or method.count('this.doSomething4();') != 1:
    raise SystemExit('Test13 is not a single-source finally')
if '@bytecode' in method or 'not recovered' in method:
    raise SystemExit('Test13 has an incomplete source marker')
PY
python3 "$HERE/mutate_external_entry.py" "$ORIGINAL_CLASS" "$OUT/external-entry.class" > "$OUT/external-entry.mutation.txt"
cmp "$OUT/external-entry.class" "$HERE/external-entry.class"
mkdir -p "$WORK/external/jadx/tests/integration/trycatch"
cp "$OUT/external-entry.class" "$WORK/external/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class"
javac --release 8 -cp "$WORK/external" -d "$WORK/external" "$HERE/VerifyEarly.java" > "$OUT/external-javac.stdout" 2> "$OUT/external-javac.stderr"
java -Xverify:all -cp "$WORK/external" jadx.tests.integration.trycatch.VerifyEarly > "$OUT/external-verify.stdout" 2> "$OUT/external-verify.stderr"
"$CLI" class-source --input "$OUT/external-entry.class" --class "$CLASS" --policy single-class --release 8 --format text > "$OUT/external-entry.jarde.java.txt" 2> "$OUT/external-entry.jarde.report.txt"
python3 - "$OUT/external-entry.jarde.java.txt" <<'PY'
import sys
from pathlib import Path
source = Path(sys.argv[1]).read_text()
start = source.index('public void test(int i) {')
end = source.index('void logError()', start)
method = source[start:end]
if 'finally {' in method or '@bytecode' not in method:
    raise SystemExit('external-entry graph was merged into a finally')
PY
printf 'fixed_and_acceptance_bci_opcode_rows_equal=true\noriginal_equals_jadx=true\noriginal_equals_jarde=true\nall_three_java_Xverify_all=true\nexternal_entry_verifier_valid_and_refused=true\n' > "$OUT/results.txt"
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
