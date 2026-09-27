#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "用法: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel)"
CLI="$1"
OUT="$2"
JADX_ROOT="${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}"
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
FIXED="$ROOT/openspec/evidence/java-syntax-2026-09-27/cf16-finally"
WORK="$(mktemp -d /private/tmp/jarde-cf16-exception-replay.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT" "$WORK/original" "$WORK/handled-only" "$WORK/acceptance" "$WORK/jadx" "$WORK/jadx-classes" "$WORK/fixed-jadx-classes" "$WORK/jarde" "$WORK/patcher"

[[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" == 2fb1b16386941660fda07e9017285aec40fcb37f ]]
[[ -x "$JADX" && -x "$CLI" ]]
[[ "$(shasum -a 256 "$FIXED/original/FinallyOnce.class" | awk '{print $1}')" == 3ad6857285368c95c3176a520300c4abba09084443fe7fc08e8972330e9cedd2 ]]

javac --release 8 -g -Xlint:-options -d "$WORK/original" "$FIXED/input/FinallyOnce.java"
cmp "$WORK/original/FinallyOnce.class" "$FIXED/original/FinallyOnce.class"
javac --release 8 -g -Xlint:-options -d "$WORK/handled-only" "$HERE/handled-only/FinallyOnce.java"
javac --release 8 -g -Xlint:-options -d "$WORK/acceptance" "$HERE/FinallyOnce.java" "$HERE/Runner.java" "$HERE/NeighborRunner.java"
cmp "$WORK/acceptance/FinallyOnce.class" "$HERE/v8/FinallyOnce.class"
javap -classpath "$WORK/original" -c -p FinallyOnce > "$OUT/original.javap.txt"
javap -classpath "$WORK/handled-only" -c -p FinallyOnce > "$OUT/handled-only.javap.txt"
javap -classpath "$WORK/acceptance" -c -p FinallyOnce > "$OUT/acceptance.javap.txt"
python3 - "$OUT/original.javap.txt" "$OUT/handled-only.javap.txt" "$OUT/acceptance.javap.txt" <<'PY'
import re, sys
from pathlib import Path
def shape(path):
    text = Path(path).read_text()
    body = text.split('public static void escaping();', 1)[1].split('public static int count();', 1)[0]
    instructions = re.findall(r'^\s*(\d+):\s+([a-z][a-z0-9_]*)', body, re.M)
    rows = re.findall(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|Class \S+)', body, re.M)
    return instructions, rows
original, handled_only, acceptance = map(shape, sys.argv[1:])
if original != handled_only or original != acceptance:
    raise SystemExit(f'escaping BCI/opcode/异常行变化: {original} != {handled_only} != {acceptance}')
print('escaping_bci_opcode_rows_equal=true')
PY

java -Xverify:all -cp "$WORK/original" FinallyOnce > "$OUT/original-full.run.txt"
[[ "$(tail -n 1 "$OUT/original-full.run.txt")" == state:1 ]]
java -Xverify:all -cp "$WORK/handled-only" FinallyOnce > "$OUT/handled-only.run.txt"
[[ "$(tail -n 1 "$OUT/handled-only.run.txt")" == state:1 ]]
java -Xverify:all -cp "$WORK/acceptance" Runner > "$OUT/acceptance-original.run.txt"
javac --release 8 -g -Xlint:-options -d "$WORK/fixed-jadx-classes" "$FIXED/jadx/sources/defpackage/FinallyOnce.java" "$HERE/JadxRunner.java"
java -Xverify:all -cp "$WORK/fixed-jadx-classes" defpackage.JadxRunner > "$OUT/fixed-jadx.run.txt"
[[ "$(cat "$OUT/fixed-jadx.run.txt")" == java.lang.IllegalStateException:state:1 ]]
"$JADX" --no-res -d "$WORK/jadx" "$WORK/acceptance/FinallyOnce.class" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
javac --release 8 -g -Xlint:-options -d "$WORK/jadx-classes" "$WORK/jadx/sources/defpackage/FinallyOnce.java" "$HERE/JadxRunner.java"
java -Xverify:all -cp "$WORK/jadx-classes" defpackage.JadxRunner > "$OUT/acceptance-jadx.run.txt"
cp "$WORK/jadx/sources/defpackage/FinallyOnce.java" "$OUT/acceptance-jadx.java"
"$CLI" class-source --input "$WORK/acceptance/FinallyOnce.class" --class FinallyOnce --policy single-class --release 8 --format text > "$WORK/jarde/FinallyOnce.java" 2> "$OUT/acceptance-jarde.report.txt"
cp "$WORK/jarde/FinallyOnce.java" "$OUT/acceptance-jarde.java"
! sed -n '/public static void escaping/,/public static int count/p' "$WORK/jarde/FinallyOnce.java" | rg -q '@bytecode|finally \{'
javac --release 8 -g -Xlint:-options -d "$WORK/jarde" "$WORK/jarde/FinallyOnce.java" "$HERE/Runner.java"
java -Xverify:all -cp "$WORK/jarde" Runner > "$OUT/acceptance-jarde.run.txt"
for source in acceptance-original acceptance-jadx acceptance-jarde; do
  [[ "$(cat "$OUT/$source.run.txt")" == java.lang.IllegalStateException:state:1 ]]
done
mkdir -p "$WORK/identity-original" "$WORK/identity-jarde"
javac --release 8 -g -Xlint:-options -d "$WORK/identity-original" "$HERE/IdentityOnce.java" "$HERE/IdentityRunner.java"
cmp "$WORK/identity-original/IdentityOnce.class" "$HERE/v8/IdentityOnce.class"
java -Xverify:all -cp "$WORK/identity-original" IdentityRunner > "$OUT/identity-original.run.txt"
"$CLI" class-source --input "$WORK/identity-original/IdentityOnce.class" --class IdentityOnce --policy single-class --release 8 --format text > "$WORK/identity-jarde/IdentityOnce.java" 2> "$OUT/identity-jarde.report.txt"
cp "$WORK/identity-jarde/IdentityOnce.java" "$OUT/identity-jarde.java"
javac --release 8 -g -Xlint:-options -d "$WORK/identity-jarde" "$WORK/identity-jarde/IdentityOnce.java" "$HERE/IdentityRunner.java"
java -Xverify:all -cp "$WORK/identity-jarde" IdentityRunner > "$OUT/identity-jarde.run.txt"
[[ "$(cat "$OUT/identity-original.run.txt")" == true:java.lang.IllegalStateException:same:1 ]]
cmp "$OUT/identity-original.run.txt" "$OUT/identity-jarde.run.txt"

javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED -d "$WORK/patcher" "$HERE/PatchNeighbor.java"
for kind in normal-exit range-shrunk range-expanded competing-row external-entry wrong-rethrow branch-cleanup; do
  mkdir -p "$WORK/$kind"
  java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED -cp "$WORK/patcher" PatchNeighbor "$WORK/acceptance/FinallyOnce.class" "$WORK/$kind/FinallyOnce.class" "$kind"
  cmp "$WORK/$kind/FinallyOnce.class" "$HERE/negatives/$kind.class"
  cp "$WORK/acceptance/NeighborRunner.class" "$WORK/$kind/"
  java -Xverify:all -cp "$WORK/$kind" NeighborRunner > "$OUT/$kind.run.txt"
  case "$kind" in
    normal-exit) [[ "$(cat "$OUT/$kind.run.txt")" == $'returned:0\nreturned:0' ]] ;;
    range-shrunk|range-expanded|competing-row) [[ "$(cat "$OUT/$kind.run.txt")" == $'java.lang.IllegalStateException:state:1\njava.lang.IllegalStateException:state:1' ]] ;;
    external-entry) [[ "$(head -n 1 "$OUT/$kind.run.txt")" == java.lang.IllegalStateException:state:1 ]] && tail -n 1 "$OUT/$kind.run.txt" | rg -q '^java.lang.NullPointerException:.*:2$' ;;
    wrong-rethrow) [[ "$(wc -l < "$OUT/$kind.run.txt" | tr -d ' ')" == 2 ]] && rg -c '^java.lang.NullPointerException:.*:1$' "$OUT/$kind.run.txt" | rg -q '^2$' ;;
    branch-cleanup) [[ "$(cat "$OUT/$kind.run.txt")" == $'java.lang.IllegalStateException:state:0\njava.lang.IllegalStateException:state:0' ]] ;;
  esac
  "$CLI" class-source --input "$WORK/$kind/FinallyOnce.class" --class FinallyOnce --policy single-class --release 8 --format text > "$OUT/$kind.jarde.java" 2> "$OUT/$kind.jarde.report.txt"
  ! sed -n '/public static void escaping/,/public static int count/p' "$OUT/$kind.jarde.java" | rg -q 'catch \(java.lang.Throwable|finally \{'
done

shasum -a 256 "$FIXED/original/FinallyOnce.class" "$HERE/v8/"*.class "$HERE/negatives/"*.class > "$OUT/class-sha256.txt"
printf 'jadx_head=%s\ncli_sha256=%s\n' "$(git -C "$JADX_ROOT" rev-parse HEAD)" "$(shasum -a 256 "$CLI" | awk '{print $1}')" > "$OUT/toolchain.txt"
echo "三方验收和七个 verifier 有效近邻通过；输出: $OUT"
