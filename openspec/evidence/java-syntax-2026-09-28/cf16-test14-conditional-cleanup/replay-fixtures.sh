#!/usr/bin/env bash
set -euo pipefail

if [[ $# -gt 1 ]]; then
  echo "usage: $0 [OUTPUT_DIR]" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
JADX_ROOT=/Users/lordcasser/workspace/testzone/jadx
JADX="$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx"
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_MINIMAL_CLASS=356108017a0d24be26105ea227e93e3089a8832411ba938d3e3ece4193ed35e0
TARGET='jadx.tests.integration.trycatch.TestTryCatchFinally14$TestCls'
PACKAGE=jadx/tests/integration/trycatch
OUT="${1:-$(mktemp -d /tmp/jarde-cf16-test14-fixtures-output.XXXXXX)}"
WORK="$(mktemp -d /tmp/jarde-cf16-test14-fixtures-work.XXXXXX)"
trap 'find "$WORK" -depth -delete' EXIT

if [[ ! -x "$JADX" ]] || [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_JADX_HEAD" ]]; then
  echo "pinned JADX checkout or installed CLI unavailable" >&2
  exit 3
fi
mkdir -p "$OUT" "$WORK/base" "$WORK/minimal" "$WORK/jadx" "$WORK/jadx-classes" "$WORK/patcher" "$WORK/mutated"

javac --release 8 -g -Xlint:-options -d "$WORK/base" \
  "$HERE/TestTryCatchFinally14.java" "$HERE/Runner.java"
javac --release 8 -g -Xlint:-options -d "$WORK/minimal" \
  "$HERE/minimal/TestTryCatchFinally14.java" "$HERE/minimal/Runner.java"
BASE_CLASS="$WORK/base/$PACKAGE/TestTryCatchFinally14\$TestCls.class"
MINIMAL_CLASS="$WORK/minimal/$PACKAGE/TestTryCatchFinally14\$TestCls.class"
[[ "$(shasum -a 256 "$BASE_CLASS" | awk '{print $1}')" == \
   "$(shasum -a 256 "$HERE/TestTryCatchFinally14\$TestCls.class" | awk '{print $1}')" ]]
if [[ "$(shasum -a 256 "$MINIMAL_CLASS" | awk '{print $1}')" != "$EXPECTED_MINIMAL_CLASS" ]]; then
  echo "minimal Test14 target class hash changed" >&2
  exit 4
fi
javap -classpath "$WORK/base" -p -c -v "$TARGET" > "$OUT/frozen.javap.txt"
javap -classpath "$WORK/minimal" -p -c -v "$TARGET" > "$OUT/minimal.javap.txt"
python3 - "$OUT/frozen.javap.txt" "$OUT/minimal.javap.txt" <<'PY'
import re
import sys
from pathlib import Path

def target(path):
    text = Path(path).read_text()
    start = text.index("  public void test();")
    end = text.index("  LineNumberTable:", start)
    method = text[start:end]
    instructions = []
    for line in method.splitlines():
        match = re.match(r"\s*(\d+):\s+(.+)", line)
        if match:
            normalized = re.sub(r"#\d+", "#", match.group(1) + ": " + match.group(2)).strip()
            instructions.append(re.sub(r"\s+", " ", normalized))
    rows = re.findall(r"^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|Class .+)$", method, re.M)
    if rows != [("0", "14", "31", "any")]:
        raise SystemExit(f"expected exactly one catch-all row in {path}: {rows}")
    return instructions, rows

frozen = target(sys.argv[1])
minimal = target(sys.argv[2])
if frozen != minimal:
    raise SystemExit("minimal test() BCI/opcode/operand/exception-row shape differs from frozen Test14")
PY
if javap -classpath "$WORK/minimal" -p "$TARGET" | rg -q 'access\$[0-9]'; then
  echo "minimal fixture still has synthetic accessor methods" >&2
  exit 4
fi
java -Xverify:all -cp "$WORK/base" jadx.tests.integration.trycatch.Runner > "$OUT/frozen.run.txt"
java -Xverify:all -cp "$WORK/minimal" jadx.tests.integration.trycatch.Runner > "$OUT/minimal.run.txt"
cmp "$OUT/frozen.run.txt" "$OUT/minimal.run.txt"
cmp "$OUT/frozen.run.txt" "$HERE/expected/original.run.txt"

jar --create --file "$WORK/minimal.jar" -C "$WORK/minimal" .
"$JADX" --no-res -d "$WORK/jadx" "$WORK/minimal.jar" > "$OUT/jadx.stdout.txt" 2> "$OUT/jadx.stderr.txt"
JADX_SOURCE="$WORK/jadx/sources/$PACKAGE/TestTryCatchFinally14.java"
cp "$JADX_SOURCE" "$OUT/jadx.java.txt"
[[ "$(rg -oF 'finally {' "$JADX_SOURCE" | wc -l | tr -d '[:space:]')" == 1 ]]
[[ "$(rg -oF '.doFinally();' "$JADX_SOURCE" | wc -l | tr -d '[:space:]')" == 1 ]]
[[ "$(rg -oF '!= null' "$JADX_SOURCE" | wc -l | tr -d '[:space:]')" == 2 ]]
javac --release 8 -g -Xlint:-options -d "$WORK/jadx-classes" \
  "$JADX_SOURCE" "$HERE/minimal/Runner.java"
java -Xverify:all -cp "$WORK/jadx-classes" jadx.tests.integration.trycatch.Runner > "$OUT/jadx.run.txt"
cmp "$OUT/frozen.run.txt" "$OUT/jadx.run.txt"
cmp "$OUT/jadx.run.txt" "$HERE/expected/original.run.txt"

ASM_EXPORTS=(
  --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED
  --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED
)
javac "${ASM_EXPORTS[@]}" -d "$WORK/patcher" "$HERE/MutateTest14.java"
java "${ASM_EXPORTS[@]}" -cp "$WORK/patcher" MutateTest14 \
  "$MINIMAL_CLASS" "$WORK/mutated"
for mutant in field call predicate self-protected external-entry throwable; do
  cmp "$WORK/mutated/$mutant.class" "$HERE/negatives/Test14-${mutant}.class"
  mkdir -p "$WORK/run-$mutant/$PACKAGE"
  cp -R "$WORK/minimal/$PACKAGE"/. "$WORK/run-$mutant/$PACKAGE/"
  cp "$WORK/mutated/$mutant.class" "$WORK/run-$mutant/$PACKAGE/TestTryCatchFinally14\$TestCls.class"
  java -Xverify:all -cp "$WORK/run-$mutant" jadx.tests.integration.trycatch.Runner \
    > "$OUT/negative-$mutant.run.txt"
  cmp "$OUT/negative-$mutant.run.txt" "$HERE/expected/negative-$mutant.run.txt"
  shasum -a 256 "$WORK/mutated/$mutant.class" > "$OUT/negative-$mutant.sha256"
  javap -classpath "$WORK/run-$mutant" -verbose "$TARGET" > "$OUT/negative-$mutant.javap.txt"
  rg -q 'major version: 52' "$OUT/negative-$mutant.javap.txt"
done

rg -q '^normal=body,ok$' "$OUT/negative-field.run.txt"
rg -q '^normal=body,alternate,ok$' "$OUT/negative-call.run.txt"
rg -q '^null=NullPointerException:' "$OUT/negative-predicate.run.txt"
rg -q '^finally-throw=body,finally,finally,IllegalArgumentException:finally$' \
  "$OUT/negative-self-protected.run.txt"
rg -q '^null=NullPointerException:' "$OUT/negative-external-entry.run.txt"
rg -q '^body-throw=body,finally,NullPointerException:' "$OUT/negative-throwable.run.txt"
(cd "$HERE/negatives" && shasum -a 256 -c SHA256SUMS)

printf 'pinned_jadx_head=%s\n' "$EXPECTED_JADX_HEAD" > "$OUT/results.txt"
printf 'frozen_target_sha256=%s\n' "$(shasum -a 256 "$HERE/TestTryCatchFinally14\$TestCls.class" | awk '{print $1}')" >> "$OUT/results.txt"
printf 'minimal_target_sha256=%s\n' "$(shasum -a 256 "$MINIMAL_CLASS" | awk '{print $1}')" >> "$OUT/results.txt"
printf 'original_equals_jadx_seven_paths=true\nsynthetic_accessors=false\n' >> "$OUT/results.txt"
for mutant in field call predicate self-protected external-entry throwable; do
  printf 'negative_%s=verifier_valid\n' "$mutant" >> "$OUT/results.txt"
done
cat "$OUT/results.txt"
printf 'output_dir=%s\n' "$OUT"
