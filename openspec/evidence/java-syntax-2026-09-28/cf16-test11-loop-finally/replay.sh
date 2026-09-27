#!/usr/bin/env bash
set -euo pipefail

if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 JARDE_CLI [OUTPUT_DIR]" >&2
  exit 2
fi

HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
if [[ $# -eq 2 ]]; then
  OUT="$2"
else
  OUT="$(mktemp -d "${TMPDIR:-/tmp}/cf16-test11-loop.XXXXXX")"
fi
mkdir -p "$OUT"

EXPECTED_TEST_SHA=3e077dc69f9325fa55dec9ba92ba14a508b2ae4e6ce3d05aeb3462f9c2df1bf5
EXPECTED_CLASS_SHA=3a67a7f63596c5ca7b7dbdb97a2027224d69f2a1d7b21e4f598d897510bc5f92
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
PINNED_TEST="$JADX_ROOT/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally11.java"
JADX="${JADX:-$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx}"
JAVA="${JAVA:-$(command -v java)}"
JAVAC="${JAVAC:-$(command -v javac)}"
JAVAP="${JAVAP:-$(command -v javap)}"

if [[ ! -x "$CLI" || ! -x "$JADX" ]]; then
  echo "JARDE_CLI and JADX must name executable launchers" >&2
  exit 3
fi
if [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_JADX_HEAD" ]]; then
  echo "pinned JADX checkout HEAD mismatch" >&2
  exit 3
fi
if [[ "$(shasum -a 256 "$PINNED_TEST" | awk '{print $1}')" != "$EXPECTED_TEST_SHA" ]]; then
  echo "pinned TestTryCatchFinally11.java SHA-256 mismatch" >&2
  exit 3
fi
if [[ "$(shasum -a 256 "$HERE/pinned/TestTryCatchFinally11.java" | awk '{print $1}')" != "$EXPECTED_TEST_SHA" ]]; then
  echo "frozen TestTryCatchFinally11.java SHA-256 mismatch" >&2
  exit 3
fi
if [[ "$(shasum -a 256 "$HERE/TestTryCatchFinally11\$TestCls.class" | awk '{print $1}')" != "$EXPECTED_CLASS_SHA" ]]; then
  echo "pinned TestCls class SHA-256 mismatch" >&2
  exit 3
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/cf16-test11-loop-work.XXXXXX")"
trap 'find "$TMP" -depth -delete' EXIT
mkdir -p "$TMP/probe-original" "$TMP/probe-jadx" "$TMP/probe-jarde" \
  "$TMP/testcls-original" "$TMP/testcls-jadx" "$TMP/testcls-jarde" \
  "$TMP/jadx-probe" "$TMP/jadx-testcls"

JARDE_SHA="$(shasum -a 256 "$CLI" | awk '{print $1}')"
JADX_SHA="$(shasum -a 256 "$JADX" | awk '{print $1}')"
printf 'jarde_checkout_head=%s\njarde_cli=%s\njarde_cli_version=%s\njarde_cli_sha256=%s\njadx_checkout_head=%s\njadx=%s\njadx_version=%s\njadx_binary_sha256=%s\njavac=%s\njava=%s\n' \
  "$(git -C "$ROOT" rev-parse HEAD)" "$CLI" "$($CLI --version 2>&1 | tail -n 1)" "$JARDE_SHA" \
  "$EXPECTED_JADX_HEAD" "$JADX" "$($JADX --version 2>&1 | head -n 1)" "$JADX_SHA" "$JAVAC" "$JAVA" > "$OUT/toolchain.txt"
printf 'pinned_test_source_sha256=%s\npinned_testcls_class_sha256=%s\n' "$EXPECTED_TEST_SHA" "$EXPECTED_CLASS_SHA" >> "$OUT/toolchain.txt"

# The pinned class is the original target. Preserve its full bytecode listing.
mkdir -p "$TMP/testcls-original/jadx/tests/integration/trycatch"
cp "$HERE/TestTryCatchFinally11\$TestCls.class" "$TMP/testcls-original/jadx/tests/integration/trycatch/"
"$JAVAP" -classpath "$TMP/testcls-original" -c -v 'jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls' > "$OUT/javap-TestCls.txt"
"$JAVAP" -classpath "$TMP/testcls-original" -verbose 'jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls' | grep -q 'major version: 55'

# The supplied probe differs only in private-call opcodes from the pinned test
# method; the instruction addresses and exception-table edges must match.
"$JAVAC" --release 8 -Xlint:-options -g -d "$TMP/probe-original" "$HERE/FinallyLoop.java" "$HERE/FinallyLoopRunner.java" \
  > "$OUT/probe-original-javac.stdout" 2> "$OUT/probe-original-javac.stderr"
"$JAVAP" -classpath "$TMP/probe-original" -c -v FinallyLoop > "$OUT/javap-FinallyLoop.txt"
"$JAVA" -Xverify:all -cp "$TMP/probe-original" FinallyLoopRunner > "$OUT/probe-original-run.txt"
"$JAVA" -Xverify:all -cp "$TMP/probe-original" FinallyLoopRunner fail >> "$OUT/probe-original-run.txt"
python3 - "$OUT/javap-TestCls.txt" "$OUT/javap-FinallyLoop.txt" > "$OUT/bytecode-shape.txt" <<'PY'
import re
import sys
from pathlib import Path

def method(path, marker):
    lines = Path(path).read_text().splitlines()
    start = next(i for i, line in enumerate(lines) if marker in line)
    end = next((i for i in range(start + 1, len(lines)) if re.match(r"\s+(?:public|private|protected) ", lines[i])), len(lines))
    code, table, in_table = [], [], False
    for line in lines[start:end]:
        match = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line)
        if match:
            code.append((int(match.group(1)), match.group(2)))
        if "Exception table:" in line:
            in_table = True
            continue
        if in_table:
            match = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$", line)
            if match:
                table.append(tuple(match.groups()))
            elif table and line.strip():
                in_table = False
    return code, table

fixed = method(sys.argv[1], "public void test(java.util.List<java.lang.Object>);")
probe = method(sys.argv[2], "public void test(java.util.List<java.lang.Object>);")
expected_bci = [0, 1, 4, 5, 10, 11, 12, 17, 20, 21, 26, 27, 28, 29, 32, 35, 38, 40, 41, 46, 48, 50, 55, 58, 60, 65, 67, 68, 70, 73, 76, 78, 79]
expected_table = [("0", "4", "38", "any"), ("38", "40", "38", "any")]
for name, item in (("pinned", fixed), ("probe", probe)):
    if [bci for bci, _ in item[0]] != expected_bci or item[1] != expected_table:
        raise SystemExit(f"{name} bytecode addresses or exception table mismatch: {item}")
fixed_ops, probe_ops = dict(fixed[0]), dict(probe[0])
differences = {bci: (fixed_ops[bci], probe_ops[bci]) for bci in fixed_ops if fixed_ops[bci] != probe_ops[bci]}
if differences != {1: ("invokevirtual", "invokespecial"), 29: ("invokevirtual", "invokespecial"), 70: ("invokevirtual", "invokespecial")}:
    raise SystemExit(f"unexpected opcode differences: {differences}")
print("pinned_bci_0_to_79=true")
print("pinned_exception_table=[0,4)->38 any;[38,40)->38 any")
print("probe_instruction_addresses_and_exception_table_match=true")
print("documented_opcode_differences=1,29,70:invokevirtual/invokespecial")
PY

# Replay the exact pinned nested class from its class file and both complete
# source presentations. A tiny assertion-helper stand-in satisfies only the
# unrelated `check()` method dependency in the decompiler source.
"$JAVAC" --release 8 -Xlint:-options -cp "$TMP/testcls-original" -d "$TMP/testcls-original" "$HERE/PinnedTestRunner.java" \
  > "$OUT/testcls-original-javac.stdout" 2> "$OUT/testcls-original-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/testcls-original" PinnedTestRunner 'jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls' > "$OUT/testcls-original-run.txt"

"$JADX" --no-res -d "$TMP/jadx-testcls" "$HERE/TestTryCatchFinally11\$TestCls.class" > "$OUT/jadx-testcls.stdout" 2> "$OUT/jadx-testcls.stderr"
JADX_TESTCLS_SOURCE="$TMP/jadx-testcls/sources/jadx/tests/integration/trycatch/TestTryCatchFinally11\$TestCls.java"
cp "$JADX_TESTCLS_SOURCE" "$OUT/TestTryCatchFinally11\$TestCls.jadx.java"
"$JAVAC" --release 8 -Xlint:-options -d "$TMP/testcls-jadx" "$JADX_TESTCLS_SOURCE" "$HERE/pinned/JadxAssertions.java" "$HERE/PinnedTestRunner.java" \
  > "$OUT/testcls-jadx-javac.stdout" 2> "$OUT/testcls-jadx-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/testcls-jadx" PinnedTestRunner 'jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls' > "$OUT/testcls-jadx-run.txt"

"$CLI" class-source --input "$HERE/TestTryCatchFinally11\$TestCls.class" --class 'jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls' \
  --policy single-class --release 8 --format text > "$OUT/TestTryCatchFinally11\$TestCls.jarde.java" 2> "$OUT/jarde-testcls.report.txt"
grep -Fq 'the graph is not reducible over 2 block(s) [48, 58]' "$OUT/TestTryCatchFinally11\$TestCls.jarde.java"
grep -Fq 'explanation only' "$OUT/TestTryCatchFinally11\$TestCls.jarde.java"
mkdir -p "$TMP/testcls-jarde/jadx/tests/integration/trycatch"
cp "$OUT/TestTryCatchFinally11\$TestCls.jarde.java" "$TMP/testcls-jarde/jadx/tests/integration/trycatch/TestTryCatchFinally11\$TestCls.java"
"$JAVAC" --release 8 -Xlint:-options -d "$TMP/testcls-jarde" "$TMP/testcls-jarde/jadx/tests/integration/trycatch/TestTryCatchFinally11\$TestCls.java" "$HERE/pinned/JadxAssertions.java" "$HERE/PinnedTestRunner.java" \
  > "$OUT/testcls-jarde-javac.stdout" 2> "$OUT/testcls-jarde-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/testcls-jarde" PinnedTestRunner 'jadx.tests.integration.trycatch.TestTryCatchFinally11$TestCls' > "$OUT/testcls-jarde-run.txt"

# Behavioral control probe: normal return and a body exception both traverse
# the finally loop, and the Jarde body remains explicitly explanation-only.
"$JADX" --no-res -d "$TMP/jadx-probe" "$TMP/probe-original/FinallyLoop.class" > "$OUT/jadx-probe.stdout" 2> "$OUT/jadx-probe.stderr"
JADX_PROBE_SOURCE="$(find "$TMP/jadx-probe/sources" -name FinallyLoop.java -print -quit)"
cp "$JADX_PROBE_SOURCE" "$OUT/FinallyLoop.jadx.java"
mkdir -p "$TMP/probe-jadx-src/defpackage"
cp "$OUT/FinallyLoop.jadx.java" "$TMP/probe-jadx-src/defpackage/FinallyLoop.java"
"$JAVAC" --release 8 -Xlint:-options -g -d "$TMP/probe-jadx" "$TMP/probe-jadx-src/defpackage/FinallyLoop.java" "$HERE/FinallyLoopRunnerJadx.java" \
  > "$OUT/probe-jadx-javac.stdout" 2> "$OUT/probe-jadx-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/probe-jadx" defpackage.FinallyLoopRunnerJadx > "$OUT/probe-jadx-run.txt"
"$JAVA" -Xverify:all -cp "$TMP/probe-jadx" defpackage.FinallyLoopRunnerJadx fail >> "$OUT/probe-jadx-run.txt"

"$CLI" class-source --input "$TMP/probe-original/FinallyLoop.class" --class FinallyLoop --policy single-class --release 8 --format text \
  > "$OUT/FinallyLoop.jarde.java" 2> "$OUT/jarde-probe.report.txt"
grep -Fq 'the graph is not reducible over 2 block(s) [48, 58]' "$OUT/FinallyLoop.jarde.java"
grep -Fq 'explanation only' "$OUT/FinallyLoop.jarde.java"
mkdir -p "$TMP/probe-jarde-src"
cp "$OUT/FinallyLoop.jarde.java" "$TMP/probe-jarde-src/FinallyLoop.java"
"$JAVAC" --release 8 -Xlint:-options -g -d "$TMP/probe-jarde" "$TMP/probe-jarde-src/FinallyLoop.java" "$HERE/FinallyLoopRunner.java" \
  > "$OUT/probe-jarde-javac.stdout" 2> "$OUT/probe-jarde-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/probe-jarde" FinallyLoopRunner > "$OUT/probe-jarde-run.txt"
"$JAVA" -Xverify:all -cp "$TMP/probe-jarde" FinallyLoopRunner fail >> "$OUT/probe-jarde-run.txt"

printf 'two:102\nempty:100\n' > "$OUT/testcls-expected.txt"
printf 'ok:102\nthrow:102:body\n' > "$OUT/probe-expected.txt"
printf 'ok:0\nok:0\n' > "$OUT/probe-jarde-expected.txt"
cmp "$OUT/testcls-original-run.txt" "$OUT/testcls-expected.txt"
cmp "$OUT/testcls-jadx-run.txt" "$OUT/testcls-expected.txt"
cmp "$OUT/probe-jadx-run.txt" "$OUT/probe-expected.txt"
cmp "$OUT/probe-jarde-run.txt" "$OUT/probe-jarde-expected.txt"
cmp "$OUT/probe-original-run.txt" "$OUT/probe-expected.txt"

cat > "$OUT/results.txt" <<EOF
pinned_test_source_sha256=$EXPECTED_TEST_SHA
pinned_testcls_class_sha256=$EXPECTED_CLASS_SHA
bytecode_shape_check=passed
testcls_original_java_Xverify_all=passed (two:102, empty:100)
testcls_jadx_javac_and_java_Xverify_all=passed (two:102, empty:100)
testcls_jarde_explanation_only_marker=present
testcls_jarde_javac_and_java_Xverify_all=passed (two:0, empty:0)
probe_original_java_Xverify_all=passed (ok:102, throw:102:body)
probe_jadx_javac_and_java_Xverify_all=passed (ok:102, throw:102:body)
probe_jarde_explanation_only_marker=present
probe_jarde_javac_and_java_Xverify_all=passed (ok:0, ok:0)
behavior_claim=Jarde source is a compilable presentation of a safely refused method; runtime mismatch is not claimed as correct recovery
EOF
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
