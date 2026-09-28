#!/bin/sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
JARDE_ROOT=$(git -C "$HERE" rev-parse --show-toplevel)
JADX_ROOT=${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}
JADX_BIN=${JADX_BIN:-$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx}
JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_SOURCE_SHA=d9729beaf977fc0236e29e6f3e9f0d7d7dde14102363c782100ac69426f56edf
EXPECTED_CLASS_SHA=4d5a64965763bf8c0a9b81cd0881a429bf715aa5d91110de8039fd992bc91827
EXPECTED_A_SHA=b51bb4ae123bbf98d12545f6c253b6c4bc7f48da96d0d7be9de7decd129af5ca
EXPECTED_B_SHA=c82451e026c8040073fd13ca17fb1106f6f07a2964f4f9570c84f91a5fdca81d
EXPECTED_C_SHA=b82b82c3c9ea414a012d815fb3d1e5d9ea7e74c8492bcc54fbb7d21bb33c5ef5
EXPECTED_D_SHA=7df63096bcf0aa9dbd624388394ff8dc482937fc7188b79fa94d1e9a87ff689d
EXPECTED_JARDE_BASELINE=2fb7c56053ed13baab250a4a12607748d4ce31a4
JADX_SOURCE=$JADX_ROOT/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally5.java
JADX_CLASSES=$JADX_ROOT/jadx-core/build/classes/java/test/jadx/tests/integration/trycatch
FIXED_CLASS=$JADX_CLASSES/TestTryCatchFinally5\$TestCls.class
TMP=$(mktemp -d /private/tmp/cf16-test5-replay.XXXXXX)
cleanup() {
    cargo clean --target-dir "$TMP/target" >/dev/null 2>&1 || true
    python3 - "$TMP" <<'PY'
import shutil, sys
shutil.rmtree(sys.argv[1], ignore_errors=True)
PY
}
trap cleanup EXIT HUP INT TERM

actual_head=$(git -C "$JADX_ROOT" rev-parse HEAD)
actual_source_sha=$(shasum -a 256 "$JADX_SOURCE" | awk '{print $1}')
actual_class_sha=$(shasum -a 256 "$FIXED_CLASS" | awk '{print $1}')
test "$actual_head" = "$JADX_HEAD"
test "$actual_source_sha" = "$EXPECTED_SOURCE_SHA"
test "$actual_class_sha" = "$EXPECTED_CLASS_SHA"
git -C "$JARDE_ROOT" diff --quiet "$EXPECTED_JARDE_BASELINE" -- crates Cargo.toml Cargo.lock
for type in A B C D; do
    case "$type" in
        A) expected_aux_sha=$EXPECTED_A_SHA ;;
        B) expected_aux_sha=$EXPECTED_B_SHA ;;
        C) expected_aux_sha=$EXPECTED_C_SHA ;;
        D) expected_aux_sha=$EXPECTED_D_SHA ;;
    esac
    aux="$JADX_CLASSES/TestTryCatchFinally5\$TestCls\$$type.class"
    test "$(shasum -a 256 "$aux" | awk '{print $1}')" = "$expected_aux_sha"
done

mkdir -p "$TMP/jadx" "$TMP/original" "$TMP/original-class/jadx/tests/integration/trycatch" "$TMP/jadx-classes" "$TMP/jarde-support"
cp "$JADX_SOURCE" "$HERE/TestTryCatchFinally5.java"
cp "$FIXED_CLASS" "$HERE/TestTryCatchFinally5\$TestCls.class"
for type in A B C D; do
    cp "$JADX_CLASSES/TestTryCatchFinally5\$TestCls\$$type.class" "$HERE/fixed-aux/TestTryCatchFinally5\$TestCls\$$type.class"
done
javap -c -v "$FIXED_CLASS" > "$HERE/bytecode/fixed-class.javap.txt"

"$JADX_BIN" -d "$TMP/jadx" \
    "$JADX_CLASSES/TestTryCatchFinally5\$TestCls.class" \
    "$JADX_CLASSES/TestTryCatchFinally5\$TestCls\$A.class" \
    "$JADX_CLASSES/TestTryCatchFinally5\$TestCls\$B.class" \
    "$JADX_CLASSES/TestTryCatchFinally5\$TestCls\$C.class" \
    "$JADX_CLASSES/TestTryCatchFinally5\$TestCls\$D.class" \
    > "$HERE/results/jadx-cli.log" 2>&1
