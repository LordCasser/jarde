#!/bin/sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(git -C "$HERE" rev-parse --show-toplevel)
JADX_ROOT=${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}
JADX_BIN=${JADX_BIN:-$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx}
SOURCE="$JADX_ROOT/jadx-core/src/test/java/jadx/tests/integration/trycatch/TestTryCatchFinally2.java"
FIXED_CLASS="$JADX_ROOT/jadx-core/build/classes/java/test/jadx/tests/integration/trycatch/TestTryCatchFinally2\$TestCls.class"
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
EXPECTED_SOURCE_SHA=d91f17350a382bd351efeb5ec21eda27fb208ab32fcb3d765c03de8426be8831
EXPECTED_CLASS_SHA=330ddd26a3bb313fde9be032e78c1be83e7b6acb2c1b86e53abc1ce604b89170
TMP=$(mktemp -d /tmp/cf16-test2-replay.XXXXXX)
cleanup() {
	cargo clean --target-dir "$TMP/target" >/dev/null 2>&1 || true
	python3 - "$TMP" <<'PY'
import shutil, sys
shutil.rmtree(sys.argv[1], ignore_errors=True)
PY
}
trap cleanup EXIT HUP INT TERM

test "$(git -C "$JADX_ROOT" rev-parse HEAD)" = "$EXPECTED_JADX_HEAD"
test "$(shasum -a 256 "$SOURCE" | awk '{print $1}')" = "$EXPECTED_SOURCE_SHA"
test "$(shasum -a 256 "$FIXED_CLASS" | awk '{print $1}')" = "$EXPECTED_CLASS_SHA"
test -x "$JADX_BIN"
test "$(shasum -a 256 "$HERE/fixed/TestTryCatchFinally2\$TestCls.class" | awk '{print $1}')" = "$EXPECTED_CLASS_SHA"

# The upstream test has no profile annotation and defaults to the DX input plugin.
TEST_INPUT_PLUGIN=dx "$JADX_ROOT/gradlew" -p "$JADX_ROOT" :jadx-core:test \
	--tests jadx.tests.integration.trycatch.TestTryCatchFinally2 --rerun-tasks \
	> "$HERE/results/jadx-test-dx.log" 2>&1

mkdir -p "$TMP/fixed/jadx/tests/integration/trycatch" "$TMP/original" "$TMP/jadx"
cp "$HERE/fixed/TestTryCatchFinally2\$TestCls.class" "$TMP/fixed/jadx/tests/integration/trycatch/"
javap -p -c -v "$HERE/fixed/TestTryCatchFinally2\$TestCls.class" > "$HERE/bytecode/fixed.javap.txt"
mkdir -p "$TMP/support/jadx/core/clsp" "$TMP/support/jadx/core/dex/instructions/args"
cp "$HERE/support/jadx/core/clsp/ClspClass.java" "$TMP/support/jadx/core/clsp/"
cp "$HERE/support/jadx/core/dex/instructions/args/ArgType.java" "$TMP/support/jadx/core/dex/instructions/args/"

"$JADX_BIN" --no-res -d "$TMP/jadx-output" "$FIXED_CLASS" > "$HERE/results/jadx-cli.log" 2>&1
cp "$TMP/jadx-output/sources/jadx/tests/integration/trycatch/TestTryCatchFinally2\$TestCls.java" "$HERE/jadx/"

cargo build --offline --locked --target-dir "$TMP/target" -p jarde-cli > "$HERE/results/cargo-build.log" 2>&1
JARDE="$TMP/target/debug/jarde-cli"
CLASS="$HERE/fixed/TestTryCatchFinally2\$TestCls.class"
"$JARDE" class-source --input "$CLASS" --policy single-class \
	--class 'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls' \
	> "$HERE/jarde/TestTryCatchFinally2\$TestCls.java" 2> "$HERE/results/jarde-class-source.txt"
"$JARDE" recover --input "$CLASS" --policy single-class \
	--class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls' \
	--method-name test --descriptor '(Ljava/io/OutputStream;)V' --format json --evidence all \
	> "$HERE/results/jarde-test-evidence.json" 2> "$HERE/results/jarde-recover-report.txt"

JAVAC=${JAVAC:-$(command -v javac)}
JAVA=${JAVA:-$(command -v java)}
JAVAP=${JAVAP:-$(command -v javap)}
for variant in fixed original jadx; do
	case "$variant" in
		fixed) classes="$TMP/fixed"; source="$HERE/probe/Runner.java"; javapout=fixed-loadable ;;
		original) classes="$TMP/original"; source="$HERE/original/TestTryCatchFinally2\$TestCls.java"; javapout=original ;;
		jadx) classes="$TMP/jadx"; source="$HERE/jadx/TestTryCatchFinally2\$TestCls.java"; javapout=jadx ;;
	esac
	"$JAVAC" --release 8 -g -Xlint:-options -classpath "$classes" -d "$classes" \
		"$TMP/support/jadx/core/clsp/ClspClass.java" \
		"$TMP/support/jadx/core/dex/instructions/args/ArgType.java" \
		"$HERE/probe/Runner.java" $([ "$variant" = fixed ] || printf '%s' "$source") \
		> "$HERE/results/$variant.javac.stdout" 2> "$HERE/results/$variant.javac.stderr"
	"$JAVA" -Xverify:all -cp "$classes" jadx.tests.integration.trycatch.Runner \
		> "$HERE/results/$variant.behavior.txt" 2> "$HERE/results/$variant.verify.stderr"
	"$JAVAP" -classpath "$classes" -p -c -v 'jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls' \
		> "$HERE/bytecode/$javapout.javap.txt"
done
python3 "$HERE/check-bytecode-shape.py" \
	"$HERE/bytecode/fixed-loadable.javap.txt" "$HERE/bytecode/original.javap.txt" "$HERE/bytecode/jadx.javap.txt" \
	> "$HERE/results/bytecode-shape.txt"
cmp "$HERE/results/fixed.behavior.txt" "$HERE/results/original.behavior.txt"
cmp "$HERE/results/fixed.behavior.txt" "$HERE/results/jadx.behavior.txt"

printf 'JADX HEAD %s\nJADX source SHA-256 %s\nJADX fixed class SHA-256 %s\nJADX CLI SHA-256 %s\nJADX CLI version %s\n' \
	"$EXPECTED_JADX_HEAD" "$EXPECTED_SOURCE_SHA" "$EXPECTED_CLASS_SHA" \
	"$(shasum -a 256 "$JADX_BIN" | awk '{print $1}')" "$($JADX_BIN --version)" > "$HERE/results/identities.txt"
printf 'Jarde source revision %s\nJava %s\njavac %s\n' \
	"$(git -C "$ROOT" rev-parse HEAD)" "$($JAVA -version 2>&1 | head -1)" "$($JAVAC -version 2>&1)" >> "$HERE/results/identities.txt"

(cd "$HERE" && shasum -a 256 \
	'fixed/TestTryCatchFinally2$TestCls.class' fixed/TestTryCatchFinally2.java \
	'original/TestTryCatchFinally2$TestCls.java' 'jadx/TestTryCatchFinally2$TestCls.java' \
	'jarde/TestTryCatchFinally2$TestCls.java' probe/Runner.java support/jadx/core/clsp/ClspClass.java \
	support/jadx/core/dex/instructions/args/ArgType.java) > "$HERE/results/SHA256SUMS"
