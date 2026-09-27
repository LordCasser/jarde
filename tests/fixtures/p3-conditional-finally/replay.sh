#!/usr/bin/env bash
set -euo pipefail

if [[ $# -ne 2 ]]; then
  echo "usage: $0 JARDE_CLI OUTPUT_DIR" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
ROOT="$(git -C "$HERE" rev-parse --show-toplevel)"
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test14-conditional-cleanup"
CLI="$1"
OUT="$2"
WORK="$(mktemp -d /private/tmp/jarde-cf16-test14-replay.XXXXXX)"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$OUT" "$WORK/minimal" "$WORK/jarde-src" "$WORK/jarde-classes" "$WORK/patcher"
[[ -x "$CLI" ]]

"$EVIDENCE/replay-fixtures.sh" "$OUT/baseline" > "$OUT/baseline-summary.txt"
javac --release 8 -g -Xlint:-options -d "$WORK/minimal" \
  "$EVIDENCE/minimal/TestTryCatchFinally14.java" "$EVIDENCE/minimal/Runner.java"
PACKAGE=jadx/tests/integration/trycatch
TARGET='TestTryCatchFinally14$TestCls'
[[ "$(shasum -a 256 "$WORK/minimal/$PACKAGE/$TARGET.class" | awk '{print $1}')" == 356108017a0d24be26105ea227e93e3089a8832411ba938d3e3ece4193ed35e0 ]]
cmp "$WORK/minimal/$PACKAGE/$TARGET.class" "$HERE/Test14-minimal.class"

for binary in TestTryCatchFinally14 'TestTryCatchFinally14$TestCls' 'TestTryCatchFinally14$TestCls$TCls'; do
  "$CLI" class-source --input "$WORK/minimal/$PACKAGE/$binary.class" \
    --class "$PACKAGE/$binary" --policy single-class --release 8 --format text \
    > "$WORK/jarde-src/$binary.java" 2> "$OUT/$binary.report.txt"
  cp "$WORK/jarde-src/$binary.java" "$OUT/$binary.jarde.java"
  ! rg -q '@bytecode' "$WORK/jarde-src/$binary.java"
done
[[ "$(rg -oF 'finally {' "$WORK/jarde-src/$TARGET.java" | wc -l | tr -d '[:space:]')" == 1 ]]
[[ "$(rg -oF 'this.t != null' "$WORK/jarde-src/$TARGET.java" | wc -l | tr -d '[:space:]')" == 2 ]]
[[ "$(rg -oF 'this.t.doSomething();' "$WORK/jarde-src/$TARGET.java" | wc -l | tr -d '[:space:]')" == 1 ]]
[[ "$(rg -oF 'this.t.doFinally();' "$WORK/jarde-src/$TARGET.java" | wc -l | tr -d '[:space:]')" == 1 ]]
# The frozen Runner refers to the nested source spelling. The generated class source names the
# same binary owner as a top-level `$` class; only the runner's type spelling changes.
sed 's/TestTryCatchFinally14\.TestCls/TestTryCatchFinally14$TestCls/g' \
  "$EVIDENCE/minimal/Runner.java" > "$WORK/jarde-src/Runner.java"
javac --release 8 -g -Xlint:-options -d "$WORK/jarde-classes" "$WORK/jarde-src/"*.java
java -Xverify:all -cp "$WORK/jarde-classes" jadx.tests.integration.trycatch.Runner > "$OUT/jarde.run.txt"
cmp "$OUT/jarde.run.txt" "$OUT/baseline/frozen.run.txt"
cmp "$OUT/jarde.run.txt" "$OUT/baseline/jadx.run.txt"
cmp "$OUT/jarde.run.txt" "$EVIDENCE/expected/original.run.txt"

for mutant in field call predicate self-protected external-entry throwable; do
  "$CLI" class-source --input "$EVIDENCE/negatives/Test14-$mutant.class" \
    --class "$PACKAGE/$TARGET" --policy single-class --release 8 --format text \
    > "$OUT/negative-$mutant.jarde.java" 2> "$OUT/negative-$mutant.report.txt"
  ! sed -n '/public void test()/,/public void clearT()/p' "$OUT/negative-$mutant.jarde.java" | rg -q 'finally \{'
  sed -n '/public void test()/,/public void clearT()/p' "$OUT/negative-$mutant.jarde.java" | rg -q '@bytecode'
done

ASM_EXPORTS=(
  --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED
  --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED
)
javac "${ASM_EXPORTS[@]}" -d "$WORK/patcher" "$HERE/Test14Neighbor.java"
for mutant in slot0 second-field handler-call; do
  java "${ASM_EXPORTS[@]}" -cp "$WORK/patcher" Test14Neighbor "$mutant" \
    "$HERE/Test14-minimal.class" "$WORK/minimal/$PACKAGE/$TARGET.class"
  cmp "$WORK/minimal/$PACKAGE/$TARGET.class" "$HERE/Test14-$mutant.class"
  java -Xverify:all -cp "$WORK/minimal" jadx.tests.integration.trycatch.Runner > "$OUT/negative-$mutant.run.txt"
  if [[ "$mutant" == slot0 ]]; then
    cmp "$OUT/negative-$mutant.run.txt" "$EVIDENCE/expected/original.run.txt"
  fi
  "$CLI" class-source --input "$WORK/minimal/$PACKAGE/$TARGET.class" \
    --class "$PACKAGE/$TARGET" --policy single-class --release 8 --format text \
    > "$OUT/negative-$mutant.jarde.java" 2> "$OUT/negative-$mutant.report.txt"
  ! sed -n '/public void test()/,/public void clearT()/p' "$OUT/negative-$mutant.jarde.java" | rg -q 'finally \{'
  sed -n '/public void test()/,/public void clearT()/p' "$OUT/negative-$mutant.jarde.java" | rg -q '@bytecode'
done

shasum -a 256 "$EVIDENCE/TestTryCatchFinally14\$TestCls.class" "$HERE"/Test14-*.class > "$OUT/class-sha256.txt"
printf 'cli_sha256=%s\n' "$(shasum -a 256 "$CLI" | awk '{print $1}')" > "$OUT/toolchain.txt"
echo "original, pinned JADX, and Jarde seven-path Java 8 replay passed; nine verifier-valid neighbors refused: $OUT"
