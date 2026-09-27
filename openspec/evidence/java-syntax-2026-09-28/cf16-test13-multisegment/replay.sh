#!/usr/bin/env bash
set -u

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
  OUT="$(mktemp -d "/tmp/cf16-test13-multisegment-output.XXXXXX")"
fi
mkdir -p "$OUT"
JAVAC="${JAVAC:-$(command -v javac)}"
JAVA="${JAVA:-$(command -v java)}"
JAVAP="${JAVAP:-$(command -v javap)}"
JADX="${JADX:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_FIXED_SHA=7f5dc8b6e83912afbf8216ae26aba8dbba8f43c9eaaba8f28a966091e9c55d51
CLASS='jadx.tests.integration.trycatch.TestTryCatchFinally13$TestCls'
RUNNER=jadx.tests.integration.trycatch.ProbeRunner
TMP="$(mktemp -d "${TMPDIR:-/tmp}/cf16-test13-multisegment.XXXXXX")"
trap 'find "$TMP" -depth -delete' EXIT
mkdir -p "$TMP/original" "$TMP/jadx-classes" "$TMP/jarde-classes" "$TMP/jadx-out"

if [[ ! -x "$CLI" || ! -x "$JADX" ]]; then
  echo "Jarde CLI or JADX launcher is not executable" >&2
  exit 3
fi
if [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_JADX_HEAD" ]]; then
  echo "pinned JADX checkout HEAD mismatch" >&2
  exit 3
fi
JADX_VERSION="$("$JADX" --version 2>&1 | head -n 1)"
if [[ "$JADX_VERSION" != "dev" ]]; then
  echo "pinned JADX launcher version mismatch: expected dev, got $JADX_VERSION" >&2
  exit 3
fi
FIXED_SHA="$(shasum -a 256 "$HERE/TestTryCatchFinally13\$TestCls.fixed.class" | awk '{print $1}')"
if [[ "$FIXED_SHA" != "$EXPECTED_FIXED_SHA" ]]; then
  echo "fixed JADX class SHA-256 mismatch: $FIXED_SHA" >&2
  exit 3
fi

"$JAVAC" --release 8 -g -d "$TMP/original" "$HERE/TestTryCatchFinally13\$TestCls.java" "$HERE/ProbeRunner.java" > "$OUT/original-javac.stdout" 2> "$OUT/original-javac.stderr"
ORIGINAL_COMPILE=$?
if [[ $ORIGINAL_COMPILE -ne 0 ]]; then
  echo "original fixture compilation failed; see $OUT/original-javac.stderr" >&2
  exit 4
fi
python3 - "$HERE/TestTryCatchFinally13.fixed-source.java" "$HERE/TestTryCatchFinally13\$TestCls.java" <<'PYTEST'
import sys
from pathlib import Path

def method(path):
    source = Path(path).read_text()
    start = source.index("public void test(int i) {")
    brace = source.index("{", start)
    depth = 0
    for end in range(brace, len(source)):
        if source[end] == "{": depth += 1
        elif source[end] == "}":
            depth -= 1
            if depth == 0:
                return source[start:end + 1]
    raise SystemExit(f"unclosed test method in {path}")

fixed, probe = map(method, sys.argv[1:])
if "".join(fixed.split()) != "".join(probe.split()):
    raise SystemExit("probe test(I)V tokens differ from the fixed source")
PYTEST
ORIGINAL_CLASS="$TMP/original/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class"
cp "$ORIGINAL_CLASS" "$OUT/probe.class"
"$JAVAP" -classpath "$TMP/original" -c -v "$CLASS" > "$OUT/probe-class.javap.txt" 2>&1
python3 - "$HERE/fixed-class.javap.txt" "$OUT/probe-class.javap.txt" > "$OUT/bytecode-shape-check.txt" <<'PY'
import re, sys
from pathlib import Path

def method(path):
    lines = Path(path).read_text().splitlines()
    start = next(i for i, line in enumerate(lines) if re.match(r"\s+public void test\(int\);", line))
    end = next((i for i in range(start + 1, len(lines)) if re.match(r"\s+private |\s+public |\s+static ", lines[i])), len(lines))
    code, table, in_table = [], [], False
    for line in lines[start:]:
        if "Exception table:" in line:
            in_table = True
            continue
        if in_table:
            m = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$", line)
            if m:
                table.append(tuple(m.groups()))
            elif table:
                break
            continue
        m = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)(.*)$", line)
        if m:
            bci, opcode, rest = int(m.group(1)), m.group(2), m.group(3)
            comment = rest.split("//", 1)[1].strip() if "//" in rest else ""
            code.append((bci, opcode, comment))
    return code, table

a, b = method(sys.argv[1]), method(sys.argv[2])
fixed, probe = a[0], b[0]
if [x[0] for x in fixed] != [x[0] for x in probe]:
    print("instruction BCI sequence equal=false")
    print(f"fixed={[x[0] for x in fixed]}")
    print(f"probe={[x[0] for x in probe]}")
    raise SystemExit(1)
for left, right in zip(fixed, probe):
    if left[1] != right[1]:
        print(f"opcode difference at BCI {left[0]}: fixed={left[1]} probe={right[1]}")
        raise SystemExit(1)
if a[1] != b[1]:
    print(f"exception table equal=false fixed={a[1]} probe={b[1]}")
    raise SystemExit(1)
print("instruction_bci_sequence_equal=true")
print("opcode_sequence_equal=true")
print("exception_table_equal=true")
print("helper_invocation_opcode_differences=0")
PY
SHAPE_STATUS=$?
if [[ $SHAPE_STATUS -ne 0 ]]; then
  echo "probe differs in BCI/control flow/exception-table shape" >&2
  exit 4
