#!/bin/sh
# The change's corpus render differential: every class file under `openspec/evidence` and
# `tests/fixtures` — loose files and every `.class` entry of every committed jar — is rendered by
# two binaries (the baseline, built from this change's parent commit `5a9f32bf`, and the patched
# one) through the same entry point, and the two texts are diffed. A class whose render moves is
# printed with the refusals it gained or lost.
#
# The count is only trusted after the script's own self-test: the known positive (the patrol's
# `CB.loopElseIfRet` loses its one canonical-overlap refusal on the patched binary and keeps it on
# the baseline) and the known negative (this change's `LB` keeps both canonical-overlap refusals and
# both cross-quote refusals on **both** binaries) must both hold, or the script exits non-zero.
# Every render's first line must carry jarde's own self-header — a render of nothing is not a
# render, and counting refusals in one would be a false zero. A loose class whose file name is not
# its own internal name is resolved against the class file itself; one that renders under no name
# is reported, never dropped silently.
set -eu

BASE=${BASE:-/tmp/ladder/base-wt/target/debug/jarde-cli}
PATCHED=${PATCHED:-target/debug/jarde-cli}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
WORK=${1:-/tmp/ladder/corpus}
rm -rf "$WORK"
mkdir -p "$WORK"

# One render that answers whether it was one: the output carries jarde's own self-header.
try_render() {
    binary=$1
    policy=$2
    path=$3
    name=$4
    out=$5
    "$binary" class-source --policy "$policy" --input "$path" --class "$name" \
        --format text >"$out" 2>"$out.err" || true
    head -1 "$out" | grep -q '// jarde: presentation of'
}

