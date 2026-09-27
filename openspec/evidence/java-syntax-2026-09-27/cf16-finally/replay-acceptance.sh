#!/usr/bin/env bash
set -euo pipefail

EVIDENCE_DIR=$(cd -- "$(dirname -- "$0")" && pwd)
REPO=${JARDE_REPO:-$(cd "$EVIDENCE_DIR/../../../.." && pwd)}
JADX_CHECKOUT=${JADX_CHECKOUT:-/Users/lordcasser/workspace/testzone/jadx}
JADX_BIN=${JADX_BIN:-$JADX_CHECKOUT/jadx-cli/build/install/jadx/bin/jadx}
EXPECTED_JADX_SHA=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_JADX_VERSION=dev

actual_jadx_sha=$(git -C "$JADX_CHECKOUT" rev-parse HEAD)
if [[ "$actual_jadx_sha" != "$EXPECTED_JADX_SHA" ]]; then
  printf 'wrong JADX checkout: expected %s, got %s\n' "$EXPECTED_JADX_SHA" "$actual_jadx_sha" >&2
  exit 1
fi
actual_jadx_version=$("$JADX_BIN" --version 2>&1)
if [[ "$actual_jadx_version" != "$EXPECTED_JADX_VERSION" ]]; then
  printf 'wrong pinned-checkout JADX binary: expected %s, got %s\n' "$EXPECTED_JADX_VERSION" "$actual_jadx_version" >&2
  exit 1
fi

RUN=$(mktemp -d /tmp/jarde-cf16-finally-replay.XXXXXX)
TARGET=/private/tmp/jarde-cf16-shared-join-target
cleanup() {
  CARGO_TARGET_DIR="$TARGET" CARGO_INCREMENTAL=0 cargo clean --manifest-path "$REPO/Cargo.toml" >/dev/null || true
}
trap cleanup EXIT
mkdir -p "$RUN/original" "$RUN/jarde" "$RUN/jadx" "$RUN/jadx-classes" "$RUN/jadx-no-finally" "$RUN/jadx-no-finally-classes" "$RUN/jarde-classes" "$RUN/negative" "$RUN/patcher"

javac --release 8 -g:none -Xlint:-options -d "$RUN/original" \
  "$EVIDENCE_DIR/fixture/JadxAssertions.java" \
  "$EVIDENCE_DIR/fixture/TestTryCatchFinally.java" \
  "$EVIDENCE_DIR/fixture/RunnerReflect.java"
java -Xverify:all -cp "$RUN/original" RunnerReflect 'TestTryCatchFinally$TestCls' \
  > "$RUN/original-run.stdout.txt" 2> "$RUN/original-run.stderr.txt"
javap -classpath "$RUN/original" -p -c -v 'TestTryCatchFinally$TestCls' \
  > "$RUN/original.javap.txt"

"$JADX_BIN" --no-res --single-class 'TestTryCatchFinally$TestCls' \
  -d "$RUN/jadx" "$RUN/original/TestTryCatchFinally\$TestCls.class" \
  > "$RUN/jadx.log" 2>&1
JADX_SOURCE="$RUN/jadx/sources/defpackage/TestTryCatchFinally\$TestCls.java"
rg -q 'finally \{' "$JADX_SOURCE"
[[ $(rg -o 'this\.f = true;' "$JADX_SOURCE" | wc -l | tr -d '[:space:]') == 1 ]]
javac --release 8 -g:none -Xlint:-options -cp "$RUN/original" -d "$RUN/jadx-classes" \
  "$JADX_SOURCE" "$EVIDENCE_DIR/fixture/RunnerReflect.java"
java -Xverify:all -cp "$RUN/jadx-classes:$RUN/original" RunnerReflect 'defpackage.TestTryCatchFinally$TestCls' \
  > "$RUN/jadx-run.stdout.txt" 2> "$RUN/jadx-run.stderr.txt"

"$JADX_BIN" --no-res --no-finally --single-class 'TestTryCatchFinally$TestCls' \
  -d "$RUN/jadx-no-finally" "$RUN/original/TestTryCatchFinally\$TestCls.class" \
  > "$RUN/jadx-no-finally.log" 2>&1
JADX_NO_FINALLY_SOURCE="$RUN/jadx-no-finally/sources/defpackage/TestTryCatchFinally\$TestCls.java"
rg -q 'catch \(Throwable th\)' "$JADX_NO_FINALLY_SOURCE"
[[ $(rg -o 'this\.f = true;' "$JADX_NO_FINALLY_SOURCE" | wc -l | tr -d '[:space:]') == 3 ]]
javac --release 8 -g:none -Xlint:-options -cp "$RUN/original" -d "$RUN/jadx-no-finally-classes" \
  "$JADX_NO_FINALLY_SOURCE" "$EVIDENCE_DIR/fixture/RunnerReflect.java"