cp "$TMP/jadx/sources/jadx/tests/integration/trycatch/TestTryCatchFinally5\$TestCls.java" \
    "$HERE/jadx/TestTryCatchFinally5\$TestCls.java"

javac --release 8 -g:none -Xlint:-options -d "$TMP/original" \
    "$HERE/original/TestTryCatchFinally5.java" "$HERE/probe/Runner.java" \
    > "$HERE/results/original.javac.stdout" 2> "$HERE/results/original.javac.stderr"
java -Xverify:all -cp "$TMP/original" jadx.tests.integration.trycatch.Runner \
    > "$HERE/results/original.run.txt" 2> "$HERE/results/original.verify.stderr"
javap -classpath "$TMP/original" -c -v 'jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls' \
    > "$HERE/bytecode/original.javap.txt"

cp "$HERE/TestTryCatchFinally5\$TestCls.class" "$TMP/original-class/jadx/tests/integration/trycatch/"
for type in A B C D; do
    cp "$HERE/fixed-aux/TestTryCatchFinally5\$TestCls\$$type.class" "$TMP/original-class/jadx/tests/integration/trycatch/"
done
javac --release 8 -g:none -Xlint:-options -d "$TMP/original-class" "$HERE/probe/Runner.java" \
    > "$HERE/results/original-class.javac.stdout" 2> "$HERE/results/original-class.javac.stderr"
java -Xverify:all -cp "$TMP/original-class" jadx.tests.integration.trycatch.Runner \
    > "$HERE/results/original-class.run.txt" 2> "$HERE/results/original-class.verify.stderr"

javac --release 8 -g:none -Xlint:-options -d "$TMP/jadx-classes" \
    "$HERE/jadx/TestTryCatchFinally5\$TestCls.java" "$HERE/probe/Runner.java" \
    > "$HERE/results/jadx.javac.stdout" 2> "$HERE/results/jadx.javac.stderr"
java -Xverify:all -cp "$TMP/jadx-classes" jadx.tests.integration.trycatch.Runner \
    > "$HERE/results/jadx.run.txt" 2> "$HERE/results/jadx.verify.stderr"
javap -classpath "$TMP/jadx-classes" -c -v 'jadx.tests.integration.trycatch.TestTryCatchFinally5$TestCls' \
    > "$HERE/bytecode/jadx-java8.javap.txt"
cmp "$HERE/results/original.run.txt" "$HERE/results/jadx.run.txt"
cmp "$HERE/results/original-class.run.txt" "$HERE/results/jadx.run.txt"
cmp "$HERE/results/original-class.run.txt" "$HERE/expected/behavior.txt"

cargo build --offline --locked --target-dir "$TMP/target" -p jarde-cli > "$HERE/results/cargo-build.log" 2>&1
"$TMP/target/debug/jarde-cli" class-source \
    --input "$HERE/TestTryCatchFinally5\$TestCls.class" --policy single-class \
    --class 'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls' \
    > "$HERE/jarde/TestTryCatchFinally5\$TestCls.java" 2> "$HERE/results/jarde-report.txt"
