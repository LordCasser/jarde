#!/usr/bin/env bash
set -euo pipefail

EVIDENCE_DIR=$(cd -- "$(dirname -- "$0")" && pwd)
REPO=${JARDE_REPO:-/Users/lordcasser/workspace/projects/jarde}
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
TARGET="$RUN/cargo-target"
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
if javac --release 8 -g:none -Xlint:-options -cp "$RUN/original" -d "$RUN/jarde-classes" \
  "$RUN/jarde/TestTryCatchFinally\$TestCls.java" "$EVIDENCE_DIR/fixture/RunnerReflect.java" \
  > "$RUN/jarde-javac.txt" 2>&1; then
  printf 'expected the current Jarde complete source to fail Java 8 compilation\n' >&2
  exit 1
fi
if ! rg -q '缺少返回语句|missing return statement' "$RUN/jarde-javac.txt"; then
  printf 'Jarde compilation failed for a reason other than the incomplete test() body\n' >&2
  exit 1
fi

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

printf 'JADX checkout: %s\n' "$actual_jadx_sha"
printf 'JADX binary: %s (%s)\n' "$JADX_BIN" "$actual_jadx_version"
printf 'Original class: %s\n' "$RUN/original/TestTryCatchFinally\$TestCls.class"
printf 'Replay outputs: %s\n' "$RUN"
printf 'Jarde complete-source compile failure: expected\n'
printf 'Negative class: verifier passed; check() failed as expected; Jarde still refused test()\n'
