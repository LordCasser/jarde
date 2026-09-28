#!/bin/sh
set -eu

EVIDENCE_DIR=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
JARDE_ROOT=$(git -C "$EVIDENCE_DIR" rev-parse --show-toplevel)
JADX_ROOT=${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_TEST_SHA=39c220b0081e01b1be7f60760bf6d7dbf7309b7ee3a5c8ec18a6c6fd74c99017
EXPECTED_CLASS_SHA=251e21b8ade9f6a9096d807dc31a9fcaae4e2a551600a7230b8f1b7f9f165381
EXPECTED_PROFILE_SHA=13fc172963c7a12b97ce500cb17fe1e893802ec227dc4ca3ac0955e5a0bd325c
EXPECTED_CLI_SHA=d5c53496bc9f534660417c60fb642e7440f0511189d655c1dbad76e0819cf562
TEST_SOURCE=$JADX_ROOT/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally9.java
PROFILE_SOURCE=$JADX_ROOT/jadx-core/src/test/java/jadx/tests/api/IntegrationTest.java
CLI_SOURCE=$JADX_ROOT/jadx-cli/src/main/java/jadx/cli/JadxCLIArgs.java
CLASS_FILE=$JADX_ROOT/jadx-core/build/classes/java/test/jadx/tests/integration/trycatch/TestTryCatchFinally9\$TestCls.class
TMP_TARGET=$(mktemp -d /private/tmp/jarde-cf16-test9-target.XXXXXX)
TMP_JADX_JAVA=$(mktemp -d /private/tmp/cf16-test9-java.XXXXXX)
TMP_JADX_DX=$(mktemp -d /private/tmp/cf16-test9-dx.XXXXXX)
trap 'rm -rf "$TMP_TARGET" "$TMP_JADX_JAVA" "$TMP_JADX_DX"' EXIT HUP INT TERM

actual_head=$(git -C "$JADX_ROOT" rev-parse HEAD)
actual_test_sha=$(shasum -a 256 "$TEST_SOURCE" | awk '{print $1}')
actual_class_sha=$(shasum -a 256 "$CLASS_FILE" | awk '{print $1}')
actual_profile_sha=$(shasum -a 256 "$PROFILE_SOURCE" | awk '{print $1}')
actual_cli_sha=$(shasum -a 256 "$CLI_SOURCE" | awk '{print $1}')
test "$actual_head" = "$EXPECTED_JADX_HEAD"
test "$actual_test_sha" = "$EXPECTED_TEST_SHA"
test "$actual_class_sha" = "$EXPECTED_CLASS_SHA"
test "$actual_profile_sha" = "$EXPECTED_PROFILE_SHA"
test "$actual_cli_sha" = "$EXPECTED_CLI_SHA"

cp "$TEST_SOURCE" "$EVIDENCE_DIR/TestTryCatchFinally9.java"
cp "$CLASS_FILE" "$EVIDENCE_DIR/TestTryCatchFinally9\$TestCls.class"
javap -c -v "$EVIDENCE_DIR/TestTryCatchFinally9\$TestCls.class" \
    | sed -E -e '1s@^Classfile .*@Classfile <fixed-TestCls.class>@' -e '2s@Last modified .*; size @Last modified <filesystem timestamp>; size @' \
    > "$EVIDENCE_DIR/bytecode/TestCls.javap.txt"

(
    cd "$JADX_ROOT"
    ./gradlew :jadx-cli:run --offline --args="-d $TMP_JADX_JAVA $EVIDENCE_DIR/TestTryCatchFinally9\$TestCls.class"
) > "$TMP_JADX_JAVA/gradle.log" 2>&1
(
    cd "$JADX_ROOT"
    ./gradlew :jadx-cli:run --offline --args="--use-dx -d $TMP_JADX_DX $EVIDENCE_DIR/TestTryCatchFinally9\$TestCls.class"
) > "$TMP_JADX_DX/gradle.log" 2>&1
{
    printf '%s\n' 'Command: ./gradlew :jadx-cli:run --offline --args="-d <output> <TestCls.class>"'
    printf '%s\n' 'Profile: Java-input (no --use-dx flag)'
    printf '%s\n' 'Result: success; source emitted'
} > "$EVIDENCE_DIR/results/jadx-java-input.log"
{
    printf '%s\n' 'Command: ./gradlew :jadx-cli:run --offline --args="--use-dx -d <output> <TestCls.class>"'
    printf '%s\n' 'Profile: Java bytecode conversion to DEX'
    if rg -q 'WARN  - DX convert failed, trying D8' "$TMP_JADX_DX/gradle.log"; then
        printf '%s\n' 'Observed: DX conversion failed; D8 fallback emitted source successfully'
    else
        printf '%s\n' 'Observed: conversion emitted source successfully'
    fi
} > "$EVIDENCE_DIR/results/jadx-dx.log"
sed -E 's@/\* JADX INFO: loaded from: .* \*/@/* JADX INFO: loaded from: <java-class-input> */@' \
    "$TMP_JADX_JAVA/sources/jadx/tests/integration/trycatch/TestTryCatchFinally9\$TestCls.java" \
    > "$EVIDENCE_DIR/jadx-java-input/TestTryCatchFinally9\$TestCls.java"
sed -E 's@/\* JADX INFO: loaded from: .* \*/@/* JADX INFO: loaded from: <dx-converted-class> */@' \
    "$TMP_JADX_DX/sources/jadx/tests/integration/trycatch/TestTryCatchFinally9\$TestCls.java" \
    > "$EVIDENCE_DIR/jadx-dx/TestTryCatchFinally9\$TestCls.java"

(
    cd "$JADX_ROOT"
    unset TEST_INPUT_PLUGIN
    ./gradlew :jadx-core:test --offline --rerun-tasks --tests jadx.tests.integration.trycatch.TestTryCatchFinally9
) > "$EVIDENCE_DIR/results/jadx-integration-test.raw.log" 2>&1
{
    printf '%s\n' 'TEST_INPUT_PLUGIN: unset (IntegrationTest default is dx)'
    printf '%s\n' 'Command: ./gradlew :jadx-core:test --offline --rerun-tasks --tests jadx.tests.integration.trycatch.TestTryCatchFinally9'
    rg '^> Task :jadx-core:test|^BUILD SUCCESSFUL' "$EVIDENCE_DIR/results/jadx-integration-test.raw.log"
} > "$EVIDENCE_DIR/results/jadx-integration-test.tmp.txt"
test_summary=$(sed -n -E '/<testsuite name="jadx\.tests\.integration\.trycatch\.TestTryCatchFinally9"/s/.* tests="([^"]*)" skipped="([^"]*)" failures="([^"]*)" errors="([^"]*)".*/tests=\1 skipped=\2 failures=\3 errors=\4/p' \
    "$JADX_ROOT/jadx-core/build/test-results/test/TEST-jadx.tests.integration.trycatch.TestTryCatchFinally9.xml")
test -n "$test_summary"
printf 'JUnit result: %s\n' "$test_summary" >> "$EVIDENCE_DIR/results/jadx-integration-test.tmp.txt"
sed -E 's/^BUILD SUCCESSFUL in .*$/BUILD SUCCESSFUL/' \
    "$EVIDENCE_DIR/results/jadx-integration-test.tmp.txt" \
    > "$EVIDENCE_DIR/results/jadx-integration-test.txt"
find "$EVIDENCE_DIR/results/jadx-integration-test.tmp.txt" -depth -delete
find "$EVIDENCE_DIR/results/jadx-integration-test.raw.log" -depth -delete

cargo build --offline --locked --target-dir "$TMP_TARGET" -p jarde-cli > /dev/null 2>&1
{
    printf '%s\n' 'Command: cargo build --offline --locked --target-dir <temporary /private/tmp target> -p jarde-cli'
    printf '%s\n' 'Result: success'
    printf '%s\n' 'Cleanup: temporary target removed on script exit'
} > "$EVIDENCE_DIR/results/cargo-build.txt"
find "$EVIDENCE_DIR/results" -maxdepth 1 \
    \( -name 'cargo-build.log' -o -name 'jarde-class-source-report.txt' -o -name 'jarde-method-recovery-report.txt' \) \
    -depth -delete
"$TMP_TARGET/debug/jarde-cli" class-source \
    --input "$EVIDENCE_DIR/TestTryCatchFinally9\$TestCls.class" \
    --policy single-class \
    --class 'jadx/tests/integration/trycatch/TestTryCatchFinally9$TestCls' \
    > "$EVIDENCE_DIR/jarde-out/TestTryCatchFinally9\$TestCls.java" \
    2> "$TMP_TARGET/class-source-report.txt"
"$TMP_TARGET/debug/jarde-cli" recover \
    --input "$EVIDENCE_DIR/TestTryCatchFinally9\$TestCls.class" \
    --policy single-class \
    --class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally9$TestCls' \
    --method-name test \
    --descriptor '()Ljava/lang/String;' \
    > "$EVIDENCE_DIR/jarde-out/test.recovery.txt" \
    2> "$TMP_TARGET/method-recovery-report.txt"

rg '^(execution\.status|methods\.1\.outcome\.(kind|report\.execution\.status|report\.syntax_status|text)) = ' \
    "$TMP_TARGET/class-source-report.txt" > "$EVIDENCE_DIR/results/jarde-class-source-summary.txt"
rg '^(outcome|presentation\.content|recovered\.analysis\.(compile_status|execution\.status)|recovered\.recovery\.(execution\.status|syntax_status)) = ' \
    "$TMP_TARGET/method-recovery-report.txt" > "$EVIDENCE_DIR/results/jarde-method-recovery-summary.txt"

{
    printf 'JADX HEAD %s\n' "$actual_head"
    printf 'JADX TestTryCatchFinally9.java SHA-256 %s\n' "$actual_test_sha"
    printf 'JADX TestCls.class SHA-256 %s\n' "$actual_class_sha"
    printf 'JADX IntegrationTest.java SHA-256 %s\n' "$actual_profile_sha"
    printf 'JADX JadxCLIArgs.java SHA-256 %s\n' "$actual_cli_sha"
    printf 'Jarde HEAD %s\n' "$(git -C "$JARDE_ROOT" rev-parse HEAD)"
} > "$EVIDENCE_DIR/results/identities.txt"
(
    cd "$EVIDENCE_DIR"
    shasum -a 256 \
        TestTryCatchFinally9.java \
        'TestTryCatchFinally9$TestCls.class' \
        'fixtures/TestTryCatchFinally9$TestCls.java' \
        'jadx-java-input/TestTryCatchFinally9$TestCls.java' \
        'jadx-dx/TestTryCatchFinally9$TestCls.java' \
        'jarde-out/TestTryCatchFinally9$TestCls.java'
) > "$EVIDENCE_DIR/results/source-and-class-sha256.txt"
"$EVIDENCE_DIR/run.sh"
