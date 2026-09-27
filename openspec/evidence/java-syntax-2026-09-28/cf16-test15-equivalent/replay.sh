#!/usr/bin/env bash
set -euo pipefail

HERE="$(cd -- "$(dirname -- "$0")" && pwd)"
REPO=${JARDE_REPO:-$(git -C "$HERE" rev-parse --show-toplevel)}
JADX_ROOT=${JADX_ROOT:-/Users/lordcasser/workspace/testzone/jadx}
JADX_BIN=${JADX_BIN:-$JADX_ROOT/jadx-cli/build/install/jadx/bin/jadx}
EXPECTED_JADX=2fb1b16386941660fda07e9017285aec40fcb37f
SMALI="$JADX_ROOT/jadx-core/src/test/smali/trycatch/TestTryCatchFinally15.smali"
EXPECTED_SMALI_SHA=438b7e88987722ae4e015fb7eb475a291f8384ed37ca4f39bfcc1de42b3833b6

if [[ ! -x "$JADX_BIN" || ! -f "$SMALI" ]]; then
	echo "pinned JADX checkout and installed CLI are required" >&2
	exit 2
fi
actual_jadx=$(git -C "$JADX_ROOT" rev-parse HEAD)
if [[ "$actual_jadx" != "$EXPECTED_JADX" ]]; then
	echo "wrong JADX checkout: expected $EXPECTED_JADX, got $actual_jadx" >&2
	exit 1
fi

RUN=$(mktemp -d /private/tmp/jarde-cf16-test15-replay.XXXXXX)
CARGO_TARGET_DIR="$RUN/cargo-target"
cleanup() {
	CARGO_TARGET_DIR="$CARGO_TARGET_DIR" CARGO_INCREMENTAL=0 cargo clean --manifest-path "$REPO/Cargo.toml" >/dev/null || true
	if [[ ${KEEP_OUTPUTS:-0} == 1 ]]; then
		printf 'kept replay outputs: %s\n' "$RUN"
	else
		python3 - "$RUN" <<'PY'
import shutil
import sys
shutil.rmtree(sys.argv[1])
PY
	fi
}
trap cleanup EXIT

mkdir -p "$RUN/original-classes" "$RUN/jadx-adapter" "$RUN/jadx-classes" "$RUN/jadx-smali" "$RUN/jarde"
STUBS=(
	"$HERE/original/android/os/IBinder.java"
	"$HERE/original/android/os/IInterface.java"
	"$HERE/original/android/os/Parcel.java"
	"$HERE/original/android/os/RemoteException.java"
)
javac --release 8 -g:none -Xlint:-options -d "$RUN/original-classes" \
	"${STUBS[@]}" "$HERE/original/TestTryCatchFinally15.java" "$HERE/probe/Runner.java"

ORIGINAL_CLASS="$RUN/original-classes/trycatch/TestTryCatchFinally15.class"
ORIGINAL_SHA=$(shasum -a 256 "$ORIGINAL_CLASS" | awk '{print $1}')
EXPECTED_ORIGINAL_SHA=880a6934faa48c317538cc37e4dc72c4d1130d8b50b4a78c0656a583565b4ba3
if [[ "$ORIGINAL_SHA" != "$EXPECTED_ORIGINAL_SHA" ]]; then
	echo "Java 8 sample class hash changed: $ORIGINAL_SHA" >&2
	exit 1
fi

javap -classpath "$RUN/original-classes" -p -c -v trycatch.TestTryCatchFinally15 > "$RUN/original.javap.txt"
python3 - "$RUN/original.javap.txt" <<'PY'
from pathlib import Path
import sys
text = Path(sys.argv[1]).read_text()
method = text.split('  protected final android.os.Parcel test(', 1)[1]
table = method.split('      Exception table:', 1)[1].split('      StackMapTable:', 1)[0]
rows = [' '.join(line.split()) for line in table.splitlines() if line.strip()]
expected = [
    'from to target type',
    '4 25 32 Class java/lang/RuntimeException',
    '4 25 41 any',
    '32 43 41 any',
]
assert rows == expected, f'unexpected JVM exception table: {rows!r}'
PY

"$JADX_BIN" --no-res --single-class trycatch.TestTryCatchFinally15 \
	-d "$RUN/jadx-smali" "$SMALI" > "$RUN/jadx-smali.log" 2>&1
