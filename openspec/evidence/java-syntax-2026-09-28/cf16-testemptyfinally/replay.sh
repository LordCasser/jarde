#!/usr/bin/env bash
set -euo pipefail
if [[ $# -ne 2 ]]; then echo 'usage: replay.sh JARDE_CLI OUTPUT_DIR' >&2; exit 2; fi
HERE="$(cd "$(dirname "$0")" && pwd)"
OUT="$2"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
PACKAGE=jadx/tests/integration/trycatch
PACKAGE_DOTS=jadx.tests.integration.trycatch
TARGET='TestEmptyFinally$TestCls'
FIXED="$HERE/original/$TARGET.class"
EXPECTED_CLASS=dcf5de9a4037ddd2169f103e426be38fac60dba8fb2d5cb38dc6ffe3c648018a
EXPECTED_SOURCE=258da505cd3729bfcf4e54598314fff43d838f7ca3f8cc536c206f6ae790cc3f
EXPECTED_STANDALONE_SOURCE=a1d378eb5b6b064d889da629e381388283d459b1d0b6384d9ec405ac21b8737f
EXPECTED_JADX=2fb1b16386941660fda07e9017285aec40fcb37f
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
[[ -x "$CLI" && -x "$JADX" ]]
[[ "$(shasum -a 256 "$FIXED" | awk '{print $1}')" == "$EXPECTED_CLASS" ]]
[[ "$(shasum -a 256 "$HERE/original/TestEmptyFinally.java" | awk '{print $1}')" == "$EXPECTED_SOURCE" ]]
[[ "$(shasum -a 256 "$HERE/original/$TARGET.java" | awk '{print $1}')" == "$EXPECTED_STANDALONE_SOURCE" ]]
[[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" == "$EXPECTED_JADX" ]]
mkdir -p "$OUT"
TMP="$(mktemp -d /tmp/cf16-empty-finally.XXXXXX)"
trap 'rm -rf "$TMP"' EXIT
for v in fixed original pinned jarde; do mkdir -p "$TMP/$v/$PACKAGE"; done
JAVAC="${JAVAC:-$(command -v javac)}"
JAVA="${JAVA:-$(command -v java)}"
JAVAP="${JAVAP:-$(command -v javap)}"

printf 'jarde_cli_sha256=%s\njarde_source_revision=%s\njadx_head=%s\njadx_cli_sha256=%s\nsource_sha256=%s\nstandalone_source_sha256=%s\nfixed_class_sha256=%s\nfixed_class_major=52\n' \
 "$(shasum -a 256 "$CLI" | awk '{print $1}')" "$(git -C "$HERE" rev-parse HEAD)" \
 "$EXPECTED_JADX" "$(shasum -a 256 "$JADX" | awk '{print $1}')" "$EXPECTED_SOURCE" \
 "$EXPECTED_STANDALONE_SOURCE" "$EXPECTED_CLASS" > "$OUT/toolchain.txt"
printf 'javac=%s\njava=%s\n' "$($JAVAC -version 2>&1)" "$($JAVA -version 2>&1 | head -n 1)" >> "$OUT/toolchain.txt"

# The fixed inner class is the exact output built in the pinned JADX checkout.
cp "$FIXED" "$TMP/fixed/$PACKAGE/$TARGET.class"
cp "$HERE/BehaviorProbe.java" "$TMP/fixed/$PACKAGE/"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/fixed" "$TMP/fixed/$PACKAGE/BehaviorProbe.java" 2> /dev/null
"$JAVAP" -p -c -v "$FIXED" > "$OUT/original.fixed.javap.txt"
"$JAVA" -Xverify:all -cp "$TMP/fixed" "$PACKAGE_DOTS.BehaviorProbe" "$PACKAGE_DOTS.$TARGET" > "$OUT/behavior.fixed-original.txt"

# The source transcription is the exact target method without the JADX JUnit wrapper.
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/original" \
  "$HERE/original/$TARGET.java" "$HERE/BehaviorProbe.java" 2> /dev/null
"$JAVAP" -classpath "$TMP/original" -p -c -v "$PACKAGE_DOTS.$TARGET" > "$OUT/original.standalone.javap.txt"
cp "$TMP/original/$PACKAGE/$TARGET.class" "$OUT/original.standalone.class"
"$JAVA" -Xverify:all -cp "$TMP/original" "$PACKAGE_DOTS.BehaviorProbe" "$PACKAGE_DOTS.$TARGET" > "$OUT/behavior.standalone-original.txt"

# Decompile only the fixed target class. The output remains a standalone source file.
"$JADX" --no-res -d "$TMP/jadx" "$FIXED" > "$OUT/jadx.log" 2>&1
cp "$TMP/jadx/sources/$PACKAGE/$TARGET.java" "$OUT/pinned.jadx.java"
cp "$OUT/pinned.jadx.java" "$TMP/pinned/$PACKAGE/$TARGET.java"
cp "$HERE/BehaviorProbe.java" "$TMP/pinned/$PACKAGE/"
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/pinned" \
  "$TMP/pinned/$PACKAGE/$TARGET.java" "$TMP/pinned/$PACKAGE/BehaviorProbe.java" 2> /dev/null
"$JAVAP" -classpath "$TMP/pinned" -p -c -v "$PACKAGE_DOTS.$TARGET" > "$OUT/pinned.javap.txt"
cp "$TMP/pinned/$PACKAGE/$TARGET.class" "$OUT/pinned.class"
printf 'pinned_jadx_class_sha256=%s\n' "$(shasum -a 256 "$OUT/pinned.class" | awk '{print $1}')" >> "$OUT/toolchain.txt"
python3 "$HERE/check-bytecode-shape.py" "$OUT/original.fixed.javap.txt" \
  "$OUT/original.standalone.javap.txt" "$OUT/pinned.javap.txt" > "$OUT/bytecode-shape.txt"
"$JAVA" -Xverify:all -cp "$TMP/pinned" "$PACKAGE_DOTS.BehaviorProbe" "$PACKAGE_DOTS.$TARGET" > "$OUT/behavior.pinned-jadx.txt"

# Jarde's complete source must now compile and match the three fixed behavior paths.
"$CLI" class-source --input "$FIXED" --class "$PACKAGE_DOTS.$TARGET" --policy single-class --release 8 --format text > "$OUT/jarde.java.txt" 2> "$OUT/jarde.report.txt"
cp "$OUT/jarde.java.txt" "$TMP/jarde/$TARGET.java"
set +e
"$JAVAC" --release 8 -g -Xlint:-options -d "$TMP/jarde" \
  "$TMP/jarde/$TARGET.java" "$HERE/BehaviorProbe.java" > /dev/null 2> "$OUT/jarde-javac.stderr"
JARDE_JAVAC_STATUS=$?
set -e
python3 - "$OUT/jarde-javac.stderr" "$TMP/jarde" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
path.write_text(path.read_text().replace(sys.argv[2], "JARDE_OUTPUT"), encoding="utf-8")
PY
printf 'jarde_java8_compile_exit=%s\n' "$JARDE_JAVAC_STATUS" > "$OUT/results.txt"
if [[ $JARDE_JAVAC_STATUS -eq 0 ]]; then
  "$JAVAP" -classpath "$TMP/jarde" -p -c -v "$PACKAGE_DOTS.$TARGET" > "$OUT/jarde.javap.txt"
  "$JAVA" -Xverify:all -cp "$TMP/jarde" "$PACKAGE_DOTS.BehaviorProbe" "$PACKAGE_DOTS.$TARGET" > "$OUT/behavior.jarde.txt"
  echo 'jarde_behavior_trace=available' >> "$OUT/results.txt"
else
  echo 'jarde_behavior_trace=unavailable_source_did_not_compile' >> "$OUT/results.txt"
fi
[[ $JARDE_JAVAC_STATUS -eq 0 ]]
cmp "$OUT/behavior.fixed-original.txt" "$OUT/behavior.jarde.txt"
! grep -q '@bytecode\|finally {' "$OUT/jarde.java.txt"
grep -q 'methods.1.outcome.report.fallbacks = \[\]' "$OUT/jarde.report.txt"
echo "fixed_class_sha256=$(shasum -a 256 "$FIXED" | awk '{print $1}')" >> "$OUT/results.txt"
echo "standalone_class_sha256=$(shasum -a 256 "$OUT/original.standalone.class" | awk '{print $1}')" >> "$OUT/results.txt"
cmp "$OUT/behavior.fixed-original.txt" "$OUT/behavior.standalone-original.txt"
cmp "$OUT/behavior.fixed-original.txt" "$OUT/behavior.pinned-jadx.txt"
printf 'fixed_equals_standalone_behavior=true\nfixed_equals_pinned_jadx_behavior=true\nfixed_equals_jarde_behavior=true\n' >> "$OUT/results.txt"
mkdir -p "$OUT/neighbors"
for name in changed-throwable rows-swapped handler-normal-exit; do
  case "$name" in
    changed-throwable) expected=aa52a381c7fce255c3043dabce882c055a39e50ea616dd496b0f8e8fa590049b ;;
    rows-swapped) expected=b0a5daef736ba2ff388cbc3267353a0609095703d7b2b42cf50c6268baa3f989 ;;
    handler-normal-exit) expected=26130f374f5a3808240e293bae28cd81ba5de8ab6dd2e09e1582806f7316102e ;;
  esac
  neighbor="$HERE/neighbors/$name.class"
  [[ "$(shasum -a 256 "$neighbor" | awk '{print $1}')" == "$expected" ]]
  cp "$neighbor" "$TMP/fixed/$PACKAGE/$TARGET.class"
  "$JAVA" -Xverify:all -cp "$TMP/fixed" "$PACKAGE_DOTS.BehaviorProbe" "$PACKAGE_DOTS.$TARGET" > "$OUT/neighbors/$name.behavior.txt"
  "$CLI" class-source --input "$neighbor" --class "$PACKAGE_DOTS.$TARGET" --policy single-class --release 8 --format text > "$OUT/neighbors/$name.jarde.java.txt" 2> "$OUT/neighbors/$name.jarde.report.txt"
  grep -q '@bytecode' "$OUT/neighbors/$name.jarde.java.txt"
  ! grep -q '^[[:space:]]*f1.close();' "$OUT/neighbors/$name.jarde.java.txt"
  printf 'neighbor_%s_sha256=%s\nneighbor_%s_verifier_and_refusal=true\n' "$name" "$expected" "$name" >> "$OUT/results.txt"
done
cat "$OUT/results.txt"
python3 - "$OUT" <<'PY'
import hashlib
import pathlib
import sys

root = pathlib.Path(sys.argv[1])
paths = sorted(path for path in root.rglob("*") if path.is_file() and path.name != "SHA256SUMS")
with (root / "SHA256SUMS").open("w", encoding="utf-8") as sums:
    for path in paths:
        digest = hashlib.sha256(path.read_bytes()).hexdigest()
        sums.write(f"{digest}  {path.relative_to(root).as_posix()}\n")
PY