java -Xverify:all -cp "$RUN/jadx-no-finally-classes:$RUN/original" RunnerReflect 'defpackage.TestTryCatchFinally$TestCls' \
  > "$RUN/jadx-no-finally-run.stdout.txt" 2> "$RUN/jadx-no-finally-run.stderr.txt"

CARGO_TARGET_DIR="$TARGET" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked -p jarde-cli -- \
  class-source --policy single-class --input "$RUN/original/TestTryCatchFinally\$TestCls.class" \
  --class 'TestTryCatchFinally$TestCls' --format text \
  > "$RUN/jarde-report.txt" 2> "$RUN/jarde-cli.stderr.txt"
python3 - "$RUN/jarde-report.txt" "$RUN/jarde/TestTryCatchFinally\$TestCls.java" <<'PY'
import sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
marker = '// jarde: presentation of `TestTryCatchFinally$TestCls`'
start = text.rfind(marker)
if start < 0:
    raise SystemExit('Jarde class source was not present in the CLI output')
Path(sys.argv[2]).write_text(text[start:])
PY
rg -q 'finally \{' "$RUN/jarde/TestTryCatchFinally\$TestCls.java"
[[ $(rg -o 'this\.f = true;' "$RUN/jarde/TestTryCatchFinally\$TestCls.java" | wc -l | tr -d '[:space:]') == 1 ]]
rg -q 'return this\.f;' "$RUN/jarde/TestTryCatchFinally\$TestCls.java"
if rg -q '@bytecode|not recovered' "$RUN/jarde/TestTryCatchFinally\$TestCls.java"; then
  printf 'Jarde fixed class still contains bytecode fallback\n' >&2
  exit 1
fi
javac --release 8 -g:none -Xlint:-options -cp "$RUN/original" -d "$RUN/jarde-classes" \
  "$RUN/jarde/TestTryCatchFinally\$TestCls.java" "$EVIDENCE_DIR/fixture/RunnerReflect.java" \
  > "$RUN/jarde-javac.txt" 2>&1
java -Xverify:all -cp "$RUN/jarde-classes:$RUN/original" RunnerReflect 'TestTryCatchFinally$TestCls' \
  > "$RUN/jarde-run.stdout.txt" 2> "$RUN/jarde-run.stderr.txt"
diff -u "$RUN/original-run.stdout.txt" "$RUN/jarde-run.stdout.txt"
diff -u "$RUN/original-run.stdout.txt" "$RUN/jadx-run.stdout.txt"
diff -u "$RUN/original-run.stdout.txt" "$RUN/jadx-no-finally-run.stdout.txt"

javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
  -d "$RUN/patcher" "$EVIDENCE_DIR/fixture/NegativePatch.java"
java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
  -cp "$RUN/patcher" NegativePatch \
  "$RUN/original/TestTryCatchFinally\$TestCls.class" \
  "$RUN/negative/TestTryCatchFinally\$TestCls.class"
java -Xverify:all -cp "$RUN/negative:$RUN/original" RunnerReflect 'TestTryCatchFinally$TestCls' \
  > "$RUN/negative-run.stdout.txt" 2> "$RUN/negative-run.stderr.txt" || negative_status=$?
if [[ ${negative_status:-0} -ne 1 ]] || ! rg -q 'AssertionError: expected true' "$RUN/negative-run.stderr.txt"; then
  printf 'verifier-valid mismatch negative did not reach its expected check() assertion failure\n' >&2
  exit 1
fi
CARGO_TARGET_DIR="$TARGET" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked -p jarde-cli -- \
  class-source --policy single-class --input "$RUN/negative/TestTryCatchFinally\$TestCls.class" \
  --class 'TestTryCatchFinally$TestCls' --format text \
  > "$RUN/jarde-negative-report.txt" 2> "$RUN/jarde-negative-cli.stderr.txt"
rg -q 'BCI 31:.*finally.*copy' "$RUN/jarde-negative-report.txt"

mkdir -p "$RUN/error-original" "$RUN/error-jarde" "$RUN/error-jarde-classes" "$RUN/row" "$RUN/edge"
javac --release 8 -g:none -Xlint:-options -cp "$RUN/original" -d "$RUN/error-original" \
  "$EVIDENCE_DIR/fixture/TestTryCatchFinallyError.java" \
  "$EVIDENCE_DIR/fixture/RunnerErrorReflect.java"
