#!/usr/bin/env sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
ROOT=$(git -C "$HERE" rev-parse --show-toplevel)
JADX_CHECKOUT=${JADX_CHECKOUT:-/Users/lordcasser/workspace/testzone/jadx}
JADX=${JADX:-$JADX_CHECKOUT/jadx-cli/build/install/jadx/bin/jadx}
OUT=${1:?usage: replay.sh EMPTY_OUTPUT_DIRECTORY}
TARGET="${TMPDIR:-/tmp}/jarde-cf16-test7-cargo-target"
python3 -c "import shutil,sys; shutil.rmtree(sys.argv[1], ignore_errors=True)" "$TARGET"
mkdir -p "$TARGET"
trap 'python3 -c "import shutil,sys; shutil.rmtree(sys.argv[1], ignore_errors=True)" "$TARGET"' EXIT HUP INT TERM

CARGO_TARGET_DIR="$TARGET" CARGO_INCREMENTAL=0 CARGO_BUILD_JOBS=1 \
	cargo build --manifest-path "$ROOT/Cargo.toml" -p jarde-cli --locked
python3 "$HERE/replay.py" \
	--jarde "$TARGET/debug/jarde-cli" \
	--jadx "$JADX" \
	--jadx-checkout "$JADX_CHECKOUT" \
	--out "$OUT"
python3 "$HERE/probe/replay.py" \
	--jarde "$TARGET/debug/jarde-cli" \
	--jadx "$JADX" \
	--out "$OUT/probe"
