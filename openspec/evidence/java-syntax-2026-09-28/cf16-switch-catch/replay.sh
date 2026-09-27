#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
	 echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
	 exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(cd "$HERE/../../../.." && pwd)"
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
OUT="$2"
JAVAC="${JAVAC:-$(command -v javac)}"
JAVA="${JAVA:-$(command -v java)}"
JAVAP="${JAVAP:-$(command -v javap)}"
JADX="${JADX:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_FIXED_CLASS_SHA=d9b9cb676203d943ee3cf97d66e62dda2a637125d7f737be1e22ca275c209185
mkdir -p "$OUT"

if [[ ! -x "$CLI" ]]; then echo "Jarde CLI is not executable: $CLI" >&2; exit 3; fi
if [[ ! -x "$JADX" ]]; then echo "set JADX to the pinned JADX executable" >&2; exit 3; fi
if [[ "$(shasum -a 256 "$HERE/TestTryCatchFinally12\$TestCls.class" | awk '{print $1}')" != "$EXPECTED_FIXED_CLASS_SHA" ]]; then
	echo "fixed TestTryCatchFinally12\$TestCls class SHA mismatch" >&2
	exit 3
fi
if [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_JADX_HEAD" ]]; then
	echo "pinned JADX checkout HEAD mismatch" >&2
	exit 3
fi

TMP="$(mktemp -d "${TMPDIR:-/tmp}/cf16-switch-catch.XXXXXX")"
trap 'find "$TMP" -depth -delete' EXIT
mkdir -p "$TMP/original" "$TMP/jadx" "$TMP/jarde" "$TMP/jadx-out"
mkdir -p "$TMP/pinned-original/jadx/tests/integration/trycatch" "$TMP/pinned-jadx" "$TMP/pinned-jarde"
python3 "$HERE/make-variants.py"

printf 'jarde_checkout_head=%s\njarde_cli=%s\njarde_cli_version=%s\njarde_cli_sha256=%s\n' \
	"$(git -C "$ROOT" rev-parse HEAD)" "$CLI" "$($CLI --version | tail -n 1)" "$(shasum -a 256 "$CLI" | awk '{print $1}')" > "$OUT/toolchain.txt"
printf 'javac=%s\njava=%s\njavap=%s\njadx_checkout_head=%s\njadx=%s\njadx_version=%s\njadx_binary_sha256=%s\n' \
	"$JAVAC" "$JAVA" "$JAVAP" "$(git -C "$JADX_ROOT" rev-parse HEAD)" "$JADX" "$($JADX --version | head -n 1)" "$(shasum -a 256 "$JADX" | awk '{print $1}')" >> "$OUT/toolchain.txt"

"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/original" \
	"$HERE/SwitchCatchMinimal.java" "$HERE/SwitchCatchRunner.java" "$HERE/JadxAssertions.java" \
	"$HERE/PartialSwitchCatch1.java" "$HERE/PartialSwitchCatch2.java" "$HERE/PartialSwitchCatch3.java" \
	"$HERE/TwrSwitchCatch.java" "$HERE/VerifyVariants.java" > "$OUT/original-javac.stdout" 2> "$OUT/original-javac.stderr"
cp "$TMP/original/jadx/tests/integration/trycatch/SwitchCatchMinimal.class" "$OUT/SwitchCatchMinimal.class"
cp "$HERE/TestTryCatchFinally12\$TestCls.class" "$OUT/TestTryCatchFinally12\$TestCls.class"
shasum -a 256 "$OUT/SwitchCatchMinimal.class" "$OUT/TestTryCatchFinally12\$TestCls.class" > "$OUT/class-sha256.txt"
"$JAVAP" -classpath "$TMP/original" -c -v jadx.tests.integration.trycatch.SwitchCatchMinimal > "$OUT/original.javap.txt"
"$JAVAP" -c -v "$OUT/TestTryCatchFinally12\$TestCls.class" > "$OUT/fixed.javap.txt"
python3 - "$OUT/original.javap.txt" "$OUT/fixed.javap.txt" > "$OUT/method-shapes.txt" <<'PY'
import re, sys
from pathlib import Path

def shape(path, method):
    lines = Path(path).read_text().splitlines()
    if method == "runTest": marker = r"\s+public java\.lang\.String runTest\(int, int\);"
    else: marker = rf"\s+public void {method}\(int\);"
    start = next(i for i, line in enumerate(lines) if re.match(marker, line))
    end = next((i for i in range(start + 1, len(lines)) if re.match(r"\s+public ", lines[i])), len(lines))
    code, table, in_table = [], [], False
    for line in lines[start:end]:
        m = re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)", line)
        if m: code.append((int(m.group(1)), m.group(2)))
        if "Exception table:" in line: in_table = True; continue
        if in_table:
            m = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$", line)
            if m: table.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), m.group(4)))
            elif table and line.strip(): in_table = False
    return code, table

