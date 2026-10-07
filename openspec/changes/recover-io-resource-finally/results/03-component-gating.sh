#!/bin/sh
# The slice's per-component gating: each component is reverted alone (and in combination) and the
# same probe set is rendered, so every claim about "which component flips what" is measured rather
# than argued.
#
# The four configurations:
#
#   full        the row-set certificate + the two `java.io` widening rows + the `new@1` depth (3)
#   cert-only   the certificate alone (the rows and the depth reverted)
#   cert+rows   the certificate + the widening rows (the depth reverted)
#   cert+depth  the certificate + the depth (the rows reverted)
#
# The probe set (each entry names the predicate its presentation is read by):
#
#   io          `IO.countLines` presents its `finally` (needs the certificate, the rows and the depth)
#   mid         `IOMidRead.countRemaining` presents its `finally` (needs the certificate alone)
#   widening    `WideningProbe` presents both argument casts (needs the two rows)
#   depth       `NestedDepth.threeLayer` presents one `new` expression (needs the depth)
#   lk          `LK` is byte-identical to the baseline in every configuration (the invariant)
#   void-loop   the CF-16 fixed fixture is byte-identical to the baseline (the certificate must not
#               claim the fixed certificates' own shapes)
#
# Usage: 03-component-gating.sh [baseline-cli] [worktree]
#   defaults: /tmp/io-baseline-target/debug/jarde-cli and this script's worktree.
#
# Output: `03-component-gating.out` beside this script. The script restores every edited file; a
# run interrupted after an edit leaves `*.component-gating.bak` files behind to restore by hand.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BASELINE=${1:-/tmp/io-baseline-target/debug/jarde-cli}
WORKTREE=${2:-$ROOT}
OUT="$HERE/03-component-gating.out"
BUILD_RS="$WORKTREE/crates/jarde-java/src/build.rs"
INIT_RS="$WORKTREE/crates/jarde-java/src/init.rs"

[ -x "$BASELINE" ] || { echo "no baseline binary at $BASELINE" >&2; exit 2; }
[ -f "$BUILD_RS" ] || { echo "no build.rs at $BUILD_RS" >&2; exit 2; }
[ -f "$INIT_RS" ] || { echo "no init.rs at $INIT_RS" >&2; exit 2; }

cp "$BUILD_RS" "$BUILD_RS.component-gating.bak"
cp "$INIT_RS" "$INIT_RS.component-gating.bak"
restore() {
    cp "$BUILD_RS.component-gating.bak" "$BUILD_RS"
    cp "$INIT_RS.component-gating.bak" "$INIT_RS"
    rm -f "$BUILD_RS.component-gating.bak" "$INIT_RS.component-gating.bak"
}
trap restore EXIT INT TERM

# $1 = on | off — the two `java.io` rows of `platform_reference_argument_widens`, idempotently.
set_rows() {
    python3 "$HERE/03-set-rows.py" "$BUILD_RS" "$1"
}

# $1 = 2 | 3 — `MAX_NESTED_CONSTRUCTION_LAYERS`.
set_depth() {
    sed -i '' "s/const MAX_NESTED_CONSTRUCTION_LAYERS: u32 = [0-9];/const MAX_NESTED_CONSTRUCTION_LAYERS: u32 = $1;/" "$INIT_RS"
}

render() {
    "$1" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null
}

IO="$ROOT/openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar"
MID="$ROOT/tests/fixtures/recover-io-resource-finally/v8/IOMidRead.class"
WIDENING="$ROOT/tests/fixtures/recover-io-resource-finally/v8/WideningProbe.class"
DEPTH="$ROOT/tests/fixtures/recover-io-resource-finally/v8/NestedDepth.class"
LK="$ROOT/tests/fixtures/recover-lock-guard-loop-finally/v8/LK.class"
VOID="$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally/fixed/TestTryCatchFinally2\$TestCls.class"

baseline_lk=$(render "$BASELINE" "$LK" single-class LK | shasum -a 256 | cut -d' ' -f1)
baseline_void=$(render "$BASELINE" "$VOID" single-class 'jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls' | shasum -a 256 | cut -d' ' -f1)

{
    echo "# The per-component gating (baseline = the parent commit c99e26a5)"
    echo "# columns: configuration | io | mid | widening | depth | lk | void-loop"
    echo "configuration|io|mid|widening|depth|lk|void-loop"
} > "$OUT"

for configuration in full cert-only cert+rows cert+depth
do
    case "$configuration" in
        full) set_rows on; set_depth 3 ;;
        cert-only) set_rows off; set_depth 2 ;;
        cert+rows) set_rows on; set_depth 2 ;;
        cert+depth) set_rows off; set_depth 3 ;;
    esac
    ( cd "$WORKTREE" && cargo build -p jarde-cli ) > /dev/null 2>&1
    CLI="$WORKTREE/target/debug/jarde-cli"
    io=$(render "$CLI" "$IO" plain-jar IO | grep -c "finally {" || true)
    mid=$(render "$CLI" "$MID" single-class IOMidRead | grep -c "finally {" || true)
    widening_first=$(render "$CLI" "$WIDENING" single-class WideningProbe | grep -c "java.io.InputStreamReader((java.io.InputStream)" || true)
    widening_second=$(render "$CLI" "$WIDENING" single-class WideningProbe | grep -c "BufferedReader((java.io.Reader)" || true)
    widening="$widening_first/$widening_second"
    depth=$(render "$CLI" "$DEPTH" single-class NestedDepth | grep -c "new NestedDepth\$First" || true)
    lk=$(render "$CLI" "$LK" single-class LK | shasum -a 256 | cut -d' ' -f1)
    void=$(render "$CLI" "$VOID" single-class 'jadx.tests.integration.trycatch.TestTryCatchFinally2$TestCls' | shasum -a 256 | cut -d' ' -f1)
    [ "$lk" = "$baseline_lk" ] && lk=identical || lk=moved
    [ "$void" = "$baseline_void" ] && void=identical || void=moved
    printf '%s|%s|%s|%s|%s|%s|%s\n' "$configuration" "$io" "$mid" "$widening" "$depth" "$lk" "$void" >> "$OUT"
done
restore
trap - EXIT INT TERM
( cd "$WORKTREE" && cargo build -p jarde-cli ) > /dev/null 2>&1
cat "$OUT"
