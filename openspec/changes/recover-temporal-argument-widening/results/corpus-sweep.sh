#!/bin/sh
# The change's corpus render differential: every class file under `openspec/evidence/**` whose
# constant pool mentions one of the four target descriptors is rendered by two binaries — the
# baseline (the tree before this change's `build.rs` rows) and the patched one — through the same
# entry point, and the two texts are diffed. A class whose render moves is printed with its reason.
#
# The count is only trusted after the script's own self-test: the known positive (`JT.fmt/spans`
# lose their four refusals on the patched binary) and the known negative (`TWX.month/year` keep
# both refusals on both binaries) must both hold, or the script exits non-zero.
set -eu

BASE=/tmp/tw-render/jarde-cli-base
PATCHED=/tmp/tw-render/jarde-cli-patched2
WORK=/tmp/tw-render/sweep
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

refusals() {
    grep -c "no safe reference conversion evidence" "$1" || true
}

# ---- self-test: a known positive and a known negative --------------------------------
JAR=openspec/evidence/java-syntax-2026-10-05/java-time-temporal-patrol/fixture/jt.jar
render "$BASE" plain-jar "$JAR" JT "$WORK/selftest-jt-base.txt"
render "$PATCHED" plain-jar "$JAR" JT "$WORK/selftest-jt-patched.txt"
positive_base=$(refusals "$WORK/selftest-jt-base.txt")
positive_patched=$(refusals "$WORK/selftest-jt-patched.txt")
if [ "$positive_base" -ne 4 ] || [ "$positive_patched" -ne 0 ]; then
    echo "SELF-TEST FAILED: JT refusals base=$positive_base patched=$positive_patched (want 4 then 0)"
    exit 1
fi

TWX=tests/fixtures/recover-temporal-argument-widening/v8/TWX.class
render "$BASE" single-class "$TWX" TWX "$WORK/selftest-twx-base.txt"
render "$PATCHED" single-class "$TWX" TWX "$WORK/selftest-twx-patched.txt"
negative_base=$(refusals "$WORK/selftest-twx-base.txt")
negative_patched=$(refusals "$WORK/selftest-twx-patched.txt")
if [ "$negative_base" -ne 2 ] || [ "$negative_patched" -ne 2 ]; then
    echo "SELF-TEST FAILED: TWX refusals base=$negative_base patched=$negative_patched (want 2 then 2)"
    exit 1
fi
echo "SELF-TEST OK: known positive JT 4 -> 0 refusals; known negative TWX 2 -> 2"

# ---- the sweep ------------------------------------------------------------------------
PATTERN='Ljava/time/temporal/(Temporal|TemporalAccessor);|Ljava/util/concurrent/(CompletionStage|Future);'

# The loose corpus: the same 1987 class files the earlier widening slices swept.
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
refusals_before=0
refusals_after=0

while IFS= read -r path; do
    file=$(basename "$path" .class)
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render "$BASE" single-class "$path" "$file" "$WORK/$key.base.txt"
    render "$PATCHED" single-class "$path" "$file" "$WORK/$key.patched.txt"
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
    file=${entry%.class}
    key=$(printf 'jar_%s_%s' "$jar" "$file" | tr '/' '_')
    render "$BASE" plain-jar "$jar" "$file" "$WORK/$key.base.txt"
    render "$PATCHED" plain-jar "$jar" "$file" "$WORK/$key.patched.txt"
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
echo "refusal sentences across the candidates: $refusals_before -> $refusals_after"