all_equal = True
for name in ("test1", "test2", "test3", "runTest"):
    original, fixed = shape(sys.argv[1], name), shape(sys.argv[2], name)
    equal = original == fixed
    print(f"{name}: bci_opcode_exception_table_equal={str(equal).lower()}")
    if not equal:
        print(f"  minimal={original}")
        print(f"  fixed={fixed}")
    all_equal &= equal

original = shape(sys.argv[1], "runTest")
expected = ([(0,'aload_0'),(1,'new'),(4,'dup'),(5,'invokespecial'),(8,'putfield'),(11,'iload_1'),(12,'tableswitch'),(40,'aload_0'),(41,'iload_2'),(42,'invokevirtual'),(45,'goto'),(48,'aload_0'),(49,'iload_2'),(50,'invokevirtual'),(53,'goto'),(56,'aload_0'),(57,'iload_2'),(58,'invokevirtual'),(61,'goto'),(64,'astore_3'),(65,'iload_2'),(66,'invokestatic'),(69,'invokestatic'),(72,'iconst_2'),(73,'invokestatic'),(76,'invokevirtual'),(79,'aload_0'),(80,'getfield'),(83,'invokevirtual'),(86,'areturn')], [(11,61,64,'Class java/lang/IllegalArgumentException')])
print(f"fixed_runTest_bytecode_shape_equal={str(all_equal).lower()}")
print(f"runTest_exact_expected_shape={str(original == expected).lower()}")
print(f"exception_row={original[1]}")
print("required_bci: switch=12 join=61 transfer=goto_79 return=79")
if not all_equal or original != expected: raise SystemExit(1)
PY
cp "$OUT/method-shapes.txt" "$OUT/bytecode-shape.txt"

"$JAVA" -Xverify:all -cp "$TMP/original" jadx.tests.integration.trycatch.SwitchCatchRunner > "$OUT/original-run.txt" 2> "$OUT/original-run.stderr"

"$JADX" --no-res -d "$TMP/jadx-out" "$OUT/SwitchCatchMinimal.class" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
JADX_SOURCE="$TMP/jadx-out/sources/jadx/tests/integration/trycatch/SwitchCatchMinimal.java"
cp "$JADX_SOURCE" "$OUT/SwitchCatchMinimal.jadx.java"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/jadx" "$JADX_SOURCE" "$HERE/SwitchCatchRunner.java" "$HERE/JadxAssertions.java" > "$OUT/jadx-javac.stdout" 2> "$OUT/jadx-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/jadx" jadx.tests.integration.trycatch.SwitchCatchRunner > "$OUT/jadx-run.txt" 2> "$OUT/jadx-run.stderr"
cmp "$OUT/original-run.txt" "$OUT/jadx-run.txt"

# Pinned TestTryCatchFinally12.TestCls class + pinned JADX source, exercised by runTest(II).
cp "$HERE/TestTryCatchFinally12\$TestCls.class" "$TMP/pinned-original/jadx/tests/integration/trycatch/"
"$JAVAC" --release 8 -g -Xlint:-options -cp "$TMP/pinned-original" -d "$TMP/pinned-original" \
	"$HERE/PinnedFixedRunner.java" "$HERE/JadxAssertions.java" > "$OUT/pinned-original-javac.stdout" 2> "$OUT/pinned-original-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/pinned-original" jadx.tests.integration.trycatch.PinnedFixedRunner > "$OUT/pinned-original-run.txt"
cp "$HERE/TestTryCatchFinally12-TestCls.jadx-default.java" "$TMP/TestTryCatchFinally12\$TestCls.java"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/pinned-jadx" "$TMP/TestTryCatchFinally12\$TestCls.java" \
	"$HERE/PinnedFixedRunner.java" "$HERE/JadxAssertions.java" > "$OUT/pinned-jadx-javac.stdout" 2> "$OUT/pinned-jadx-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/pinned-jadx" jadx.tests.integration.trycatch.PinnedFixedRunner > "$OUT/pinned-jadx-run.txt"
cmp "$OUT/pinned-original-run.txt" "$OUT/pinned-jadx-run.txt"

"$CLI" class-source --input "$OUT/SwitchCatchMinimal.class" --class jadx.tests.integration.trycatch.SwitchCatchMinimal \
	--policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
"$CLI" class-source --input "$HERE/TestTryCatchFinally12\$TestCls.class" \
	--class 'jadx.tests.integration.trycatch.TestTryCatchFinally12$TestCls' --policy single-class --release 8 --format text \
	> "$OUT/pinned-jarde.java.txt" 2> "$OUT/pinned-jarde.report.txt"
shasum -a 256 "$OUT/SwitchCatchMinimal.class" "$OUT/TestTryCatchFinally12\$TestCls.class" \
	> "$OUT/input-sha256.txt"