javap -classpath "$RUN/error-original" -p -c 'TestTryCatchFinallyError$TestCls' > "$RUN/error.javap.txt"
python3 - "$RUN/original.javap.txt" "$RUN/error.javap.txt" <<'PY'
import re, sys
from pathlib import Path
def shape(path):
    text = Path(path).read_text()
    method = text.split('private boolean test(java.lang.Object);', 1)[1].split('private static boolean exc', 1)[0]
    instructions = [(int(bci), op) for bci, op in re.findall(r'^\s*(\d+):\s+([a-z][a-z0-9_]*)', method, re.M)]
    rows = re.findall(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(Class java/lang/Exception|any)\s*$', method, re.M)
    return instructions, rows
fixed, error = shape(sys.argv[1]), shape(sys.argv[2])
if fixed != error:
    raise SystemExit(f'Error variant changed test(Object) layout: {fixed!r} != {error!r}')
PY
java -Xverify:all -cp "$RUN/error-original:$RUN/original" RunnerErrorReflect 'TestTryCatchFinallyError$TestCls' \
  > "$RUN/error-original.stdout.txt"
CARGO_TARGET_DIR="$TARGET" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked -p jarde-cli -- \
  class-source --policy single-class --input "$RUN/error-original/TestTryCatchFinallyError\$TestCls.class" \
  --class 'TestTryCatchFinallyError$TestCls' --format text \
  > "$RUN/error-jarde-report.txt" 2> "$RUN/error-jarde-cli.stderr.txt"
python3 - "$RUN/error-jarde-report.txt" "$RUN/error-jarde/TestTryCatchFinallyError\$TestCls.java" <<'PY'
import sys
from pathlib import Path
text = Path(sys.argv[1]).read_text()
marker = '// jarde: presentation of `TestTryCatchFinallyError$TestCls`'
start = text.rfind(marker)
if start < 0:
    raise SystemExit('Error-variant Jarde class source was absent')
Path(sys.argv[2]).write_text(text[start:])
PY
[[ $(rg -o 'this\.f = true;' "$RUN/error-jarde/TestTryCatchFinallyError\$TestCls.java" | wc -l | tr -d '[:space:]') == 1 ]]
if rg -q '@bytecode|not recovered' "$RUN/error-jarde/TestTryCatchFinallyError\$TestCls.java"; then
  printf 'Error variant still contains bytecode fallback\n' >&2
  exit 1
fi
javac --release 8 -g:none -Xlint:-options -cp "$RUN/original" -d "$RUN/error-jarde-classes" \
  "$RUN/error-jarde/TestTryCatchFinallyError\$TestCls.java" \
  "$EVIDENCE_DIR/fixture/RunnerErrorReflect.java"
java -Xverify:all -cp "$RUN/error-jarde-classes:$RUN/original" RunnerErrorReflect 'TestTryCatchFinallyError$TestCls' \
  > "$RUN/error-jarde.stdout.txt"
diff -u "$RUN/error-original.stdout.txt" "$RUN/error-jarde.stdout.txt"

javac --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
  --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED \
  -d "$RUN/patcher" "$EVIDENCE_DIR/fixture/SharedJoinVariants.java"
for variant in row edge; do
  mode=$variant
  if [[ "$variant" == edge ]]; then mode=join; fi
  java --add-exports java.base/jdk.internal.org.objectweb.asm=ALL-UNNAMED \
    --add-exports java.base/jdk.internal.org.objectweb.asm.tree=ALL-UNNAMED \
    -cp "$RUN/patcher" SharedJoinVariants "$mode" \
    "$RUN/original/TestTryCatchFinally\$TestCls.class" \
    "$RUN/$variant/TestTryCatchFinally\$TestCls.class"
  java -Xverify:all -cp "$RUN/$variant:$RUN/original" RunnerReflect 'TestTryCatchFinally$TestCls' \
    > "$RUN/$variant-run.stdout.txt" 2> "$RUN/$variant-run.stderr.txt"
  CARGO_TARGET_DIR="$TARGET" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 cargo run --locked -p jarde-cli -- \
    class-source --policy single-class --input "$RUN/$variant/TestTryCatchFinally\$TestCls.class" \
    --class 'TestTryCatchFinally$TestCls' --format text \
    > "$RUN/$variant-jarde-report.txt" 2> "$RUN/$variant-jarde-cli.stderr.txt"
  rg -q 'BCI 31:.*finally.*copy' "$RUN/$variant-jarde-report.txt"
  if rg -q '^        } finally \{' "$RUN/$variant-jarde-report.txt"; then
    printf '%s variant was incorrectly folded into finally\n' "$variant" >&2
    exit 1
  fi
done

printf 'JADX checkout: %s\n' "$actual_jadx_sha"
printf 'JADX binary: %s (%s)\n' "$JADX_BIN" "$actual_jadx_version"
printf 'Original class: %s\n' "$RUN/original/TestTryCatchFinally\$TestCls.class"
printf 'Replay outputs: %s\n' "$RUN"
printf 'Original/JADX/Jarde complete classes: Java 8 compile, verifier, normal/catch/check passed\n'
printf 'Error variant: original exception identity and final field preserved; Jarde source has one assignment\n'
printf 'Value, row and join variants: verifier passed; Jarde refused shared finally\n'
