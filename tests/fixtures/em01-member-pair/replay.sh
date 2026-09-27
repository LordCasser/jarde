#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 4 ]]; then
  echo "usage: $0 JARDE_CLI JADX_BIN JADX_CHECKOUT OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel)"
CLI="$1"
JADX="$2"
JADX_CHECKOUT="$3"
OUT="$4"
WORK="$(mktemp -d /private/tmp/jarde-em01-pair-replay.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT" "$WORK/original" "$WORK/jadx" "$WORK/jarde" "$WORK/patcher" "$WORK/negative"
[[ -x "$CLI" && -x "$JADX" ]]
[[ "$(git -C "$JADX_CHECKOUT" rev-parse HEAD)" == 2fb1b16386941660fda07e9017285aec40fcb37f ]]

javac --release 8 -g:none -Xlint:-options -d "$WORK/original" "$HERE/Shape.java" "$HERE/Runner.java"
for binary in Shape 'Shape$A' 'Shape$I'; do
  cmp "$WORK/original/em01/$binary.class" "$HERE/$binary.class"
done
java -Xverify:all -cp "$WORK/original" em01.Runner > "$OUT/original.run.txt"
printf '2:1\n' | cmp - "$OUT/original.run.txt"
jar cf "$WORK/shape.jar" -C "$WORK/original" .

"$JADX" -d "$WORK/jadx" "$WORK/shape.jar" > "$OUT/jadx.log" 2>&1
cp "$WORK/jadx/sources/em01/Shape.java" "$OUT/Shape.jadx.java"
mkdir -p "$WORK/jadx-classes" "$WORK/jadx-source"
cp "$OUT/Shape.jadx.java" "$WORK/jadx-source/Shape.java"
javac --release 8 -g:none -Xlint:-options -d "$WORK/jadx-classes" "$WORK/jadx-source/Shape.java" "$HERE/Runner.java"
java -Xverify:all -cp "$WORK/jadx-classes" em01.Runner > "$OUT/jadx.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jadx.run.txt"

for binary in Shape 'Shape$A' 'Shape$I'; do
  "$CLI" class-source --input "$WORK/shape.jar" --class "em01.$binary" \
    --policy plain-jar --release 8 --format text \
    > "$OUT/$binary.jarde.java" 2> "$OUT/$binary.report.txt"
done
cp "$OUT/Shape.jarde.java" "$WORK/Shape.java"
javac --release 8 -g:none -Xlint:-options -d "$WORK/jarde" "$WORK/Shape.java" "$HERE/Runner.java"
java -Xverify:all -cp "$WORK/jarde" em01.Runner > "$OUT/jarde.run.txt"
cmp "$OUT/original.run.txt" "$OUT/jarde.run.txt"
[[ "$(rg -c 'class A extends' "$OUT/Shape.jarde.java")" == 1 ]]
[[ "$(rg -c 'interface I \{' "$OUT/Shape.jarde.java")" == 1 ]]
[[ "$(rg -c 'abstract int test2\(\);' "$OUT/Shape.jarde.java")" == 1 ]]
[[ "$(rg -c 'abstract int test\(\);' "$OUT/Shape.jarde.java")" == 1 ]]
[[ "$(rg -c 'abstract int test3\(\);' "$OUT/Shape.jarde.java")" == 1 ]]

ASM_EXPORTS=(
  --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED
  --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED
)
javac "${ASM_EXPORTS[@]}" -d "$WORK/patcher" "$HERE/PairNeighbor.java"
for mode in wrong-self interface-default interface-static field signature third-child root-use; do
  mkdir -p "$WORK/negative/$mode/em01"
  cp "$HERE/Shape\$A.class" "$WORK/negative/$mode/em01/Shape\$A.class"
  cp "$WORK/original/em01/Runner.class" "$WORK/negative/$mode/em01/Runner.class"
  if [[ "$mode" == third-child || "$mode" == root-use ]]; then
    java "${ASM_EXPORTS[@]}" -cp "$WORK/patcher" PairNeighbor "$mode" \
      "$HERE/Shape.class" "$WORK/negative/$mode/em01/Shape.class"
    cmp "$WORK/negative/$mode/em01/Shape.class" "$HERE/Shape-$mode.class"
    cp "$HERE/Shape\$I.class" "$WORK/negative/$mode/em01/Shape\$I.class"
  else
    cp "$HERE/Shape.class" "$WORK/negative/$mode/em01/Shape.class"
    java "${ASM_EXPORTS[@]}" -cp "$WORK/patcher" PairNeighbor "$mode" \
      "$HERE/Shape\$I.class" "$WORK/negative/$mode/em01/Shape\$I.class"
    cmp "$WORK/negative/$mode/em01/Shape\$I.class" "$HERE/Shape-I-$mode.class"
  fi
  java -Xverify:all -cp "$WORK/negative/$mode" em01.Runner > "$OUT/negative-$mode.run.txt"
  cmp "$OUT/original.run.txt" "$OUT/negative-$mode.run.txt"
  jar cf "$WORK/negative-$mode.jar" -C "$WORK/negative/$mode" .
  "$CLI" class-source --input "$WORK/negative-$mode.jar" --class em01.Shape \
    --policy plain-jar --release 8 --format text \
    > "$OUT/negative-$mode.jarde.java" 2> "$OUT/negative-$mode.report.txt"
  ! rg -q 'class A extends|interface I \{' "$OUT/negative-$mode.jarde.java"
done

mkdir -p "$WORK/negative/missing-child/em01"
cp "$HERE/Shape.class" "$WORK/negative/missing-child/em01/Shape.class"
cp "$HERE/Shape\$A.class" "$WORK/negative/missing-child/em01/Shape\$A.class"
cat > "$WORK/LoadRoot.java" <<'JAVA'
package em01;
public class LoadRoot {
    public static void main(String[] args) throws Exception {
        System.out.println(Class.forName("em01.Shape").getDeclaredMethods().length);
    }
}
JAVA
javac --release 8 -g:none -Xlint:-options -d "$WORK/negative/missing-child" "$WORK/LoadRoot.java"
java -Xverify:all -cp "$WORK/negative/missing-child" em01.LoadRoot > "$OUT/negative-missing-child.run.txt"
printf '0\n' | cmp - "$OUT/negative-missing-child.run.txt"
jar cf "$WORK/negative-missing-child.jar" -C "$WORK/negative/missing-child" .
"$CLI" class-source --input "$WORK/negative-missing-child.jar" --class em01.Shape \
  --policy plain-jar --release 8 --format text \
  > "$OUT/negative-missing-child.jarde.java" 2> "$OUT/negative-missing-child.report.txt"
! rg -q 'class A extends|interface I \{' "$OUT/negative-missing-child.jarde.java"

mkdir -p "$WORK/generic"
javac --release 8 -g:none -Xlint:-options -d "$WORK/generic" \
  "$ROOT/openspec/evidence/java-syntax-2026-09-27/em01-declarations/input/em01/Generic.java"
jar cf "$WORK/generic.jar" -C "$WORK/generic" .
"$CLI" class-source --input "$WORK/generic.jar" --class em01.Generic \
  --policy plain-jar --release 8 --format text \
  > "$OUT/Generic.jarde.java" 2> "$OUT/Generic.report.txt"
! rg -q 'class A[ <{]' "$OUT/Generic.jarde.java"
rg -q '^member_family.projection.state = "refused"' "$OUT/Generic.report.txt"

shasum -a 256 "$HERE"/*.class > "$OUT/class-sha256.txt"
echo "original, pinned JADX and Jarde Shape Java 8 replay passed; eight verifier-valid neighbors refused: $OUT"
