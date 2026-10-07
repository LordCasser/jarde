#!/bin/sh
# The slice's gating experiment: each admission alone, and both together, against the baseline.
#
# Claim this measures:
#   * A (the multi-statement finally body) alone flips ML's `nestedLocks` and nothing else;
#   * B (the throwing acquisition outside the rows) alone flips ML's `interruptibly` and nothing
#     else;
#   * A+B flips both;
#   * under **every** configuration the lock-guard and resource-guard anchors (LK/IO, their
#     negatives and their probes), the other committed finally shapes, and this change's own
#     negatives and boundaries render **byte-identically** to the baseline.
#
# The two single-admission builds are the shipped code with one temporary line each: A-only keeps
# the one-group copy's own block-start requirement, B-only refuses any copy longer than one group.
# Neither line survives the experiment (the reference copy of `guard.rs` is restored after every
# build).
#
# Usage: 03-gating.sh [worktree]
#
# Self-test before counting: every render must start with the layer's own header, or the script
# stops. Output: 03-gating.out (the table) beside this script's directory.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "${1:-$HERE/../../../..}" && pwd)
GUARD="$ROOT/crates/jarde-java/src/guard.rs"
BASELINE=${BASELINE:-/tmp/gate/baseline}
OUT="$HERE/03-gating.out"
RENDERS=${RENDERS:-/tmp/gate/renders}
RENDER="$HERE/02-render.sh"
REF=/tmp/gate/guard-reference.rs

[ -x "$BASELINE" ] || { echo "no baseline binary at $BASELINE" >&2; exit 2; }
[ -f "$GUARD" ] || { echo "no guard.rs at $GUARD" >&2; exit 2; }

mkdir -p /tmp/gate
cp "$GUARD" "$REF"

build() {
    # $1 = gate line ("" for the shipped build), $2 = destination binary
    cp "$REF" "$GUARD"
    if [ -n "$1" ]; then
        python3 - "$GUARD" "$1" <<'PY'
import sys

path, gate = sys.argv[1], sys.argv[2]
text = open(path).read()
marker = "    // The two copies release the same targets on the same values, group by group.\n"
if text.count(marker) != 1:
    raise SystemExit("the insertion marker is not unique")
text = text.replace(marker, gate + "\n" + marker)
open(path, "w").write(text)
PY
    fi
    ( cd "$ROOT" && cargo build -p jarde-cli --locked >/dev/null 2>&1 )
    cp "$ROOT/target/debug/jarde-cli" "$2"
    cp "$REF" "$GUARD"
}

build "" /tmp/gate/ab
build "    if normal_copies.len() == 1 && row.start_bci != current.bci() {
        return Ok(None);
    }" /tmp/gate/a-only
build "    if normal_copies.len() > 1 {
        return Ok(None);
    }" /tmp/gate/b-only

for configuration in base ab a-only b-only; do
    binary=/tmp/gate/$configuration
    [ "$configuration" = base ] && binary=$BASELINE
    mkdir -p "$RENDERS/$configuration"
    sh "$RENDER" "$binary" "$RENDERS/$configuration" >/dev/null
done

{
    echo "# The gating experiment: baseline vs each single admission vs both"
    echo "# columns: input | base | A-only | B-only | A+B | verdict"
    echo "input|base|A-only|B-only|A+B|verdict"
} > "$OUT"
for file in "$RENDERS/base"/*.java; do
    name=$(basename "$file" .java)
    base=$(shasum -a 256 "$RENDERS/base/$name.java" | cut -d' ' -f1)
    a_only=$(shasum -a 256 "$RENDERS/a-only/$name.java" | cut -d' ' -f1)
    b_only=$(shasum -a 256 "$RENDERS/b-only/$name.java" | cut -d' ' -f1)
    ab=$(shasum -a 256 "$RENDERS/ab/$name.java" | cut -d' ' -f1)
    verdict="all-identical"
    [ "$a_only" = "$base" ] || verdict="A-moved"
    [ "$b_only" = "$base" ] || verdict="$verdict,B-moved"
    [ "$ab" = "$base" ] || verdict="$verdict,AB-moved"
    printf '%s|%s|%s|%s|%s|%s\n' "$name" "$base" "$a_only" "$b_only" "$ab" "$verdict" >> "$OUT"
done
cat "$OUT"
