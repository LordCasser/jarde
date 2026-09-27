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
  OUT="$(mktemp -d "${TMPDIR:-/tmp}/cf16-nested-finally-output.XXXXXX")"
fi
mkdir -p "$OUT"

EXPECTED_FIXED_TEST_CLASS_SHA=d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185
ACTUAL_HEAD="$(git -C "$ROOT" rev-parse HEAD)"
if [[ ! -x "$CLI" ]]; then
  echo "Jarde CLI is not executable: $CLI" >&2
  exit 3
fi
CLI_VERSION="$("$CLI" --version 2>&1 | tail -n 1)"
FIXED_TEST_CLASS_SHA="$(shasum -a 256 "$HERE/TestTryCatchFinally12\$TestCls.class" | awk '{print $1}')"
if [[ "$FIXED_TEST_CLASS_SHA" != "$EXPECTED_FIXED_TEST_CLASS_SHA" ]]; then
  echo "fixed TestTryCatchFinally12\$TestCls class SHA-256 mismatch: $FIXED_TEST_CLASS_SHA" >&2
  exit 3
fi

JAVAC="${JAVAC:-$(command -v javac)}"
JAVA="${JAVA:-$(command -v java)}"
JAVAP="${JAVAP:-$(command -v javap)}"
JADX="${JADX:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}"
if [[ -z "$JADX" || ! -x "$JADX" ]]; then
  echo "set JADX to the executable pinned JADX dev launcher" >&2
  exit 3
fi
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
JADX_HEAD="$(git -C "$JADX_ROOT" rev-parse HEAD)"
if [[ "$JADX_HEAD" != "$EXPECTED_JADX_HEAD" ]]; then
  echo "pinned JADX checkout HEAD mismatch: expected $EXPECTED_JADX_HEAD, got $JADX_HEAD" >&2
  exit 3
fi
JADX_VERSION="$("$JADX" --version 2>&1 | head -n 1)"
if [[ "$JADX_VERSION" != "dev" ]]; then
  echo "JADX version mismatch: expected dev, got $JADX_VERSION" >&2
  exit 3
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/cf16-nested-finally.XXXXXX")"
trap 'find "$TMP" -depth -delete' EXIT
mkdir -p "$TMP/original" "$TMP/jadx-classes" "$TMP/jarde-classes" "$TMP/jadx-out" "$TMP/jadx-fixed-default" "$TMP/jadx-fixed-no-finally"

JARDE_CLI_SHA="$(shasum -a 256 "$CLI" | awk '{print $1}')"
JADX_SHA="$(shasum -a 256 "$JADX" | awk '{print $1}')"
printf 'jarde_checkout_head=%s\njarde_cli=%s\njarde_cli_version=%s\njarde_cli_sha256=%s\njavac=%s\njava=%s\njadx_checkout_head=%s\njadx=%s\njadx_version=%s\njadx_binary_sha256=%s\n' \
  "$ACTUAL_HEAD" "$CLI" "$CLI_VERSION" "$JARDE_CLI_SHA" "$JAVAC" "$JAVA" "$JADX_HEAD" "$JADX" "$JADX_VERSION" "$JADX_SHA" > "$OUT/toolchain.txt"
printf 'fixed_test_class_sha256=%s\n' "$FIXED_TEST_CLASS_SHA" >> "$OUT/toolchain.txt"

# Original complete top-level fixture + nine-path reflection runner.
"$JAVAC" --release 8 -g -d "$TMP/original" "$HERE/FinallyMinimalProbe.java" "$HERE/MinimalRunner.java" > "$OUT/original-javac.stdout" 2> "$OUT/original-javac.stderr"
ORIGINAL_JAVAC_STATUS=$?
if [[ $ORIGINAL_JAVAC_STATUS -ne 0 ]]; then
  echo "original_javac_exit=$ORIGINAL_JAVAC_STATUS" > "$OUT/original-result.txt"
  exit 4
fi
ORIGINAL_CLASS="$TMP/original/jadx/tests/integration/trycatch/FinallyMinimalProbe.class"
ACTUAL_CLASS_SHA="$(shasum -a 256 "$ORIGINAL_CLASS" | awk '{print $1}')"
printf 'recompiled_minimal_class_sha256=%s\n' "$ACTUAL_CLASS_SHA" >> "$OUT/toolchain.txt"
"$JAVAP" -classpath "$TMP/original" -c -v jadx.tests.integration.trycatch.FinallyMinimalProbe > "$OUT/minimal-bytecode.txt" 2>&1
python3 - "$OUT/minimal-bytecode.txt" "$HERE/three-method-bytecode.txt" > "$OUT/bytecode-shape.txt" <<'PY'
import re, sys
from pathlib import Path

