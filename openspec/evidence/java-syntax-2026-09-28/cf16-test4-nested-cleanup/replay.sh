#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 2 ]]; then echo 'usage: replay.sh JARDE_CLI OUTPUT_DIR' >&2; exit 2; fi
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$2"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
PACKAGE=jadx/tests/integration/trycatch
TARGET='TestTryCatchFinally4$TestCls'
FIXED="$HERE/classes/$PACKAGE/$TARGET.class"
EXPECTED_CLASS=2bf1b8932e521aade29fa1d562269903f9b8e9942f86981eb458d4f552d67060
EXPECTED_SOURCE=57b52e955fdf2db03a6949b02acc3ce9a096f939ed33c44a05c5fc36b8e0e846
EXPECTED_JADX=2fb1b16386941660fda07e9017285aec40fcb37f
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
[[ -x "$CLI" && -x "$JADX" ]]
[[ "$(shasum -a 256 "$FIXED" | awk '{print $1}')" == "$EXPECTED_CLASS" ]]
[[ "$(shasum -a 256 "$HERE/original/TestTryCatchFinally4.java" | awk '{print $1}')" == "$EXPECTED_SOURCE" ]]
[[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" == "$EXPECTED_JADX" ]]
mkdir -p "$OUT"
mkdir -p "$OUT/pinned"
TMP="$(mktemp -d /tmp/cf16-test4.XXXXXX)"
trap 'rm -rf "$TMP"' EXIT
for v in original pinned jarde; do mkdir -p "$TMP/$v"; done
JAVAC="${JAVAC:-$(command -v javac)}"
JAVA="${JAVA:-$(command -v java)}"
JAVAP="${JAVAP:-$(command -v javap)}"
printf 'jarde_cli_sha256=%s\njadx_head=%s\njadx_sha256=%s\nsource_sha256=%s\nfixed_class_sha256=%s\nfixed_class_major=55\n' \
 "$(shasum -a 256 "$CLI" | awk '{print $1}')" "$EXPECTED_JADX" "$(shasum -a 256 "$JADX" | awk '{print $1}')" "$EXPECTED_SOURCE" "$EXPECTED_CLASS" > "$OUT/toolchain.txt"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/original" "$HERE/original/TestTryCatchFinally4\$TestCls.java" "$HERE/TargetRunner.java" "$HERE/Control.java" "$HERE/ControlRunner.java" 2> "$OUT/original-javac.stderr"
"$JAVAP" -classpath "$TMP/original" -p -c -v "$PACKAGE.$TARGET" > "$OUT/original-recompiled.javap.txt"
"$JAVAP" -classpath "$TMP/original" -p -c -v "$PACKAGE.Control" > "$OUT/control.javap.txt"
printf 'javac=%s\njava=%s\nrecompiled_target_class_sha256=%s\n' \
 "$($JAVAC -version 2>&1)" "$($JAVA -version 2>&1 | head -n 1)" \
 "$(shasum -a 256 "$TMP/original/$PACKAGE/$TARGET.class" | awk '{print $1}')" >> "$OUT/toolchain.txt"
python3 "$HERE/check-shape.py" "$OUT/original-recompiled.javap.txt" "$HERE/original.javap.txt" > "$OUT/bytecode-shape.txt"

# Target tri-compare: normal-path verification only; no error injection into the pinned method.
"$JAVA" -Xverify:all -cp "$TMP/original" "$PACKAGE.TargetRunner" > "$OUT/target.original.run.txt"
"$JADX" --no-res -d "$TMP/jadx-target" "$FIXED" > "$OUT/jadx-target.log" 2>&1
JS="$TMP/jadx-target/sources/$PACKAGE/$TARGET.java"
cp "$JS" "$OUT/pinned/TestTryCatchFinally4-TestCls.jadx.java"
mkdir -p "$TMP/pinned/$PACKAGE"
cp "$JS" "$TMP/pinned/$PACKAGE/$TARGET.java"
cp "$HERE/TargetRunner.java" "$TMP/pinned/$PACKAGE/"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/pinned" "$TMP/pinned/$PACKAGE/$TARGET.java" "$TMP/pinned/$PACKAGE/TargetRunner.java" 2> "$OUT/pinned-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/pinned" "$PACKAGE.TargetRunner" > "$OUT/target.pinned.run.txt"
cmp "$OUT/target.original.run.txt" "$OUT/target.pinned.run.txt"
"$CLI" class-source --input "$FIXED" --class 'jadx.tests.integration.trycatch.TestTryCatchFinally4$TestCls' --policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
rg -q '        } finally \{' "$OUT/jarde.java.txt"
rg -q '            } catch \(java.io.IOException ' "$OUT/jarde.java.txt"
rg -q 'methods.1.outcome.report.quality = "structured"' "$OUT/jarde.report.txt"
if rg -q 'methods.1.outcome.report.fallbacks = \[[^]]+\]' "$OUT/jarde.report.txt"; then
  echo 'fixed target unexpectedly fell back' >&2; exit 1
fi
python3 - "$OUT/jarde.report.txt" <<'PY'
import re, sys
report = open(sys.argv[1]).read()
expected = {0, 2, 4, 7, 8, 11, 12, 13, 16, 17, 18, 19, 22, 23, 26, 27,
            30, 31, 34, 35, 38, 40, 41, 44, 45, 48, 49, 52, 54, 56, 57}
actual = {int(bci) for bci in re.findall(
    r'methods\.1\.outcome\.report\.source_map\.segments\.\d+\.origin\.(?:primary|derived\.\d+)\.bci = (\d+)',
    report)}
if actual != expected:
    raise SystemExit(f'fixed target source map mismatch: missing={sorted(expected-actual)} extra={sorted(actual-expected)}')
PY
mkdir -p "$TMP/jarde/$PACKAGE"
cp "$OUT/jarde.java.txt" "$TMP/jarde/$PACKAGE/$TARGET.java"
cp "$HERE/TargetRunner.java" "$TMP/jarde/$PACKAGE/"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/jarde" "$TMP/jarde/$PACKAGE/$TARGET.java" "$TMP/jarde/$PACKAGE/TargetRunner.java" 2> "$OUT/jarde-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/jarde" "$PACKAGE.TargetRunner" > "$OUT/target.jarde.run.txt"
cmp "$OUT/target.original.run.txt" "$OUT/target.jarde.run.txt"

# This control uses injectable seams and is separate from the exact target bytecode evidence.
"$JADX" --no-res -d "$TMP/jadx-control" "$TMP/original/$PACKAGE/Control.class" > "$OUT/jadx-control.log" 2>&1
JC="$TMP/jadx-control/sources/$PACKAGE/Control.java"
cp "$JC" "$OUT/pinned/Control.jadx.java"
for v in pinned jarde; do mkdir -p "$TMP/$v/$PACKAGE"; cp "$HERE/ControlRunner.java" "$TMP/$v/$PACKAGE/"; done
cp "$JC" "$TMP/pinned/$PACKAGE/Control.java"
"$JAVAC" --release 8 -g -Xlint:-options -cp "$TMP/original" -d "$TMP/pinned" "$TMP/pinned/$PACKAGE/Control.java" "$TMP/pinned/$PACKAGE/ControlRunner.java" 2> "$OUT/control-pinned-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/pinned:$TMP/original" "$PACKAGE.ControlRunner" > "$OUT/control.pinned.run.txt"
"$CLI" class-source --input "$TMP/original/$PACKAGE/Control.class" --class 'jadx.tests.integration.trycatch.Control' --policy single-class --release 8 --format text > "$OUT/Control.jarde.java" 2> "$OUT/control-jarde.report.txt"
cmp "$OUT/Control.jarde.java" "$HERE/Control.jarde.java"
rg -q 'methods.1.outcome.report.quality = "fallback"' "$OUT/control-jarde.report.txt"
cp "$OUT/Control.jarde.java" "$TMP/jarde/$PACKAGE/Control.java"
"$JAVAC" --release 8 -g -Xlint:-options -cp "$TMP/original" -d "$TMP/jarde" "$TMP/jarde/$PACKAGE/Control.java" "$TMP/jarde/$PACKAGE/ControlRunner.java" 2> "$OUT/control-jarde-javac.stderr"
"$JAVA" -Xverify:all -cp "$TMP/jarde:$TMP/original" "$PACKAGE.ControlRunner" > "$OUT/control.jarde.run.txt"
"$JAVA" -Xverify:all -cp "$TMP/original" "$PACKAGE.ControlRunner" > "$OUT/control.original.run.txt"
cmp "$OUT/control.original.run.txt" "$OUT/control.pinned.run.txt"
echo "target_test_method_bci_opcode_rows_match=true" > "$OUT/results.txt"
echo "target_normal_original_equals_pinned_jadx=$(cmp -s "$OUT/target.original.run.txt" "$OUT/target.pinned.run.txt" && echo true || echo false)" >> "$OUT/results.txt"
echo 'target_normal_original_equals_jarde=true' >> "$OUT/results.txt"
echo 'target_jarde_source_status=structured_nested_cleanup' >> "$OUT/results.txt"
echo 'target_jarde_source_map_all_31_bci=true' >> "$OUT/results.txt"
echo 'target_control=separate_three_row_helper_seam_probe' >> "$OUT/results.txt"
echo 'control_original_equals_pinned_jadx=true' >> "$OUT/results.txt"
echo 'control_jarde_source_status=safe_refusal' >> "$OUT/results.txt"
cat "$OUT/results.txt"
