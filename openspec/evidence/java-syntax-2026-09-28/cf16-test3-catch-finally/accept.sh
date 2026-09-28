#!/bin/sh
set -eu

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
PROJECT=$(git -C "$ROOT" rev-parse --show-toplevel)
WORK=$(mktemp -d /private/tmp/cf16-test3-accept.XXXXXX)
trap 'rm -rf "$WORK"' EXIT HUP INT TERM
if [ -z "${JARDE_CLI:-}" ]; then
    CARGO_TARGET_DIR="$WORK/cargo" cargo build --locked -p jarde-cli --manifest-path "$PROJECT/Cargo.toml" \
        > "$ROOT/results/cargo-build.log" 2>&1
    JARDE_CLI="$WORK/cargo/debug/jarde-cli"
fi

CLASS="$ROOT/TestTryCatchFinally3\$TestCls.class"
SOURCE="$ROOT/jarde-out/TestTryCatchFinally3\$TestCls.java"
"$JARDE_CLI" class-source --input "$CLASS" --policy single-class \
    --class 'jadx/tests/integration/trycatch/TestTryCatchFinally3$TestCls' \
    > "$SOURCE" 2> "$ROOT/results/jarde-report.txt"
"$JARDE_CLI" recover --input "$CLASS" --policy single-class \
    --class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally3$TestCls' \
    --method-name test --descriptor '(Ljadx/core/dex/nodes/ClassNode;Ljava/util/List;)V' \
    > "$ROOT/jarde-out/test.recovery.txt" 2> "$ROOT/results/jarde-recover-report.txt"

"$ROOT/run.sh"
python3 "$ROOT/neighbors.py"

python3 - "$SOURCE" "$ROOT/jarde-method-harness/TestTryCatchFinally3\$TestCls.java" <<'PY'
import pathlib
import sys

source = pathlib.Path(sys.argv[1]).read_text()
start = source.index("    public static void test(")
opening = source.index("{", start)
depth = 0
for at in range(opening, len(source)):
    if source[at] == "{":
        depth += 1
    elif source[at] == "}":
        depth -= 1
        if depth == 0:
            method = source[start:at + 1]
            break
else:
    raise AssertionError("no complete Jarde test method")
assert "@bytecode" not in method
assert method.count("finally {") == 1
assert method.count("unload();") == 1
output = pathlib.Path(sys.argv[2])
output.parent.mkdir(exist_ok=True)
output.write_text(
    "// Method-level harness: declaration and LOG initializer are supplied from the original class.\n"
    "// The test method below is extracted unchanged from fresh Jarde class-source output.\n"
    "package jadx.tests.integration.trycatch;\n"
    "public class TestTryCatchFinally3$TestCls {\n"
    "    private static final org.slf4j.Logger LOG = org.slf4j.LoggerFactory.getLogger(TestTryCatchFinally3$TestCls.class);\n"
    + method + "\n}\n"
)
PY

mkdir -p "$WORK/harness-classes"
(
    cd "$ROOT"
    find standins -name '*.java' -print > "$WORK/harness-sources.list"
    find runners -name '*.java' -print >> "$WORK/harness-sources.list"
    printf '%s\n' 'jarde-method-harness/TestTryCatchFinally3$TestCls.java' >> "$WORK/harness-sources.list"
    javac --release 8 -g:none -Xlint:-options -d "$WORK/harness-classes" @"$WORK/harness-sources.list" \
        > "$ROOT/results/jarde-method-javac.stdout" 2> "$ROOT/results/jarde-method-javac.stderr"
)
java -Xverify:all -cp "$WORK/harness-classes" jadx.tests.integration.trycatch.Runner \
    > "$ROOT/results/jarde-method-run.txt" 2> "$ROOT/results/jarde-method-verify.stderr"
cmp "$ROOT/results/original-class-run.txt" "$ROOT/results/original-run.txt"
cmp "$ROOT/results/original-class-run.txt" "$ROOT/results/jadx-run.txt"
cmp "$ROOT/results/original-class-run.txt" "$ROOT/results/jarde-method-run.txt"

{
    printf 'Jarde source: fresh CLI from this worktree\n'
    printf 'Target class SHA-256 %s\n' "$(shasum -a 256 "$CLASS" | awk '{print $1}')"
    printf 'Method harness: Jarde test with original LOG declaration/initializer, Java 8 verified\n'
    printf 'Whole Jarde class-source: independent <clinit> fallback, javac failure expected\n'
} > "$ROOT/results/acceptance.txt"

(
    cd "$ROOT"
    shasum -a 256 \
        'TestTryCatchFinally3.java' \
        'TestTryCatchFinally3$TestCls.class' \
        'jadx-src/TestTryCatchFinally3$TestCls.java' \
        'jarde-out/TestTryCatchFinally3$TestCls.java' \
        'jarde-method-harness/TestTryCatchFinally3$TestCls.java' \
        > results/source-and-class-sha256.txt
)