SMALI_SHA=$(shasum -a 256 "$SMALI" | awk '{print $1}')
[[ "$SMALI_SHA" == "$EXPECTED_SMALI_SHA" ]] || {
	echo "pinned Test15 smali changed: $SMALI_SHA" >&2
	exit 1
}
SMALI_SOURCE="$RUN/jadx-smali/sources/trycatch/TestTryCatchFinally15.java"
SMALI_SOURCE_SHA=$(python3 - "$SMALI_SOURCE" <<'PY'
from pathlib import Path
import hashlib
import re
import sys
source = Path(sys.argv[1]).read_text()
source = re.sub(r'(loaded from: ).*?TestTryCatchFinally15\.smali', r'\1<Test15.smali>', source, count=1)
print(hashlib.sha256(source.encode()).hexdigest())
PY
)
EXPECTED_SMALI_SOURCE_SHA=c26d853656dfd81f54bb8da716aec85805206b414833191aea054659ed712e27
[[ "$SMALI_SOURCE_SHA" == "$EXPECTED_SMALI_SOURCE_SHA" ]] || {
	echo "pinned JADX smali source changed: $SMALI_SOURCE_SHA" >&2
	exit 1
}
python3 - "$SMALI_SOURCE" "$RUN/jadx-adapter/TestTryCatchFinally15.java" <<'PY'
from pathlib import Path
import sys
source = Path(sys.argv[1]).read_text()
assert source.count('parcel.recycle();') == 2
assert 'catch (Throwable th)' in source
assert 'this.zza.transact(i, parcel, parcelObtain, 0);' in source
adapter = source.replace('private final IBinder zza;', 'private IBinder zza;')
adapter = adapter.replace('private final String zzb;', 'private String zzb;')
needle = '    protected final Parcel test('
assert adapter.count(needle) == 1
constructor = '    public TestTryCatchFinally15(IBinder zza) {\n        this.zza = zza;\n    }\n\n'
adapter = adapter.replace(needle, constructor + needle)
assert source.split(needle, 1)[1] == adapter.split(needle, 1)[1], 'the target test() text changed in the adapter'
Path(sys.argv[2]).write_text(adapter)
PY

javac --release 8 -g:none -Xlint:-options -d "$RUN/jadx-classes" \
	"${STUBS[@]}" "$RUN/jadx-adapter/TestTryCatchFinally15.java" "$HERE/probe/Runner.java"
for mode in ok runtime remote read cleanup; do
	printf '%s=' "$mode" >> "$RUN/original.run.txt"
	java -Xverify:all -cp "$RUN/original-classes" trycatch.Runner "$mode" >> "$RUN/original.run.txt"
	printf '%s=' "$mode" >> "$RUN/jadx.run.txt"
	java -Xverify:all -cp "$RUN/jadx-classes" trycatch.Runner "$mode" >> "$RUN/jadx.run.txt"
done
cmp "$RUN/original.run.txt" "$HERE/expected/original.txt"
cmp "$RUN/jadx.run.txt" "$HERE/expected/jadx.txt"

CARGO_TARGET_DIR="$CARGO_TARGET_DIR" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 \
	cargo build --locked --manifest-path "$REPO/Cargo.toml" -p jarde-cli > "$RUN/cargo-build.log" 2>&1
CLI="$CARGO_TARGET_DIR/debug/jarde"
[[ -x "$CLI" ]] || CLI="$CARGO_TARGET_DIR/debug/jarde-cli"
[[ -x "$CLI" ]]
"$CLI" class-source --input "$ORIGINAL_CLASS" --class trycatch/TestTryCatchFinally15 \
	--policy single-class --release 8 --format text > "$RUN/jarde/TestTryCatchFinally15.java" \
	2> "$RUN/jarde/report.txt"
rg -q 'explanation only' "$RUN/jarde/TestTryCatchFinally15.java"
rg -q 'local 3 crosses a quoted fallback region' "$RUN/jarde/TestTryCatchFinally15.java"
if rg -q '(^|[[:space:]])try[[:space:]]*\{' "$RUN/jarde/TestTryCatchFinally15.java"; then
	echo 'Jarde unexpectedly emitted a Java try body for the refused method' >&2
	exit 1
fi

printf 'jadx_head=%s\n' "$actual_jadx"
printf 'smali_sha256=%s\n' "$SMALI_SHA"
printf 'original_class_sha256=%s\n' "$ORIGINAL_SHA"
printf 'jadx_smali_source_sha256=%s\n' "$SMALI_SOURCE_SHA"
printf 'original_behavior=matched expected five paths\n'
printf 'jadx_behavior=matched expected four paths; cleanup self-throw repeats recycle\n'
printf 'jarde_behavior=whole target method refused with explanation-only\n'
