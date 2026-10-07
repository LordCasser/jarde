#!/bin/sh
# The change's corpus render differential (`recover-short-circuit-local-branch-reads`).
#
# Two passes over every class the repository commits under `openspec/evidence` and `tests/fixtures`:
#
#   A. single-class posture, every loose `.class` file;
#   C. plain-jar posture, every `.class` entry of every committed jar.
#
# Each pass renders with two binaries — the baseline built from this change's parent commit `HEAD`
# (into /tmp/brslice/bin/jarde-cli.baseline) and the patched one — and diffs the two texts.
#
# Self-tests, before any count is believed:
#   * the known positive: the patrol's `op2.jar` moves (the anchor's refusal disappears);
#   * the known negative: this change's `BranchReadNegatives` is byte-identical on both binaries;
#   * the unrelated negative: `tests/fixtures/p3-handlers/v8/Guarded.class` is byte-identical.
#
# Every render's first line must carry jarde's own self-header: a render of nothing is not a render,
# and counting moves in one would be a false reading. A candidate that renders under no name it
# states is counted and printed, never dropped silently.
set -eu

BASE=${BASE:-/tmp/brslice/bin/jarde-cli.baseline}
PATCHED=${PATCHED:-/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11634-9017-7ae3-93be-dc1f6ccbc123/target/debug/jarde-cli}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
WORK=${1:-/tmp/brslice/sweep}
OUT=$(cd "$(dirname "$0")" && pwd)/03-corpus-sweep.out
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

# The name one class file declares for itself, read off the file (`javap -v`'s first unindented
# `class`/`interface`/`enum` line; the InnerClasses table further down says `class` too, so only
# lines before the first member are read).
stated_name() {
    javap -p -v "$1" 2>/dev/null \
        | awk '/^[[:space:]]*\{/ { exit } /^[^[:space:]]/ && $0 !~ /^Classfile/' \
        | grep -m1 -oE '(class|interface|enum) [^ <]+' \
        | awk '{print $2}'
}

render_loose() {
    binary=$1
    path=$2
    out=$3
    name=$(basename "${path%.class}")
    if try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    name=$(stated_name "$path")
    if [ -n "$name" ] && try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    return 1
}

render_jar() {
    binary=$1
    jar=$2
    entry=$3
    out=$4
    name=${entry%.class}
    try_render "$binary" plain-jar "$jar" "$name" "$out"
}

# Whether an output file is a presentation at all (its own self-header is the gate).
is_render() {
    head -1 "$1" | grep -q '// jarde: presentation of'
}

{
    echo "# The whole committed corpus, rendered by the parent commit's gate and by this slice's"
    echo "# (change recover-short-circuit-local-branch-reads, sweep of $(date -u +%Y-%m-%dT%H:%M:%SZ))"
} >"$OUT"

# ---- self-tests -----------------------------------------------------------------------
OP2_JAR=$ROOT/openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar
try_render "$BASE" plain-jar "$OP2_JAR" OP2 "$WORK/selftest-op2-base.txt"
try_render "$PATCHED" plain-jar "$OP2_JAR" OP2 "$WORK/selftest-op2-patched.txt"
base_anchor=$(grep -c 'boolean local1 = arg0 > 0' "$WORK/selftest-op2-base.txt" || true)
patched_anchor=$(grep -c 'boolean local1 = arg0 > 0' "$WORK/selftest-op2-patched.txt" || true)
if [ "$base_anchor" -ne 0 ] || [ "$patched_anchor" -ne 1 ]; then
    echo "SELF-TEST FAILED: OP2.condAssignOld base=$base_anchor patched=$patched_anchor (want 0 then 1)"
    exit 1
fi
NEG=$ROOT/tests/fixtures/recover-short-circuit-local-branch-reads/v8/BranchReadNegatives.class
try_render "$BASE" single-class "$NEG" BranchReadNegatives "$WORK/selftest-neg-base.txt"
try_render "$PATCHED" single-class "$NEG" BranchReadNegatives "$WORK/selftest-neg-patched.txt"
if ! diff -q "$WORK/selftest-neg-base.txt" "$WORK/selftest-neg-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: this change's BranchReadNegatives refusals moved"
    exit 1
fi
HEALTHY=$ROOT/tests/fixtures/p3-handlers/v8/Guarded.class
try_render "$BASE" single-class "$HEALTHY" Guarded "$WORK/selftest-healthy-base.txt"
try_render "$PATCHED" single-class "$HEALTHY" Guarded "$WORK/selftest-healthy-patched.txt"
if ! diff -q "$WORK/selftest-healthy-base.txt" "$WORK/selftest-healthy-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: the healthy control moved"
    exit 1
fi
echo "SELF-TEST OK: OP2 anchor 0 -> 1; this change's negatives byte-identical; healthy control byte-identical" | tee -a "$OUT"

