#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd -- "$(dirname -- "$0")" && pwd)"
REPO="$(git -C "$HERE" rev-parse --show-toplevel)"
JADX_ROOT=${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}
JADX_BIN=${JADX_BIN:-$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx}
JARDE=${1:?usage: replay.sh JARDE_CLI OUTPUT_DIR}
OUTPUT=${2:?usage: replay.sh JARDE_CLI OUTPUT_DIR}
EXPECTED_JADX=2fb1b16386941660fda07e9017285aec40fcb37f
SMALI="$JADX_ROOT/jadx-core/src/test/smali/trycatch/TestTryCatchFinally10.smali"

if [[ "$(git -C "$JADX_ROOT" rev-parse HEAD)" != "$EXPECTED_JADX" ]]; then
	echo "wrong JADX checkout; expected $EXPECTED_JADX" >&2
	exit 1
fi
if [[ ! -x "$JADX_BIN" || ! -x "$JARDE" || ! -f "$SMALI" ]]; then
	echo "pinned JADX, smali input, and Jarde CLI are required" >&2
	exit 2
fi

mkdir -p "$OUTPUT"
RUN="$(mktemp -d "${TMPDIR:-/tmp}/cf16-test10.XXXXXX")"
cleanup() {
	python3 - "$RUN" <<'PY'
from pathlib import Path
import shutil
import sys
shutil.rmtree(Path(sys.argv[1]))
PY
}
trap cleanup EXIT

ORIGINAL_CLASSES="$RUN/original-classes"
JADX_CLASSES="$RUN/jadx-classes"
JADX_SOURCE="$RUN/jadx-smali/sources/trycatch/TestTryCatchFinally10.java"
mkdir -p "$ORIGINAL_CLASSES" "$JADX_CLASSES"

mapfile -t SUPPORT_SOURCES < <(find "$HERE/original" -name '*.java' ! -name 'TestTryCatchFinally10.java' -print | sort)
javac --release 8 -g:none -Xlint:-options -d "$ORIGINAL_CLASSES" \
	"${SUPPORT_SOURCES[@]}" "$HERE/original/trycatch/TestTryCatchFinally10.java" "$HERE/probe/Runner.java"
javap -classpath "$ORIGINAL_CLASSES" -p -c -v trycatch.TestTryCatchFinally10 > "$OUTPUT/original.javap.txt"

"$JADX_BIN" --no-res --single-class trycatch.TestTryCatchFinally10 \
	-d "$RUN/jadx-smali" "$SMALI" > "$OUTPUT/jadx-cli.log" 2>&1
cp "$JADX_SOURCE" "$OUTPUT/jadx-smali.java"
python3 - "$JADX_SOURCE" "$RUN/TestTryCatchFinally10.java" <<'PY'
from pathlib import Path
import sys
source = Path(sys.argv[1]).read_text()
field = 'private static final DebugLogger l = null;'
assert source.count(field) == 1
adapter = source.replace(field, 'private static final DebugLogger l = new DebugLogger();')
assert source.split('    public static String test(', 1)[1] == adapter.split('    public static String test(', 1)[1]
Path(sys.argv[2]).write_text(adapter)
PY
javac --release 8 -g:none -Xlint:-options -d "$JADX_CLASSES" \
	"${SUPPORT_SOURCES[@]}" "$RUN/TestTryCatchFinally10.java" "$HERE/probe/Runner.java"
cp "$RUN/TestTryCatchFinally10.java" "$OUTPUT/jadx-adapter.java"

for mode in ok empty open-throw close-io close-runtime logger-runtime; do
	java -Xverify:all -cp "$ORIGINAL_CLASSES" trycatch.Runner "$mode" >> "$OUTPUT/original-run.txt"
	java -Xverify:all -cp "$JADX_CLASSES" trycatch.Runner "$mode" >> "$OUTPUT/jadx-run.txt"
done
diff -u "$OUTPUT/original-run.txt" "$OUTPUT/jadx-run.txt"

"$JARDE" class-source --input "$ORIGINAL_CLASSES/trycatch/TestTryCatchFinally10.class" \
	--class trycatch.TestTryCatchFinally10 --policy single-class --release 8 --format text \
	> "$OUTPUT/jarde-source.txt" 2> "$OUTPUT/jarde-structured-report.txt"

echo "Java 8 original and JADX adapter runs match; Jarde source/report saved under $OUTPUT"
