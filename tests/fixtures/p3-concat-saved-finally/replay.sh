#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel)"
CLI="$1"
OUT="$2"
JADX_ROOT="${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}"
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
FIXED="$ROOT/openspec/evidence/java-syntax-2026-09-27/cf16-finally"
WORK="$(mktemp -d /private/tmp/jarde-cf16-concat-replay.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT" "$WORK/original" "$WORK/acceptance" "$WORK/jadx" "$WORK/jadx-classes" "$WORK/fixed-jadx" "$WORK/jarde" "$WORK/patcher" "$WORK/exceptional" "$WORK/exceptional-jarde"

[[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" == 2fb1b16386941660fda07e9017285aec40fcb37f ]]
[[ -x "$JADX" && -x "$CLI" ]]
[[ "$(shasum -a 256 "$FIXED/original/FinallyOnce.class" | awk '{print $1}')" == 3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2 ]]

javac --release 8 -g -Xlint:-options -d "$WORK/original" "$FIXED/input/FinallyOnce.java"
cmp "$WORK/original/FinallyOnce.class" "$FIXED/original/FinallyOnce.class"
javac --release 8 -g -Xlint:-options -d "$WORK/acceptance" "$HERE/FinallyOnce.java" "$HERE/Runner.java"
cmp "$WORK/acceptance/FinallyOnce.class" "$HERE/v8/FinallyOnce.class"
javap -classpath "$WORK/original" -c -p FinallyOnce > "$OUT/original.javap.txt"
javap -classpath "$WORK/acceptance" -c -p FinallyOnce > "$OUT/acceptance.javap.txt"
python3 - "$OUT/original.javap.txt" "$OUT/acceptance.javap.txt" <<'PY'
import re, sys
from pathlib import Path
def shape(path):
    body = Path(path).read_text().split('public static java.lang.String handled(boolean);', 1)[1].split('public static', 1)[0]
    return (re.findall(r'^\s*(\d+):\s+([a-z][a-z0-9_]*)', body, re.M),
            re.findall(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|Class \S+)', body, re.M))
assert shape(sys.argv[1]) == shape(sys.argv[2]), 'handled layout changed'
print('handled_bci_opcode_rows_equal=true')
PY

java -Xverify:all -cp "$WORK/original" FinallyOnce > "$OUT/fixed-original.run.txt"
javac --release 8 -g -Xlint:-options -d "$WORK/fixed-jadx" "$FIXED/jadx/sources/defpackage/FinallyOnce.java"
java -Xverify:all -cp "$WORK/fixed-jadx" defpackage.FinallyOnce > "$OUT/fixed-jadx.run.txt"
[[ "$(head -n 1 "$OUT/fixed-original.run.txt")" == normal:1 ]]
[[ "$(head -n 1 "$OUT/fixed-jadx.run.txt")" == normal:2 ]]
"$CLI" class-source --input "$FIXED/original/FinallyOnce.class" --class FinallyOnce --policy single-class --release 8 --format text > "$OUT/fixed-jarde.java" 2> "$OUT/fixed-jarde.report.txt"
! sed -n '/handled(boolean/,/escaping()/p' "$OUT/fixed-jarde.java" | rg -q '@bytecode'
rg -q '@bytecode' "$OUT/fixed-jarde.java"

java -Xverify:all -cp "$WORK/acceptance" Runner > "$OUT/acceptance-original.run.txt"
"$JADX" --no-res -d "$WORK/jadx" "$WORK/acceptance/FinallyOnce.class" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
javac --release 8 -g -Xlint:-options -d "$WORK/jadx-classes" "$WORK/jadx/sources/defpackage/FinallyOnce.java" "$HERE/JadxRunner.java"
java -Xverify:all -cp "$WORK/jadx-classes" defpackage.JadxRunner > "$OUT/acceptance-jadx.run.txt"
cp "$WORK/jadx/sources/defpackage/FinallyOnce.java" "$OUT/acceptance-jadx.java"
"$CLI" class-source --input "$WORK/acceptance/FinallyOnce.class" --class FinallyOnce --policy single-class --release 8 --format text > "$WORK/jarde/FinallyOnce.java" 2> "$OUT/acceptance-jarde.report.txt"
cp "$WORK/jarde/FinallyOnce.java" "$OUT/acceptance-jarde.java"
! sed -n '/handled(boolean/,/count()/p' "$WORK/jarde/FinallyOnce.java" | rg -q '@bytecode'
[[ "$(sed -n '/handled(boolean/,/count()/p' "$WORK/jarde/FinallyOnce.java" | rg -c 'finally \{')" == 1 ]]
[[ "$(sed -n '/handled(boolean/,/count()/p' "$WORK/jarde/FinallyOnce.java" | rg -c 'return "caught:"')" == 1 ]]
javac --release 8 -g -Xlint:-options -d "$WORK/jarde" "$WORK/jarde/FinallyOnce.java" "$HERE/Runner.java"
java -Xverify:all -cp "$WORK/jarde" Runner > "$OUT/acceptance-jarde.run.txt"
cmp "$OUT/acceptance-original.run.txt" "$OUT/acceptance-jarde.run.txt"
[[ "$(cat "$OUT/acceptance-original.run.txt")" == $'normal:1\ncaught:arg:1' ]]
[[ "$(head -n 1 "$OUT/acceptance-jadx.run.txt")" == normal:2 ]]

javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED -d "$WORK/patcher" "$HERE/PatchNeighbor.java"
javac --release 8 -g -Xlint:-options -d "$WORK/exceptional" "$HERE/FinallyOnce.java" "$HERE/ThrowingArgument.java" "$HERE/ExceptionalRunner.java"
java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED -cp "$WORK/patcher" PatchNeighbor "$WORK/acceptance/FinallyOnce.class" "$WORK/exceptional/FinallyOnce.class" exceptional-message
cmp "$WORK/exceptional/FinallyOnce.class" "$HERE/exceptional/FinallyOnce.class"
java -Xverify:all -cp "$WORK/exceptional" ExceptionalRunner > "$OUT/exceptional-original.run.txt"
"$CLI" class-source --input "$WORK/exceptional/FinallyOnce.class" --class FinallyOnce --policy single-class --release 8 --format text > "$WORK/exceptional-jarde/FinallyOnce.java" 2> "$OUT/exceptional-jarde.report.txt"
javac --release 8 -g -Xlint:-options -d "$WORK/exceptional-jarde" "$WORK/exceptional-jarde/FinallyOnce.java" "$HERE/ThrowingArgument.java" "$HERE/ExceptionalRunner.java"
java -Xverify:all -cp "$WORK/exceptional-jarde" ExceptionalRunner > "$OUT/exceptional-jarde.run.txt"
cmp "$OUT/exceptional-original.run.txt" "$OUT/exceptional-jarde.run.txt"
[[ "$(cat "$OUT/exceptional-original.run.txt")" == true:java.lang.IllegalStateException:message:1 ]]

for kind in non-concat extra-consumer range-shrunk; do
  mkdir -p "$WORK/$kind"
  java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED -cp "$WORK/patcher" PatchNeighbor "$WORK/acceptance/FinallyOnce.class" "$WORK/$kind/FinallyOnce.class" "$kind"
  cmp "$WORK/$kind/FinallyOnce.class" "$HERE/negatives/$kind.class"
  cp "$WORK/acceptance/Runner.class" "$WORK/$kind/"
  java -Xverify:all -cp "$WORK/$kind" Runner > "$OUT/$kind.run.txt"
  "$CLI" class-source --input "$WORK/$kind/FinallyOnce.class" --class FinallyOnce --policy single-class --release 8 --format text > "$OUT/$kind.jarde.java" 2> "$OUT/$kind.jarde.report.txt"
  ! sed -n '/handled(boolean/,/count()/p' "$OUT/$kind.jarde.java" | rg -q 'finally \{'
done

shasum -a 256 "$FIXED/original/FinallyOnce.class" "$HERE/v8/FinallyOnce.class" "$HERE/exceptional/FinallyOnce.class" "$HERE/negatives/"*.class > "$OUT/class-sha256.txt"
printf 'jadx_head=%s\ncli_sha256=%s\n' "$(git -C "$JADX_ROOT" rev-parse HEAD)" "$(shasum -a 256 "$CLI" | awk '{print $1}')" > "$OUT/toolchain.txt"
echo "three-way acceptance, exceptional identity, and verifier-valid negative neighbors passed: $OUT"