# ---- pass A: the single-class posture, the whole loose corpus --------------------------
cd "$ROOT"
: >"$WORK/candidates-class.txt"
for root in openspec/evidence tests/fixtures; do
    find "$root" -name '*.class' >>"$WORK/candidates-class.txt"
done
loose=$(wc -l <"$WORK/candidates-class.txt" | tr -d ' ')
echo "pass A loose candidate classes: $loose" | tee -a "$OUT"

moved_a=0
unrendered_a=0
while IFS= read -r path; do
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render_loose "$BASE" "$path" "$WORK/$key.a-base.txt" || true
    render_loose "$PATCHED" "$path" "$WORK/$key.a-patched.txt" || true
    base_is=no
    is_render "$WORK/$key.a-base.txt" && base_is=yes
    patched_is=no
    is_render "$WORK/$key.a-patched.txt" && patched_is=yes
    if [ "$base_is" != "$patched_is" ]; then
        echo "ONE-SIDED NON-RENDER (investigate): $path (base=$base_is patched=$patched_is)" | tee -a "$OUT"
        exit 1
    fi
    if [ "$base_is" = no ]; then
        # A class that renders under no name it states is not a moved class; it is compared byte
        # for byte all the same, so a one-sided refusal of the same bytes cannot hide here.
        unrendered_a=$((unrendered_a + 1))
        if ! diff -q "$WORK/$key.a-base.txt" "$WORK/$key.a-patched.txt" >/dev/null 2>&1; then
            echo "NON-RENDER MOVED (investigate): $path" | tee -a "$OUT"
            exit 1
        fi
        continue
    fi
    if ! diff -q "$WORK/$key.a-base.txt" "$WORK/$key.a-patched.txt" >/dev/null 2>&1; then
        moved_a=$((moved_a + 1))
        echo "MOVED (single-class): $path" | tee -a "$OUT"
        diff "$WORK/$key.a-base.txt" "$WORK/$key.a-patched.txt" | sed -n '1,8p' | sed 's/^/    /' >>"$OUT"
    fi
done <"$WORK/candidates-class.txt"
echo "pass A: moved=$moved_a unrendered=$unrendered_a" | tee -a "$OUT"

# ---- pass C: every committed jar's own entries ----------------------------------------
: >"$WORK/candidates-jar.txt"
for jar in $(find openspec/evidence tests/fixtures -name '*.jar'); do
    unzip -Z1 "$jar" 2>/dev/null | grep '\.class$' | while IFS= read -r entry; do
        printf '%s!%s\n' "$jar" "$entry" >>"$WORK/candidates-jar.txt"
    done
done
packed=$(wc -l <"$WORK/candidates-jar.txt" | tr -d ' ')
echo "archive candidate classes: $packed" | tee -a "$OUT"

moved_c=0
unrendered_c=0
while IFS= read -r candidate; do
    jar=${candidate%%!*}
    entry=${candidate#*!}
    key=$(printf 'jar_%s_%s' "$jar" "$entry" | tr '/' '_')
    render_jar "$BASE" "$jar" "$entry" "$WORK/$key.c-base.txt" || true
    render_jar "$PATCHED" "$jar" "$entry" "$WORK/$key.c-patched.txt" || true
    base_is=no
    is_render "$WORK/$key.c-base.txt" && base_is=yes
    patched_is=no
    is_render "$WORK/$key.c-patched.txt" && patched_is=yes
    if [ "$base_is" != "$patched_is" ]; then
        echo "ONE-SIDED NON-RENDER (investigate): $candidate (base=$base_is patched=$patched_is)" | tee -a "$OUT"
        exit 1
    fi
    if [ "$base_is" = no ]; then
        unrendered_c=$((unrendered_c + 1))
        if ! diff -q "$WORK/$key.c-base.txt" "$WORK/$key.c-patched.txt" >/dev/null 2>&1; then
            echo "NON-RENDER MOVED (investigate): $candidate" | tee -a "$OUT"
            exit 1
        fi
        continue
    fi
    if ! diff -q "$WORK/$key.c-base.txt" "$WORK/$key.c-patched.txt" >/dev/null 2>&1; then
        moved_c=$((moved_c + 1))
        echo "MOVED (jar): $candidate" | tee -a "$OUT"
        diff "$WORK/$key.c-base.txt" "$WORK/$key.c-patched.txt" | sed -n '1,8p' | sed 's/^/    /' >>"$OUT"
    fi
done <"$WORK/candidates-jar.txt"
echo "pass C: moved=$moved_c unrendered=$unrendered_c" | tee -a "$OUT"

echo "moved classes: single-class=$moved_a jar=$moved_c total=$((moved_a + moved_c))" | tee -a "$OUT"
echo "unrendered candidates: A=$unrendered_a C=$unrendered_c" | tee -a "$OUT"