cat > "$OUT/jarde-invocations.txt" <<EOF
$CLI class-source --input $OUT/SwitchCatchMinimal.class --class jadx.tests.integration.trycatch.SwitchCatchMinimal --policy single-class --release 8 --format text
$CLI class-source --input $HERE/TestTryCatchFinally12\$TestCls.class --class 'jadx.tests.integration.trycatch.TestTryCatchFinally12\$TestCls' --policy single-class --release 8 --format text
EOF
JARDE_JAVAC=not_run
JARDE_RUN=not_run
cp "$OUT/jarde.java.txt" "$TMP/SwitchCatchMinimal.java"
if "$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/jarde" "$TMP/SwitchCatchMinimal.java" "$HERE/SwitchCatchRunner.java" "$HERE/JadxAssertions.java" > "$OUT/jarde-javac.stdout" 2> "$OUT/jarde-javac.stderr"; then
	JARDE_JAVAC=0
	if "$JAVA" -Xverify:all -cp "$TMP/jarde" jadx.tests.integration.trycatch.SwitchCatchRunner > "$OUT/jarde-run.txt" 2> "$OUT/jarde-run.stderr"; then JARDE_RUN=0; else JARDE_RUN=$?; fi
else
	JARDE_JAVAC=$?
fi
cp "$OUT/pinned-jarde.java.txt" "$TMP/TestTryCatchFinally12\$TestCls.java"
PINNED_JARDE_JAVAC=not_run
PINNED_JARDE_RUN=not_run
if "$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/pinned-jarde" "$TMP/TestTryCatchFinally12\$TestCls.java" \
	"$HERE/PinnedFixedRunner.java" "$HERE/JadxAssertions.java" > "$OUT/pinned-jarde-javac.stdout" 2> "$OUT/pinned-jarde-javac.stderr"; then
	PINNED_JARDE_JAVAC=0
	if "$JAVA" -Xverify:all -cp "$TMP/pinned-jarde" jadx.tests.integration.trycatch.PinnedFixedRunner > "$OUT/pinned-jarde-run.txt" 2> "$OUT/pinned-jarde-run.stderr"; then PINNED_JARDE_RUN=0; else PINNED_JARDE_RUN=$?; fi
else
	PINNED_JARDE_JAVAC=$?
fi

"$JAVA" -Xverify:all -cp "$TMP/original" jadx.tests.integration.trycatch.VerifyVariants > "$OUT/variants-verify-run.txt"
for variant in PartialSwitchCatch1 PartialSwitchCatch2 PartialSwitchCatch3 TwrSwitchCatch; do
	"$JAVAP" -classpath "$TMP/original" -c -v "jadx.tests.integration.trycatch.$variant" > "$OUT/$variant.javap.txt"
done
python3 - "$OUT" > "$OUT/variant-shapes.txt" <<'PY'
import re, sys
from pathlib import Path

root = Path(sys.argv[1])
for name in ("PartialSwitchCatch1", "PartialSwitchCatch2", "PartialSwitchCatch3", "TwrSwitchCatch"):
    lines = (root / f"{name}.javap.txt").read_text().splitlines()
    start = next(i for i, line in enumerate(lines) if re.match(r"\s+public java\.lang\.String runTest\(int, int\);", line))
    end = next((i for i in range(start + 1, len(lines)) if re.match(r"\s+public ", lines[i])), len(lines))
    rows, in_table = [], False
    for line in lines[start:end]:
        if "Exception table:" in line: in_table = True; continue
        if in_table:
            m = re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$", line)
            if m: rows.append((int(m.group(1)), int(m.group(2)), int(m.group(3)), m.group(4)))
            elif rows and line.strip(): in_table = False
    print(f"{name}: runTest_exception_rows={rows}")
    print(f"{name}: verifier_valid=true; verifier_runtime=runTest(1,0)")
PY

cat > "$OUT/results.txt" <<EOF
fixed_runTest_shape_exact=true
original_javac_exit=0
original_java_Xverify_all_exit=0
jadx_javac_exit=0
jadx_java_Xverify_all_exit=0
original_equals_jadx=true
fixed_test_original_Xverify_all_exit=0
pinned_jadx_javac_exit=0
pinned_jadx_Xverify_all_exit=0
fixed_test_original_equals_pinned_jadx=true
jarde_report_expected_single_method_refusal=$(grep -q 'methods.5.outcome.report.fallbacks = \["jre_region_ownership_overlap"\]' "$OUT/jarde.report.txt" && echo true || echo false)
pinned_jarde_runTest_report_code=$(sed -n '/methods.5.outcome.report.diagnostics.0.code = /{s/.*= "\([^"]*\)"/\1/p;q;}' "$OUT/pinned-jarde.report.txt")
pinned_jarde_test1_test2_test3_statements=$(grep -q 'methods.1.outcome.report.content = "contains_statements"' "$OUT/pinned-jarde.report.txt" && grep -q 'methods.2.outcome.report.content = "contains_statements"' "$OUT/pinned-jarde.report.txt" && grep -q 'methods.3.outcome.report.content = "contains_statements"' "$OUT/pinned-jarde.report.txt" && echo true || echo false)
jarde_javac_exit=$JARDE_JAVAC
jarde_java_Xverify_all_exit=$JARDE_RUN
pinned_jarde_javac_exit=$PINNED_JARDE_JAVAC
pinned_jarde_Xverify_all_exit=$PINNED_JARDE_RUN
four_near_negative_variants_compiled_java8=true
four_near_negative_variants_Xverify_all_and_safe_run=true
EOF
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
