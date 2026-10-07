#!/bin/sh
# The gating experiment of tasks 1.2/1.3: which anchor moves when the declaration plan's
# fallback classification is relaxed, and which control does not.
#
# The two recorded patches under `patches/` are applied to `crates/jarde-java/src/build.rs` in
# order and the same render set is taken after each: a patch is applied, `cargo build -p jarde-cli`
# is run, every input below is rendered with the same CLI invocation the patrols used, and the
# tree is restored with `git checkout`. Nothing here is a change to the repository: the patches
# exist to measure which layer answers first.
#
#   patches/01-classification-only.patch
#       `has_unpresented_access` is skipped: the planner no longer answers "incomplete" for a
#       local whose uses include a quoted fallback region.
#   patches/02-classification-plus-declaration-region.patch
#       and `declaration_region` no longer vetoes a fallback in the local's own use set either, so
#       the planner has no fallback-based refusal left.
#
# Usage: sh 02-gating.sh [state]        (state: a, b, both; default both)
# The renders land in `gating/<state>/<label>.txt`, beside the ones this file's own run recorded.
set -eu

ROOT=$(cd "$(dirname "$0")/../../../../.." && pwd)
R=$ROOT/openspec/changes/preserve-local-scope-across-exception-regions/results/planning-1.2-1.3
CLI=$ROOT/target/debug/jarde-cli
LOCK=$ROOT/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar
IO=$ROOT/openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar
FIX=$ROOT/tests/fixtures/preserve-local-scope-plan

# The render set: the two patrol anchors, the fixture's own crossing class (the same two shapes
# with the planner's own sentence), and the two positive controls the classification's first two
# answers already cover.
render_set() {
    state=$1
    mkdir -p "$R/gating/$state"
    "$CLI" class-source --policy plain-jar --input "$LOCK" --class LK --format text \
        >"$R/gating/$state/LK.txt" 2>"$R/gating/$state/LK.err"
    "$CLI" class-source --policy plain-jar --input "$IO" --class IO --format text \
        >"$R/gating/$state/IO.txt" 2>"$R/gating/$state/IO.err"
    "$CLI" class-source --policy single-class --input "$FIX/v8/ScopePlanCrossing.class" \
        --class ScopePlanCrossing --format text >"$R/gating/$state/ScopePlanCrossing.txt" \
        2>"$R/gating/$state/ScopePlanCrossing.err"
    "$CLI" class-source --policy single-class --input "$FIX/v8/ScopePlan.class" \
        --class ScopePlan --format text >"$R/gating/$state/ScopePlan.txt" \
        2>"$R/gating/$state/ScopePlan.err"
    "$CLI" class-source --policy single-class \
        --input "$ROOT/tests/fixtures/p3-exception-scope/v8/ExceptionScope.class" \
        --class ExceptionScope --format text >"$R/gating/$state/ExceptionScope.txt" \
        2>"$R/gating/$state/ExceptionScope.err"
    for file in "$R/gating/$state"/*.txt; do
        head -1 "$file" | grep -q '^// jarde: presentation of' ||
            { echo "SELF-HEADER MISSING in $file (not a render)"; exit 1; }
    done
}

build() {
    (cd "$ROOT" && cargo build --locked -p jarde-cli >/dev/null)
}

restore() {
    (cd "$ROOT" && git checkout -- crates/jarde-java/src/build.rs)
}

case ${1:-both} in
a) states="a" ;;
b) states="b" ;;
*) states="a b" ;;
esac
for state in $states; do
    restore
    case $state in
    a) (cd "$ROOT" && git apply "$R/patches/01-classification-only.patch") ;;
    b) (cd "$ROOT" && git apply "$R/patches/02-classification-plus-declaration-region.patch") ;;
    esac
    build
    render_set "$state"
    restore
done
build
echo "gating renders written under $R/gating"
