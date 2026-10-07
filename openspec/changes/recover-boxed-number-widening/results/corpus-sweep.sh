#!/bin/sh
# The change's corpus render differential: every class file under `openspec/evidence/**` whose
# constant pool mentions the `java.lang.Number` target descriptor is rendered by two binaries — the
# baseline (the tree before this change's `build.rs` rows) and the patched one — through the same
# entry point, and the two texts are diffed. A class whose render moves is printed with its reason.
#
# The count is only trusted after the script's own self-test: the known positives (`C8` and `BN`
# lose their reference-conversion refusals on the patched binary) and the known negative (`BNX`
# keeps both refusals on both binaries) must hold, or the script exits non-zero.
set -eu

BASE=/tmp/bn-render/jarde-cli-base
PATCHED=/tmp/bn-render/jarde-cli-patched
WORK=${WORK:-/tmp/bn-render/sweep}
FIXTURES=tests/fixtures/recover-boxed-number-widening
rm -rf "$WORK"
mkdir -p "$WORK"

render() {
    binary=$1
    policy=$2
    path=$3
    name=$4
    out=$5
    "$binary" class-source --policy "$policy" --input "$path" --class "$name" \
        --format text >"$out" 2>"$out.err" || echo "RENDER-EXIT=$?" >>"$out"
}

# The self-header assert, applied where a count is read: a refusal count is only trusted from a
# file that starts with the presentation header. A corpus fixture whose internal class name differs
# from its file name (the hand-built negative fixtures do) answers an error document instead; that
# is reported as a non-render and compared byte for byte on both binaries, never counted.
is_render() {
    head -1 "$1" | grep -q '^// jarde: presentation of'
}

refusals() {
    grep -c "no safe reference conversion evidence" "$1" || true
}

# The class's own internal name: `javap`'s declaration line names it, and the hand-built negative
# fixtures are committed under file names that differ from it. A file `javap` cannot read falls
# back to its base name, and is then reported as a non-render rather than counted.
internal_name() {
    path=$1
    name=$(javap -p "$path" 2>/dev/null | head -3 | awk '
        BEGIN { found = "" }
        { line = $0
          sub(/[[:space:]]*\{[[:space:]]*$/, "", line)
          n = split(line, a, " ")
          for (i = 1; i <= n; i++)
            if (found == "" && (a[i] == "class" || a[i] == "interface" || a[i] == "enum")) {
              name = a[i + 1]
              gsub(/<.*/, "", name)
              found = name
            } }
        END { if (found != "") print found }')
    if [ -z "$name" ]; then
        name=$(basename "$path" .class)
    fi
    printf '%s' "$name"
}

# ---- self-test: the known positives and the known negative ---------------------------
for class in C8 BN; do
    render "$BASE" single-class "$FIXTURES/v8/$class.class" "$class" "$WORK/selftest-$class-base.txt"
    render "$PATCHED" single-class "$FIXTURES/v8/$class.class" "$class" "$WORK/selftest-$class-patched.txt"
    is_render "$WORK/selftest-$class-base.txt" && is_render "$WORK/selftest-$class-patched.txt" ||
        { echo "SELF-TEST FAILED: $class is not a render on both binaries"; exit 1; }
    base=$(refusals "$WORK/selftest-$class-base.txt")
    patched=$(refusals "$WORK/selftest-$class-patched.txt")
    if [ "$patched" -ne 0 ]; then
        echo "SELF-TEST FAILED: $class patched refusals=$patched (want 0)"
        exit 1
    fi
    echo "self-test positive $class: base $base -> patched $patched refusals"
done
base=$(refusals "$WORK/selftest-C8-base.txt")
[ "$base" -gt 0 ] || { echo "SELF-TEST FAILED: C8 base refusals=$base (want > 0)"; exit 1; }

render "$BASE" single-class "$FIXTURES/v8/BNX.class" BNX "$WORK/selftest-BNX-base.txt"
render "$PATCHED" single-class "$FIXTURES/v8/BNX.class" BNX "$WORK/selftest-BNX-patched.txt"
is_render "$WORK/selftest-BNX-base.txt" && is_render "$WORK/selftest-BNX-patched.txt" ||
    { echo "SELF-TEST FAILED: BNX is not a render on both binaries"; exit 1; }
negative_base=$(refusals "$WORK/selftest-BNX-base.txt")
negative_patched=$(refusals "$WORK/selftest-BNX-patched.txt")
if [ "$negative_base" -ne 2 ] || [ "$negative_patched" -ne 2 ]; then
    echo "SELF-TEST FAILED: BNX refusals base=$negative_base patched=$negative_patched (want 2 then 2)"
    exit 1
fi
echo "SELF-TEST OK: known positives recover; known negative BNX 2 -> 2 refusals"

# ---- the sweep ------------------------------------------------------------------------
PATTERN='Ljava/lang/Number;'

# The loose corpus: the same class files the earlier widening slices swept.
grep -rlE "$PATTERN" --include='*.class' openspec/evidence >"$WORK/candidates-class.txt" || true
loose=$(wc -l <"$WORK/candidates-class.txt" | tr -d ' ')
echo "loose candidate classes (target descriptor in the pool): $loose"

# The archive corpus: a class the patrols only committed inside a jar is a class all the same, so
# every `.class` entry of every jar below `openspec/evidence` is asked the same question.
: >"$WORK/candidates-jar.txt"
find openspec/evidence -name '*.jar' | while IFS= read -r jar; do
    unzip -Z1 "$jar" 2>/dev/null | grep '\.class$' | while IFS= read -r entry; do
        if unzip -p "$jar" "$entry" 2>/dev/null | LC_ALL=C grep -qE "$PATTERN"; then
            printf '%s!%s\n' "$jar" "$entry" >>"$WORK/candidates-jar.txt"
        fi
    done
done
packed=$(wc -l <"$WORK/candidates-jar.txt" | tr -d ' ')
echo "archive candidate classes: $packed"

moved=0
nonrenders=0
refusals_before=0
refusals_after=0

while IFS= read -r path; do
    file=$(internal_name "$path")
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render "$BASE" single-class "$path" "$file" "$WORK/$key.base.txt"
    render "$PATCHED" single-class "$path" "$file" "$WORK/$key.patched.txt"
    if ! is_render "$WORK/$key.base.txt" || ! is_render "$WORK/$key.patched.txt"; then
        nonrenders=$((nonrenders + 1))
        if ! diff -q "$WORK/$key.base.txt" "$WORK/$key.patched.txt" >/dev/null 2>&1; then
            echo "NON-RENDER MOVED (investigate): $path"
            exit 1
        fi
        echo "NON-RENDER (same document on both binaries): $path"
        continue
    fi
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
    file=$(basename "$entry" .class)
    key=$(printf 'jar_%s_%s' "$jar" "$file" | tr '/' '_')
    render "$BASE" plain-jar "$jar" "$file" "$WORK/$key.base.txt"
    render "$PATCHED" plain-jar "$jar" "$file" "$WORK/$key.patched.txt"
    if ! is_render "$WORK/$key.base.txt" || ! is_render "$WORK/$key.patched.txt"; then
        nonrenders=$((nonrenders + 1))
        if ! diff -q "$WORK/$key.base.txt" "$WORK/$key.patched.txt" >/dev/null 2>&1; then
            echo "NON-RENDER MOVED (investigate): $candidate"
            exit 1
        fi
        echo "NON-RENDER (same document on both binaries): $candidate"
        continue
    fi
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
echo "non-render candidates (identical on both binaries, not counted): $nonrenders"
echo "refusal sentences across the rendered candidates: $refusals_before -> $refusals_after"