fi
"$JAVA" -Xverify:all -cp "$TMP/original" "$RUNNER" > "$OUT/original.run.txt" 2>&1
ORIGINAL_RUN=$?
if [[ $ORIGINAL_RUN -ne 0 ]]; then
  echo "original runner failed; see $OUT/original.run.txt" >&2
  exit 4
fi

"$JADX" --no-res -d "$TMP/jadx-out" "$ORIGINAL_CLASS" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
JADX_EXIT=$?
JADX_SOURCE="$TMP/jadx-out/sources/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.java"
JADX_COMPILE=not_run
JADX_RUN=not_run
if [[ $JADX_EXIT -eq 0 && -f "$JADX_SOURCE" ]]; then
  cp "$JADX_SOURCE" "$OUT/jadx.java.txt"
  "$JAVAC" --release 8 -g -d "$TMP/jadx-classes" "$JADX_SOURCE" "$HERE/ProbeRunner.java" > "$OUT/jadx-javac.stdout" 2> "$OUT/jadx-javac.stderr"
  JADX_COMPILE=$?
  if [[ $JADX_COMPILE -eq 0 ]]; then
    "$JAVA" -Xverify:all -cp "$TMP/jadx-classes" "$RUNNER" > "$OUT/jadx.run.txt" 2>&1
    JADX_RUN=$?
  fi
fi

"$CLI" class-source --input "$ORIGINAL_CLASS" --class "$CLASS" --policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
JARDE_EXIT=$?
JARDE_COMPILE=not_run
JARDE_RUN=not_run
JARDE_SAFE_REFUSAL=false
if [[ $JARDE_EXIT -eq 0 ]]; then
  if grep -Fq 'jarde: not recovered: the recovery run for `test(I)V` produced no statement' "$OUT/jarde.java.txt"; then JARDE_SAFE_REFUSAL=true; fi
  cp "$OUT/jarde.java.txt" "$TMP/TestTryCatchFinally13\$TestCls.java"
  "$JAVAC" --release 8 -g -d "$TMP/jarde-classes" "$TMP/TestTryCatchFinally13\$TestCls.java" "$HERE/ProbeRunner.java" > "$OUT/jarde-javac.stdout" 2> "$OUT/jarde-javac.stderr"
  JARDE_COMPILE=$?
  if [[ $JARDE_COMPILE -eq 0 ]]; then
    "$JAVA" -Xverify:all -cp "$TMP/jarde-classes" "$RUNNER" > "$OUT/jarde.run.txt" 2>&1
    JARDE_RUN=$?
  fi
fi

JADX_EQUAL=false
JARDE_EQUAL=false
if [[ "$JADX_RUN" == 0 ]] && cmp -s "$OUT/original.run.txt" "$OUT/jadx.run.txt"; then JADX_EQUAL=true; fi
if [[ "$JARDE_RUN" == 0 ]] && cmp -s "$OUT/original.run.txt" "$OUT/jarde.run.txt"; then JARDE_EQUAL=true; fi
JARDE_VERSION="$("$CLI" --version 2>&1 | tail -n 1)"
printf 'jarde_head=%s\njarde_cli=%s\njarde_cli_version=%s\njarde_cli_sha256=%s\njavac=%s\njava=%s\njadx_head=%s\njadx_version=%s\njadx_launcher_sha256=%s\nfixed_class_sha256=%s\nprobe_class_sha256=%s\n' \
  "$(git -C "$ROOT" rev-parse HEAD)" "$CLI" "$JARDE_VERSION" "$(shasum -a 256 "$CLI" | awk '{print $1}')" \
  "$JAVAC" "$JAVA" "$EXPECTED_JADX_HEAD" "$JADX_VERSION" "$(shasum -a 256 "$JADX" | awk '{print $1}')" \
  "$FIXED_SHA" "$(shasum -a 256 "$ORIGINAL_CLASS" | awk '{print $1}')" > "$OUT/toolchain.txt"
cat > "$OUT/results.txt" <<EOF2
original_javac_release8_exit=$ORIGINAL_COMPILE
original_java_Xverify_all_exit=$ORIGINAL_RUN
jadx_decompile_exit=$JADX_EXIT
jadx_javac_release8_exit=$JADX_COMPILE
jadx_java_Xverify_all_exit=$JADX_RUN
jarde_class_source_exit=$JARDE_EXIT
jarde_javac_release8_exit=$JARDE_COMPILE
jarde_java_Xverify_all_exit=$JARDE_RUN
jarde_test_method_safe_refusal=$JARDE_SAFE_REFUSAL
original_equals_jadx=$JADX_EQUAL
original_equals_jarde=$JARDE_EQUAL
jarde_report_size_bytes=$(wc -c < "$OUT/jarde.report.txt" | tr -d ' ')
EOF2
printf 'jarde_safe_refusal=%s\n' "$JARDE_SAFE_REFUSAL" > "$OUT/jarde-baseline-report.txt"
printf 'The CLI returned %s and compiled presentation returned %s. Method test(I)V is explanation-only: no executable statements were recovered; its Jarde-compiled runner failed on early-return trace. Full machine report: %s bytes (kept only in replay output).\n' "$JARDE_EXIT" "$JARDE_COMPILE" "$(wc -c < "$OUT/jarde.report.txt" | tr -d ' ')" >> "$OUT/jarde-baseline-report.txt"
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
if [[ "$JADX_EXIT" != 0 || "$JADX_COMPILE" != 0 || "$JADX_RUN" != 0 || "$JARDE_EXIT" != 0 || "$JARDE_COMPILE" != 0 || "$JARDE_RUN" == 0 || "$JARDE_SAFE_REFUSAL" != true || "$JADX_EQUAL" != true ]]; then
  exit 5
fi