def methods(path):
    lines = Path(path).read_text().splitlines()
    result = {}
    for name in ("test1", "test2", "test3"):
        marker = re.compile(r"^\s+public void " + name + r"\(int\);")
        start = next(i for i, line in enumerate(lines) if marker.match(line))
        end = next((i for i in range(start + 1, len(lines)) if re.match(r"\s+public ", lines[i])), len(lines))
        code, table, in_table = [], [], False
        for line in lines[start:end]:
            m = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line)
            if m:
                code.append((int(m.group(1)), m.group(2)))
            if "Exception table:" in line:
                in_table = True
                continue
            if in_table:
                m = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$", line)
                if m:
                    table.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), m.group(4)))
                elif table and line.strip():
                    in_table = False
        result[name] = (code, table)
    return result

a, b = methods(sys.argv[1]), methods(sys.argv[2])
for name in ("test1", "test2", "test3"):
    same = a[name] == b[name]
    print(f"{name}: bci_opcode_and_exception_table_equal={str(same).lower()}")
    if not same:
        print(f"  recompiled={a[name]}")
        print(f"  fixed_test={b[name]}")
        raise SystemExit(1)
PY
SHAPE_STATUS=$?
if [[ $SHAPE_STATUS -ne 0 ]]; then
  echo "recompiled minimal fixture differs from the fixed JADX test method shapes" >&2
  exit 4
fi
"$JAVA" -Xverify:all -cp "$TMP/original" jadx.tests.integration.trycatch.MinimalRunner > "$OUT/original-run.txt" 2>&1
ORIGINAL_RUN_STATUS=$?
if [[ $ORIGINAL_RUN_STATUS -ne 0 ]]; then
  echo "original nine-path runner failed; see $OUT/original-run.txt" >&2
  exit 4
fi

# JADX decompile, then compile and verify-run the complete recovered class.
"$JADX" --no-res -d "$TMP/jadx-out" "$ORIGINAL_CLASS" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
JADX_STATUS=$?
JADX_SOURCE="$TMP/jadx-out/sources/jadx/tests/integration/trycatch/FinallyMinimalProbe.java"
JADX_JAVAC_STATUS=not_run
JADX_RUN_STATUS=not_run
if [[ $JADX_STATUS -eq 0 && -f "$JADX_SOURCE" ]]; then
  cp "$JADX_SOURCE" "$OUT/FinallyMinimalProbe.jadx.java"
  "$JAVAC" --release 8 -g -d "$TMP/jadx-classes" "$JADX_SOURCE" "$HERE/MinimalRunner.java" > "$OUT/jadx-javac.stdout" 2> "$OUT/jadx-javac.stderr"
  JADX_JAVAC_STATUS=$?
  if [[ $JADX_JAVAC_STATUS -eq 0 ]]; then
    "$JAVA" -Xverify:all -cp "$TMP/jadx-classes" jadx.tests.integration.trycatch.MinimalRunner > "$OUT/jadx-run.txt" 2>&1
    JADX_RUN_STATUS=$?
  fi
fi
if [[ $JADX_STATUS -ne 0 || "$JADX_JAVAC_STATUS" != 0 || "$JADX_RUN_STATUS" != 0 ]]; then
  echo "JADX nine-path control failed (decompile=$JADX_STATUS compile=$JADX_JAVAC_STATUS run=$JADX_RUN_STATUS)" >&2
  exit 5
fi
if ! cmp -s "$OUT/original-run.txt" "$OUT/jadx-run.txt"; then
  echo "original and JADX nine-path stdout differ" >&2
  exit 5
fi

