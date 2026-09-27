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
TARGET='TestTryCatchFinally16$TestCls'
[[ -x "$CLI" && -x "$JADX" ]]
[[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" == 2fb1b16386941660fda07e9017285aec40fcb37f ]]
mkdir -p "$OUT/original-classes" "$OUT/probe-classes" "$OUT/jadx-classes" "$OUT/jarde-src"

javac --release 8 -g -Xlint:-options -d "$OUT/original-classes" "$HERE/original/TestTryCatchFinally16.java"
javac --release 8 -g -Xlint:-options -d "$OUT/probe-classes" "$HERE/probe/"*.java
[[ "$(shasum -a 256 "$OUT/original-classes/$PACKAGE/$TARGET.class" | awk '{print $1}')" == a32f63ffe5749eac6553192b4f4dd10cf109883fc993681891f99e0140f60402 ]]
[[ "$(shasum -a 256 "$OUT/probe-classes/$PACKAGE/$TARGET.class" | awk '{print $1}')" == 84567faf733ef3932df19f8e89478cc796d2d3511cff88b8a64301313b2f1cb2 ]]

python3 - "$OUT/original-classes" "$OUT/probe-classes" <<'PY'
import re
import subprocess
import sys

name = 'jadx.tests.integration.trycatch.TestTryCatchFinally16$TestCls'
def method(classes):
    text = subprocess.check_output(['javap', '-classpath', classes, '-p', '-c', '-v', name], text=True)
    return re.sub(r'#\d+', '#', text.split('  public void test();', 1)[1].split('      LineNumberTable:', 1)[0])
original, probe = map(method, sys.argv[1:])
assert original == probe, 'the observable probe changed test() bytecode or exception rows'
assert '0     3     9   Class java/lang/Exception' in probe
assert '0     3    16   any' in probe
assert probe.count('doFinally:()V') == 3
PY

java -Xverify:all -cp "$OUT/probe-classes" jadx.tests.integration.trycatch.Runner > "$OUT/original.run.txt"
cmp "$OUT/original.run.txt" "$HERE/expected/six-paths.txt"
"$JADX" -d "$OUT/jadx" "$OUT/probe-classes" > "$OUT/jadx.log" 2>&1
JADX_SOURCE="$OUT/jadx/sources/$PACKAGE/TestTryCatchFinally16.java"
[[ "$(rg -oF 'finally {' "$JADX_SOURCE" | wc -l | tr -d '[:space:]')" == 1 ]]
rg -q 'catch \(Exception e\)' "$JADX_SOURCE"
javac --release 8 -g -Xlint:-options -d "$OUT/jadx-classes" "$OUT/jadx/sources/$PACKAGE/"*.java
java -Xverify:all -cp "$OUT/jadx-classes" jadx.tests.integration.trycatch.Runner > "$OUT/jadx.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jadx.run.txt"

for binary in TestTryCatchFinally16 "$TARGET" 'TestTryCatchFinally16$TestCls$TCls'; do
  "$CLI" class-source --input "$OUT/probe-classes/$PACKAGE/$binary.class" \
    --class "$PACKAGE/$binary" --policy single-class --release 8 --format text \
    > "$OUT/jarde-src/$binary.java" 2> "$OUT/$binary.report.txt"
done
if rg -q '@bytecode' "$OUT/jarde-src/$TARGET.java"; then
  echo 'target_status=refused' > "$OUT/jarde-status.txt"
else
  echo 'target_status=recovered' > "$OUT/jarde-status.txt"
fi
! rg -q '@bytecode' "$OUT/jarde-src/TestTryCatchFinally16.java" "$OUT/jarde-src/TestTryCatchFinally16\$TestCls\$TCls.java"
printf 'pinned_jadx_head=%s\ncli_sha256=%s\n' \
  "$(git -C "$JADX_ROOT" rev-parse HEAD)" "$(shasum -a 256 "$CLI" | awk '{print $1}')" \
  > "$OUT/toolchain.txt"
echo "original and pinned JADX six-path Java 8 replay passed; $(cat "$OUT/jarde-status.txt"): $OUT"