# One loose class file, under the name it declares: the file's own name first, then every suffix of
# its path (shortest first), because a committed fixture's file name is not always its class name.
render_loose() {
    binary=$1
    path=$2
    out=$3
    stem=${path%.class}
    name=$(basename "$stem")
    if try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    candidates=$stem
    while [ "${stem#*/}" != "$stem" ]; do
        stem=${stem#*/}
        candidates="$stem
$candidates"
    done
    printf '%s\n' "$candidates" | while IFS= read -r candidate; do
        if try_render "$binary" single-class "$path" "$candidate" "$out"; then
            echo "$candidate"
            break
        fi
    done >"$out.name"
    if [ -s "$out.name" ]; then
        return 0
    fi
    # A file whose name is neither its class name nor a suffix of its path: the class states its
    # own name, and `javap` reads it off the file.
    name=$(javap -p -v "$path" 2>/dev/null \
        | grep -m1 -oE ' (class|interface|enum) [^ <]+' | awk '{print $2}')
    if [ -n "$name" ] && try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    echo "UNRENDERED: $path renders under no name it states"
    return 1
}

render_jar() {
    binary=$1
    jar=$2
    entry=$3
    out=$4
    name=${entry%.class}
    if ! try_render "$binary" plain-jar "$jar" "$name" "$out"; then
        echo "UNRENDERED: $jar!$entry renders under its own entry name"
        return 1
    fi
    return 0
}

overlaps() {
    grep -c "more than one owner in the completed Region tree" "$1" || true
}

cross_quotes() {
    grep -c "crosses a quoted fallback region" "$1" || true
}

# ---- self-test: a known positive and a known negative --------------------------------
FIXTURES=$ROOT/tests/fixtures/recover-loop-else-if-early-returns
render_loose "$BASE" "$FIXTURES/v8/CB.class" "$WORK/selftest-cb-base.txt"
render_loose "$PATCHED" "$FIXTURES/v8/CB.class" "$WORK/selftest-cb-patched.txt"
positive_base=$(overlaps "$WORK/selftest-cb-base.txt")
positive_patched=$(overlaps "$WORK/selftest-cb-patched.txt")
if [ "$positive_base" -ne 1 ] || [ "$positive_patched" -ne 0 ]; then
    echo "SELF-TEST FAILED: CB overlap refusals base=$positive_base patched=$positive_patched (want 1 then 0)"
    exit 1
fi
render_loose "$BASE" "$FIXTURES/v8/LB.class" "$WORK/selftest-lb-base.txt"
render_loose "$PATCHED" "$FIXTURES/v8/LB.class" "$WORK/selftest-lb-patched.txt"
# The two MVP-out negatives keep their own sentence on **both** binaries, byte for byte — the
# baseline's two further overlap refusals are the two ladder cells this change recovers.
for sentence in "canonical block at BCI 47" "canonical block at BCI 56"; do
    if ! grep -q "$sentence" "$WORK/selftest-lb-base.txt"; then
        echo "SELF-TEST FAILED: the baseline lost the negative sentence '$sentence'"
        exit 1
    fi
    if ! grep -q "$sentence" "$WORK/selftest-lb-patched.txt"; then
        echo "SELF-TEST FAILED: the patched binary lost the negative sentence '$sentence'"
        exit 1
    fi
done
negative_base=$(overlaps "$WORK/selftest-lb-base.txt")
negative_patched=$(overlaps "$WORK/selftest-lb-patched.txt")
cross_base=$(cross_quotes "$WORK/selftest-lb-base.txt")
cross_patched=$(cross_quotes "$WORK/selftest-lb-patched.txt")
if [ "$negative_base" -ne 4 ] || [ "$negative_patched" -ne 2 ]; then
    echo "SELF-TEST FAILED: LB overlap refusals base=$negative_base patched=$negative_patched (want 4 then 2)"
    exit 1
fi
if [ "$cross_base" -ne 2 ] || [ "$cross_patched" -ne 2 ]; then
    echo "SELF-TEST FAILED: LB cross-quote refusals base=$cross_base patched=$cross_patched (want 2 then 2)"
    exit 1
fi
# The controls' own members are byte-identical inside CB's render: only `loopElseIfRet` moves.
for method in loopElseIfNoRet loopIfElseRet noLoopElseIfRet; do
    sed -n "/static int $method(/,/^    }$/p" "$WORK/selftest-cb-base.txt" >"$WORK/selftest-$method-base.txt"
    sed -n "/static int $method(/,/^    }$/p" "$WORK/selftest-cb-patched.txt" >"$WORK/selftest-$method-patched.txt"
    if ! diff -q "$WORK/selftest-$method-base.txt" "$WORK/selftest-$method-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: the control `$method` moved on the patched binary"
        exit 1
    fi
done
echo "SELF-TEST OK: known positive CB 1 -> 0 overlap refusals; known negative LB 2 -> 2 overlaps and 2 -> 2 cross-quotes; the three controls byte-identical"

# ---- the sweep ------------------------------------------------------------------------
cd "$ROOT"
: >"$WORK/candidates-class.txt"
for root in openspec/evidence tests/fixtures; do
    find "$root" -name '*.class' >>"$WORK/candidates-class.txt"
done
loose=$(wc -l <"$WORK/candidates-class.txt" | tr -d ' ')
echo "loose candidate classes: $loose"

# The archive corpus: a class a patrol only committed inside a jar is a class all the same, so every
# `.class` entry of every jar below the two roots is asked the same question.
: >"$WORK/candidates-jar.txt"
find openspec/evidence tests/fixtures -name '*.jar' | while IFS= read -r jar; do
    unzip -Z1 "$jar" 2>/dev/null | grep '\.class$' | while IFS= read -r entry; do
        printf '%s!%s\n' "$jar" "$entry" >>"$WORK/candidates-jar.txt"
    done
done
packed=$(wc -l <"$WORK/candidates-jar.txt" | tr -d ' ')
echo "archive candidate classes: $packed"

moved=0
unrendered=0
overlaps_before=0
overlaps_after=0

while IFS= read -r path; do
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render_loose "$BASE" "$path" "$WORK/$key.base.txt" || { unrendered=$((unrendered + 1)); continue; }
    render_loose "$PATCHED" "$path" "$WORK/$key.patched.txt" || { unrendered=$((unrendered + 1)); continue; }
    before=$(overlaps "$WORK/$key.base.txt")
    after=$(overlaps "$WORK/$key.patched.txt")
    overlaps_before=$((overlaps_before + before))
    overlaps_after=$((overlaps_after + after))
    if ! diff -q "$WORK/$key.base.txt" "$WORK/$key.patched.txt" >/dev/null 2>&1; then
        moved=$((moved + 1))
        echo "MOVED: $path (overlap refusals $before -> $after)"
    fi
done <"$WORK/candidates-class.txt"

while IFS= read -r candidate; do
    jar=${candidate%%!*}
    entry=${candidate#*!}
    key=$(printf 'jar_%s_%s' "$jar" "$entry" | tr '/' '_')
    render_jar "$BASE" "$jar" "$entry" "$WORK/$key.base.txt" || { unrendered=$((unrendered + 1)); continue; }
    render_jar "$PATCHED" "$jar" "$entry" "$WORK/$key.patched.txt" || { unrendered=$((unrendered + 1)); continue; }
    before=$(overlaps "$WORK/$key.base.txt")
    after=$(overlaps "$WORK/$key.patched.txt")
    overlaps_before=$((overlaps_before + before))
    overlaps_after=$((overlaps_after + after))
    if ! diff -q "$WORK/$key.base.txt" "$WORK/$key.patched.txt" >/dev/null 2>&1; then
        moved=$((moved + 1))
        echo "MOVED: $candidate (overlap refusals $before -> $after)"
    fi
done <"$WORK/candidates-jar.txt"

echo "moved classes: $moved"
echo "unrendered candidates: $unrendered"
echo "canonical-overlap refusal sentences across the candidates: $overlaps_before -> $overlaps_after"
