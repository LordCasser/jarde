#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
CLI="$1"
OUT="$2"
PACKAGE=jadx/tests/integration/trycatch
TARGET='TestTryCatchFinally17$TestCls'
[[ -x "$CLI" && -x "$JADX" ]]
[[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" == 2fb1b16386941660fda07e9017285aec40fcb37f ]]
mkdir -p "$OUT/original-classes" "$OUT/probe-classes" "$OUT/jadx-classes" "$OUT/jarde-src"

javac --release 8 -g -Xlint:-options -d "$OUT/original-classes" "$HERE/original/TestTryCatchFinally17.java"
javac --release 8 -g -Xlint:-options -d "$OUT/probe-classes" "$HERE/probe/"*.java
[[ "$(shasum -a 256 "$OUT/original-classes/$PACKAGE/$TARGET.class" | awk '{print $1}')" == a6f958b7e117786d684150527757162b708b100ecbe43e630252c95ae29ffc3a ]]
[[ "$(shasum -a 256 "$OUT/probe-classes/$PACKAGE/$TARGET.class" | awk '{print $1}')" == fd132342a90bdca0fc50510a3e758e61215a44d5874e44f90828c1a5c6094cdd ]]

python3 - "$OUT/original-classes" "$OUT/probe-classes" <<'PY'
import re
import subprocess
import sys

name = 'jadx.tests.integration.trycatch.TestTryCatchFinally17$TestCls'
def method(classes):
    text = subprocess.check_output(['javap', '-classpath', classes, '-p', '-c', '-v', name], text=True)
    return re.sub(r'#\d+', '#', text.split('  public int test();', 1)[1].split('      LineNumberTable:', 1)[0])
original, probe = map(method, sys.argv[1:])
assert original == probe, 'the observable probe changed test() bytecode or exception rows'
assert '0     3     9   Class java/lang/UnsupportedOperationException' in probe
assert '0     3    16   Class java/lang/NullPointerException' in probe
assert '0     3    24   any' in probe
assert '16    19    24   any' in probe
assert probe.count('doFinally:()V') == 4
PY

java -Xverify:all -cp "$OUT/probe-classes" jadx.tests.integration.trycatch.Runner > "$OUT/original.run.txt"
cmp "$OUT/original.run.txt" "$HERE/expected/eight-paths.txt"
"$JADX" -d "$OUT/jadx" "$OUT/probe-classes" > "$OUT/jadx.log" 2>&1
JADX_SOURCE="$OUT/jadx/sources/$PACKAGE/TestTryCatchFinally17.java"
[[ "$(rg -oF 'finally {' "$JADX_SOURCE" | wc -l | tr -d '[:space:]')" == 1 ]]
rg -q 'catch \(NullPointerException ' "$JADX_SOURCE"
rg -q 'catch \(UnsupportedOperationException ' "$JADX_SOURCE"
javac --release 8 -g -Xlint:-options -d "$OUT/jadx-classes" "$OUT/jadx/sources/$PACKAGE/"*.java
java -Xverify:all -cp "$OUT/jadx-classes" jadx.tests.integration.trycatch.Runner > "$OUT/jadx.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jadx.run.txt"

for binary in TestTryCatchFinally17 "$TARGET" 'TestTryCatchFinally17$TestCls$TCls'; do
  "$CLI" class-source --input "$OUT/probe-classes/$PACKAGE/$binary.class" \
    --class "$PACKAGE/$binary" --policy single-class --release 8 --format text \
    > "$OUT/jarde-src/$binary.java" 2> "$OUT/$binary.report.txt"
done
if rg -q '@bytecode' "$OUT/jarde-src/$TARGET.java"; then
  echo 'target_status=refused' > "$OUT/jarde-status.txt"
else
  echo 'target_status=recovered' > "$OUT/jarde-status.txt"
fi
! rg -q '@bytecode' "$OUT/jarde-src/TestTryCatchFinally17.java" "$OUT/jarde-src/TestTryCatchFinally17\$TestCls\$TCls.java"
printf 'pinned_jadx_head=%s\ncli_sha256=%s\n' \
  "$(git -C "$JADX_ROOT" rev-parse HEAD)" "$(shasum -a 256 "$CLI" | awk '{print $1}')" \
  > "$OUT/toolchain.txt"
echo "original and pinned JADX eight-path Java 8 replay passed; $(cat "$OUT/jarde-status.txt"): $OUT"
