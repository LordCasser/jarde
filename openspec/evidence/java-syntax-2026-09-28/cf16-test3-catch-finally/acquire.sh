#!/bin/sh
set -eu

EVIDENCE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
JARDE_ROOT=$(git -C "$EVIDENCE_DIR" rev-parse --show-toplevel)
JADX_ROOT=${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_SOURCE_SHA=d61f1f6b7c092af2e5d947c8def626887cb0dc8adb40d6c68dec3be96f432acd
EXPECTED_CLASS_SHA=9c3dc61868a81eb87f90228c5ddad4336ca4ae540621e5cb9b1589474c90e71e
JADX_SOURCE=$JADX_ROOT/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally3.java
JADX_CLASS=$JADX_ROOT/jadx-core/build/classes/java/test/jadx/tests/integration/trycatch/TestTryCatchFinally3\$TestCls.class
TMP_TARGET=$(mktemp -d /private/tmp/jarde-cf16-test3-target.XXXXXX)
TMP_JADX_OUT=$(mktemp -d /private/tmp/cf16-jadx.XXXXXX)
trap 'rm -rf "$TMP_TARGET" "$TMP_JADX_OUT"' EXIT HUP INT TERM

actual_head=$(git -C "$JADX_ROOT" rev-parse HEAD)
actual_source_sha=$(shasum -a 256 "$JADX_SOURCE" | awk '{print $1}')
actual_class_sha=$(shasum -a 256 "$JADX_CLASS" | awk '{print $1}')
test "$actual_head" = "$EXPECTED_JADX_HEAD"
test "$actual_source_sha" = "$EXPECTED_SOURCE_SHA"
test "$actual_class_sha" = "$EXPECTED_CLASS_SHA"

cp "$JADX_SOURCE" "$EVIDENCE_DIR/TestTryCatchFinally3.java"
cp "$JADX_CLASS" "$EVIDENCE_DIR/TestTryCatchFinally3\$TestCls.class"
javap -c -v "$EVIDENCE_DIR/TestTryCatchFinally3\$TestCls.class" > "$EVIDENCE_DIR/bytecode/TestCls.javap.txt"

(
    cd "$JADX_ROOT"
    ./gradlew :jadx-cli:run --offline --args="-d $TMP_JADX_OUT $EVIDENCE_DIR/TestTryCatchFinally3\$TestCls.class"
) > "$EVIDENCE_DIR/results/jadx-cli.log" 2>&1
tr -d '\r' < "$EVIDENCE_DIR/results/jadx-cli.log" \
    | sed -E 's/[[:blank:]]+$//' \
    > "$EVIDENCE_DIR/results/jadx-cli.clean"
mv "$EVIDENCE_DIR/results/jadx-cli.clean" "$EVIDENCE_DIR/results/jadx-cli.log"
cp "$TMP_JADX_OUT/sources/jadx/tests/integration/trycatch/TestTryCatchFinally3\$TestCls.java" "$EVIDENCE_DIR/jadx-src/TestTryCatchFinally3\$TestCls.java"

cargo build --offline --locked --target-dir "$TMP_TARGET" -p jarde-cli > "$EVIDENCE_DIR/results/cargo-build.log" 2>&1
"$TMP_TARGET/debug/jarde-cli" class-source \
    --input "$EVIDENCE_DIR/TestTryCatchFinally3\$TestCls.class" \
    --policy single-class \
    --class 'jadx/tests/integration/trycatch/TestTryCatchFinally3$TestCls' \
    > "$EVIDENCE_DIR/jarde-out/TestTryCatchFinally3\$TestCls.java" \
    2> "$EVIDENCE_DIR/results/jarde-report.txt"
"$TMP_TARGET/debug/jarde-cli" recover \
    --input "$EVIDENCE_DIR/TestTryCatchFinally3\$TestCls.class" \
    --policy single-class \
    --class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally3$TestCls' \
    --method-name test \
    --descriptor '(Ljadx/core/dex/nodes/ClassNode;Ljava/util/List;)V' \
    > "$EVIDENCE_DIR/jarde-out/test.recovery.txt" \
    2> "$EVIDENCE_DIR/results/jarde-recover-report.txt"

"$EVIDENCE_DIR/run.sh"
{
    printf 'JADX HEAD %s\n' "$actual_head"
    printf 'JADX source SHA-256 %s\n' "$actual_source_sha"
    printf 'JADX TestCls class SHA-256 %s\n' "$actual_class_sha"
    printf 'Jarde HEAD %s\n' "$(git -C "$JARDE_ROOT" rev-parse HEAD)"
} > "$EVIDENCE_DIR/results/identities.txt"
shasum -a 256 \
    "$EVIDENCE_DIR/TestTryCatchFinally3.java" \
    "$EVIDENCE_DIR/TestTryCatchFinally3\$TestCls.class" \
    "$EVIDENCE_DIR/jadx-src/TestTryCatchFinally3\$TestCls.java" \
    "$EVIDENCE_DIR/jarde-out/TestTryCatchFinally3\$TestCls.java" \
    > "$EVIDENCE_DIR/results/source-and-class-sha256.txt"