# Pinned JADX test control: default extracts three finally clauses; --no-finally
# leaves the seven javac cleanup copies as ordinary statements.
"$JADX" --no-res -d "$TMP/jadx-fixed-default" "$HERE/TestTryCatchFinally12\$TestCls.class" > "$OUT/jadx-fixed-default.stdout" 2> "$OUT/jadx-fixed-default.stderr"
JADX_FIXED_DEFAULT_STATUS=$?
"$JADX" --no-res --no-finally -d "$TMP/jadx-fixed-no-finally" "$HERE/TestTryCatchFinally12\$TestCls.class" > "$OUT/jadx-fixed-no-finally.stdout" 2> "$OUT/jadx-fixed-no-finally.stderr"
JADX_FIXED_NO_FINALLY_STATUS=$?
DEFAULT_SOURCE="$TMP/jadx-fixed-default/sources/jadx/tests/integration/trycatch/TestTryCatchFinally12\$TestCls.java"
NO_FINALLY_SOURCE="$TMP/jadx-fixed-no-finally/sources/jadx/tests/integration/trycatch/TestTryCatchFinally12\$TestCls.java"
if [[ $JADX_FIXED_DEFAULT_STATUS -ne 0 || $JADX_FIXED_NO_FINALLY_STATUS -ne 0 || ! -f "$DEFAULT_SOURCE" || ! -f "$NO_FINALLY_SOURCE" ]]; then
  echo "pinned JADX TestTryCatchFinally12 control decompilation failed" >&2
  exit 6
fi
cp "$DEFAULT_SOURCE" "$OUT/TestTryCatchFinally12\$TestCls.jadx-default.java"
cp "$NO_FINALLY_SOURCE" "$OUT/TestTryCatchFinally12\$TestCls.jadx-no-finally.java"
DEFAULT_CLEANUP_COUNT="$(grep -o 'sb\.append("-finally");' "$DEFAULT_SOURCE" | wc -l | tr -d ' ')"
NO_FINALLY_CLEANUP_COUNT="$(grep -o 'sb\.append("-finally");' "$NO_FINALLY_SOURCE" | wc -l | tr -d ' ')"
if [[ "$DEFAULT_CLEANUP_COUNT" != 3 || "$NO_FINALLY_CLEANUP_COUNT" != 7 ]]; then
  echo "pinned JADX cleanup count mismatch: default=$DEFAULT_CLEANUP_COUNT no-finally=$NO_FINALLY_CLEANUP_COUNT" >&2
  exit 6
fi

# Current Jarde CLI full-class presentation; compilation and runtime are observations.
"$CLI" class-source --input "$ORIGINAL_CLASS" --class jadx.tests.integration.trycatch.FinallyMinimalProbe \
  --policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
JARDE_STATUS=$?
JARDE_JAVAC_STATUS=not_run
JARDE_RUN_STATUS=not_run
if [[ $JARDE_STATUS -eq 0 ]]; then
  cp "$OUT/jarde.java.txt" "$TMP/FinallyMinimalProbe.java"
  "$JAVAC" --release 8 -g -d "$TMP/jarde-classes" "$TMP/FinallyMinimalProbe.java" "$HERE/MinimalRunner.java" > "$OUT/jarde-javac.stdout" 2> "$OUT/jarde-javac.stderr"
  JARDE_JAVAC_STATUS=$?
  if [[ $JARDE_JAVAC_STATUS -eq 0 ]]; then
    "$JAVA" -Xverify:all -cp "$TMP/jarde-classes" jadx.tests.integration.trycatch.MinimalRunner > "$OUT/jarde-run.txt" 2>&1
    JARDE_RUN_STATUS=$?
  fi
fi

JARDE_EQUALS_ORIGINAL=false
if [[ "$JARDE_RUN_STATUS" == 0 ]] && cmp -s "$OUT/original-run.txt" "$OUT/jarde-run.txt"; then
  JARDE_EQUALS_ORIGINAL=true
fi
cat > "$OUT/results.txt" <<EOF
original_javac_exit=$ORIGINAL_JAVAC_STATUS
original_java_Xverify_all_exit=$ORIGINAL_RUN_STATUS
jadx_decompile_exit=$JADX_STATUS
jadx_javac_exit=$JADX_JAVAC_STATUS
jadx_java_Xverify_all_exit=$JADX_RUN_STATUS
jarde_class_source_exit=$JARDE_STATUS
jarde_javac_exit=$JARDE_JAVAC_STATUS
jarde_java_Xverify_all_exit=$JARDE_RUN_STATUS
original_equals_jadx=true
jarde_equals_original=$JARDE_EQUALS_ORIGINAL
jadx_default_finally_clause_count=$DEFAULT_CLEANUP_COUNT
jadx_no_finally_cleanup_copy_count=$NO_FINALLY_CLEANUP_COUNT
EOF
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