"$TMP/target/debug/jarde-cli" recover \
    --input "$HERE/TestTryCatchFinally5\$TestCls.class" --policy single-class \
    --class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls' \
    --method-name test \
    --descriptor '(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;' \
    > "$HERE/jarde/test.recovery.txt" 2> "$HERE/results/jarde-recover-report.txt"
"$TMP/target/debug/jarde-cli" recover \
    --input "$HERE/TestTryCatchFinally5\$TestCls.class" --policy single-class \
    --class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls' \
    --method-name test \
    --descriptor '(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;' \
    --format json --evidence all \
    > "$HERE/results/jarde-recover-all.json" 2> "$HERE/results/jarde-recover-all.stderr"

cat > "$HERE/jarde/Support.java" <<'JAVA'
package jadx.tests.integration.trycatch;
interface TestTryCatchFinally5$TestCls$A {}
interface TestTryCatchFinally5$TestCls$C {}
interface TestTryCatchFinally5$TestCls$D {
    boolean first(); boolean toNext(); void close();
}
interface TestTryCatchFinally5$TestCls$B<T> {
    TestTryCatchFinally5$TestCls$D f(TestTryCatchFinally5$TestCls$C c);
    T load(TestTryCatchFinally5$TestCls$D d);
}
JAVA
if javac --release 8 -g:none -Xlint:-options -d "$TMP/jarde-support" \
    "$HERE/jarde/Support.java" "$HERE/jarde/TestTryCatchFinally5\$TestCls.java" \
    > "$HERE/results/jarde.javac.stdout" 2> "$HERE/results/jarde.javac.stderr"; then
    printf '%s\n' 'UNEXPECTED: Jarde presentation compiled; inspect before treating it as recovery.' \
        > "$HERE/results/jarde-compile-result.txt"
    exit 1
else
    printf '%s\n' 'EXPECTED REFUSAL: Jarde explanation-only body does not compile; no Jarde behavior run was attempted.' \
        > "$HERE/results/jarde-compile-result.txt"
fi

python3 - "$HERE" <<'PY'
from pathlib import Path
import re, sys
root=Path(sys.argv[1])
def shape(path):
    text=Path(path).read_text()
    start=text.index('  public <E extends java.lang.Object> java.util.List<E> test(')
    end=text.index('      StackMapTable:', start)
    body=text[start:end]
    instructions=[]
    for line in body.splitlines():
        m=re.match(r'\s*(\d+):\s+([a-z][a-z0-9_]*)', line)
        if m: instructions.append((int(m.group(1)), m.group(2)))
    rows=[]
    for m in re.finditer(r'^\s*(\d+)\s+(\d+)\s+(\d+)\s+(any|[\w/$]+)$', body, re.M):
        rows.append(tuple(m.groups()))
    return instructions, rows
paths=[root/'bytecode/fixed-class.javap.txt',root/'bytecode/original.javap.txt',root/'bytecode/jadx-java8.javap.txt']
shapes=[shape(p) for p in paths]
offsets=[[bci for bci,_ in item[0]] for item in shapes]
without_private_call=[[(bci,op) for bci,op in item[0] if bci != 2] for item in shapes]
if not (offsets[0] == offsets[1] == offsets[2] and without_private_call[0] == without_private_call[1] == without_private_call[2] and shapes[0][1] == shapes[1][1] == shapes[2][1]):
    raise SystemExit('fixed/original/JADX method BCI and exception-row shapes differ outside the Java compiler private-call opcode')
if [item[0][2][1] for item in shapes] != ['invokevirtual','invokespecial','invokespecial']:
    raise SystemExit('unexpected opcode at BCI 2, the private p(A) call')
with (root/'results/bytecode-shape.txt').open('w') as out:
    out.write('fixed class, Java 8 original extraction, and Java 8 JADX adapter have identical BCI sequence and exception rows. All opcodes match except BCI 2, where fixed Java 8 javac uses invokevirtual for the private p(A) call and JDK 23 javac --release 8 uses invokespecial.\n')
    out.write('descriptor: (L...$A;L...$B;)Ljava/util/List;\n')
    out.write('rows: ' + ', '.join(f'[{s},{e})->{h} {t}' for s,e,h,t in shapes[0][1]) + '\n')
    out.write('normal loop back edge: 76 ifne 53; protected body loop is reachable from method entry via 44 -> 53.\n')
PY
{
    printf 'JADX HEAD %s\n' "$actual_head"
    printf 'JADX source SHA-256 %s\n' "$actual_source_sha"
    printf 'JADX fixed TestCls class SHA-256 %s\n' "$actual_class_sha"
    printf 'JADX fixed nested A/B/C/D class SHA-256 %s %s %s %s\n' "$EXPECTED_A_SHA" "$EXPECTED_B_SHA" "$EXPECTED_C_SHA" "$EXPECTED_D_SHA"
    printf 'JADX CLI version %s\n' "$("$JADX_BIN" --version)"
    printf 'Jarde tested code baseline HEAD %s\n' "$EXPECTED_JARDE_BASELINE"
    printf 'Java runtime %s\n' "$(java -version 2>&1 | head -1)"
    printf 'javac %s\n' "$(javac -version 2>&1)"
} > "$HERE/results/identities.txt"
(cd "$HERE" && shasum -a 256 \
    TestTryCatchFinally5.java 'TestTryCatchFinally5$TestCls.class' \
    fixed-aux/TestTryCatchFinally5\$TestCls\$A.class fixed-aux/TestTryCatchFinally5\$TestCls\$B.class \
    fixed-aux/TestTryCatchFinally5\$TestCls\$C.class fixed-aux/TestTryCatchFinally5\$TestCls\$D.class \
    original/TestTryCatchFinally5.java 'jadx/TestTryCatchFinally5$TestCls.java' \
    'jarde/TestTryCatchFinally5$TestCls.java' probe/Runner.java) \
    > "$HERE/results/source-class-sha256.txt"
