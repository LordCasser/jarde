#!/bin/sh
# The change's corpus render differential: every class file under `openspec/evidence` and
# `tests/fixtures` — loose files and every `.class` entry of every committed jar — is rendered by
# two binaries (the baseline, built from this change's parent commit, and the patched one) through
# the same entry point, and the two texts are diffed. A class whose render moves is printed with the
# refusals it gained or lost.
#
# The count is only trusted after the script's own self-test: the known positive (`OP.sideEffect`
# loses its one bound-receiver refusal on the patched binary, and keeps it on the baseline, on both
# compiler legs) and the known negative (`BRN`'s three negatives keep all three refusals on **both**
# binaries, byte for byte) must both hold, or the script exits non-zero. Every render's first line
# must carry jarde's own self-header — a render of nothing is not a render, and counting refusals in
# one would be a false zero. A loose class whose file name is not its own internal name is resolved
# against the class file itself; one that renders under no name is reported, never dropped silently.
set -eu

BASE=${BASE:-/tmp/br-bin/baseline}
PATCHED=${PATCHED:-/tmp/br-bin/patched}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
WORK=${1:-/tmp/br-corpus}
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
    # A file whose name is neither its class name nor a suffix of its path (a patrol's `original.class`
    # holding `TwrAudit`): the class states its own name, and `javap` reads it off the file.
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

refusals() {
    grep -c "adapting this bound receiver would move its null failure" "$1" || true
}

# ---- self-test: a known positive and a known negative --------------------------------
FIXTURES=$ROOT/openspec/changes/recover-proved-nonnull-bound-receivers/results/fixtures
for leg in v8 v8-javac8; do
    render_loose "$BASE" "$FIXTURES/$leg/OP.class" "$WORK/selftest-op-$leg-base.txt"
    render_loose "$PATCHED" "$FIXTURES/$leg/OP.class" "$WORK/selftest-op-$leg-patched.txt"
    positive_base=$(refusals "$WORK/selftest-op-$leg-base.txt")
    positive_patched=$(refusals "$WORK/selftest-op-$leg-patched.txt")
    if [ "$positive_base" -ne 1 ] || [ "$positive_patched" -ne 0 ]; then
        echo "SELF-TEST FAILED: OP ($leg) refusals base=$positive_base patched=$positive_patched (want 1 then 0)"
        exit 1
    fi
done
render_loose "$BASE" "$FIXTURES/v8/BRN.class" "$WORK/selftest-brn-base.txt"
render_loose "$PATCHED" "$FIXTURES/v8/BRN.class" "$WORK/selftest-brn-patched.txt"
negative_base=$(refusals "$WORK/selftest-brn-base.txt")
negative_patched=$(refusals "$WORK/selftest-brn-patched.txt")
if [ "$negative_base" -ne 3 ] || [ "$negative_patched" -ne 3 ]; then
    echo "SELF-TEST FAILED: BRN refusals base=$negative_base patched=$negative_patched (want 3 then 3)"
    exit 1
fi
if ! diff -q "$WORK/selftest-brn-base.txt" "$WORK/selftest-brn-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: BRN's own text moved on the patched binary"
    exit 1
fi
echo "SELF-TEST OK: known positive OP 1 -> 0 refusals on both legs; known negative BRN 3 -> 3, byte-identical"

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
refusals_before=0
refusals_after=0

while IFS= read -r path; do
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render_loose "$BASE" "$path" "$WORK/$key.base.txt" || { unrendered=$((unrendered + 1)); continue; }
    render_loose "$PATCHED" "$path" "$WORK/$key.patched.txt" || { unrendered=$((unrendered + 1)); continue; }
    before=$(refusals "$WORK/$key.base.txt")
    after=$(refusals "$WORK/$key.patched.txt")
    refusals_before=$((refusals_before + before))
    refusals_after=$((refusals_after + after))
    if ! diff -q "$WORK/$key.base.txt" "$WORK/$key.patched.txt" >/dev/null 2>&1; then
        moved=$((moved + 1))
        echo "MOVED: $path (refusals $before -> $after)"
        diff -u "$WORK/$key.base.txt" "$WORK/$key.patched.txt" | sed -n '1,14p'
    fi
done <"$WORK/candidates-class.txt"

while IFS= read -r candidate; do
    jar=${candidate%%!*}
    entry=${candidate#*!}
    key=$(printf 'jar_%s_%s' "$jar" "$entry" | tr '/' '_')
    render_jar "$BASE" "$jar" "$entry" "$WORK/$key.base.txt" || { unrendered=$((unrendered + 1)); continue; }
    render_jar "$PATCHED" "$jar" "$entry" "$WORK/$key.patched.txt" || { unrendered=$((unrendered + 1)); continue; }
    before=$(refusals "$WORK/$key.base.txt")
    after=$(refusals "$WORK/$key.patched.txt")
    refusals_before=$((refusals_before + before))
    refusals_after=$((refusals_after + after))
    if ! diff -q "$WORK/$key.base.txt" "$WORK/$key.patched.txt" >/dev/null 2>&1; then
        moved=$((moved + 1))
        echo "MOVED: $candidate (refusals $before -> $after)"
        diff -u "$WORK/$key.base.txt" "$WORK/$key.patched.txt" | sed -n '1,14p'
    fi
done <"$WORK/candidates-jar.txt"

echo "moved classes: $moved"
echo "unrendered candidates: $unrendered"
echo "bound-receiver refusal sentences across the candidates: $refusals_before -> $refusals_after"
