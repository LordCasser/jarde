#!/bin/sh
# The change's corpus render differential (`recover-boolean-int-bitwise-operands`).
#
# Two passes over every class the repository commits under `openspec/evidence` and `tests/fixtures`:
#
#   A. single-class posture, every loose `.class` file;
#   C. plain-jar posture, every `.class` entry of every committed jar.
#
# Each pass renders with two binaries — the baseline built from this change's parent commit `HEAD`
# (a separate worktree, /tmp/jarde-bwslice-baseline, target dir /tmp/jarde-baseline-target) and the
# patched one — and diffs the two texts.
#
# Self-tests, before any count is believed:
#   * the known positive: the patrol's own `BW.class` renders the two anchors on the patched binary
#     and the two refusals on the baseline;
#   * the known negative: this change's `BWN.class` renders the same refusals on both binaries;
#   * the precedent families: the conditional-rhs field-compound change's `RC.class`/`RCN.class`, the
#     chained-field-assignment change's `CF.class`/`NEG.class` and the inline-conditional-concat
#     change's `ICM.class`/`ICN.class` are byte-identical on both binaries.
#
# Every render's first line must carry jarde's own self-header: a render of nothing is not a render,
# and counting moves in one would be a false reading. A candidate that renders under no name it
# states is counted and printed, never dropped silently.
set -eu

ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11405-eb09-78f1-babc-dbe140bcaad2
BASE=${BASE:-/tmp/jarde-baseline-target/debug/jarde-cli}
PATCHED=${PATCHED:-$ROOT/target/debug/jarde-cli}
WORK=${1:-/tmp/boolean-int-bitwise/corpus}
rm -rf "$WORK"
mkdir -p "$WORK"

# One render that answers whether it was one: the output carries jarde's own self-header.
try_render() {
    local binary=$1
    local policy=$2
    local path=$3
    local name=$4
    local out=$5
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
    local binary=$1
    local path=$2
    local out=$3
    local name
    name=$(basename "${path%.class}")
    if try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    name=$(stated_name "$path")
    if [ -n "$name" ] && try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    echo "UNRENDERED: $path renders under no name it states"
    return 1
}

render_jar() {
    local binary=$1
    local jar=$2
    local entry=$3
    local out=$4
    local name=${entry%.class}
    if ! try_render "$binary" plain-jar "$jar" "$name" "$out"; then
        echo "UNRENDERED: $jar!$entry renders under its own entry name"
        return 1
    fi
    return 0
}

# ---- self-tests -----------------------------------------------------------------------
PATROL=$ROOT/openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.class
try_render "$BASE" single-class "$PATROL" BW "$WORK/selftest-bw-base.txt"
try_render "$PATCHED" single-class "$PATROL" BW "$WORK/selftest-bw-patched.txt"
base_refusals=$(grep -c 'which no Java integral or boolean bitwise expression accepts' "$WORK/selftest-bw-base.txt" || true)
patched_refusals=$(grep -c 'which no Java integral or boolean bitwise expression accepts' "$WORK/selftest-bw-patched.txt" || true)
patched_andnot=$(grep -c 'return arg0 & !arg1;' "$WORK/selftest-bw-patched.txt" || true)
patched_mix=$(grep -c 'local1 = local1 \^ local5;' "$WORK/selftest-bw-patched.txt" || true)
if [ "$base_refusals" -ne 2 ] || [ "$patched_refusals" -ne 0 ] || [ "$patched_andnot" -ne 1 ] || [ "$patched_mix" -ne 1 ]; then
    echo "SELF-TEST FAILED: BW refusals base=$base_refusals patched=$patched_refusals, andNot=$patched_andnot mix=$patched_mix (want 2 then 0/1/1)"
    exit 1
fi
BWN=$ROOT/tests/fixtures/recover-boolean-int-bitwise-operands/v8/BWN.class
try_render "$BASE" single-class "$BWN" BWN "$WORK/selftest-bwn-base.txt"
try_render "$PATCHED" single-class "$BWN" BWN "$WORK/selftest-bwn-patched.txt"
if ! diff -q "$WORK/selftest-bwn-base.txt" "$WORK/selftest-bwn-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: this change's BWN refusals moved"
    exit 1
fi
RCF=$ROOT/tests/fixtures/recover-conditional-rhs-field-compound
( cd "$RCF/v8" && jar cf "$WORK/selftest-rc.jar" BI.class RC.class RCN.class )
for class in BI RC RCN; do
    try_render "$BASE" plain-jar "$WORK/selftest-rc.jar" "$class" "$WORK/selftest-$class-base.txt"
    try_render "$PATCHED" plain-jar "$WORK/selftest-rc.jar" "$class" "$WORK/selftest-$class-patched.txt"
    if ! diff -q "$WORK/selftest-$class-base.txt" "$WORK/selftest-$class-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: the conditional-rhs field-compound change's $class presentation moved"
        exit 1
    fi
done
CFF=$ROOT/tests/fixtures/recover-chained-field-assignment
( cd "$CFF/v8" && jar cf "$WORK/selftest-cf.jar" CF.class NEG.class )
for class in CF NEG; do
    try_render "$BASE" plain-jar "$WORK/selftest-cf.jar" "$class" "$WORK/selftest-$class-base.txt"
    try_render "$PATCHED" plain-jar "$WORK/selftest-cf.jar" "$class" "$WORK/selftest-$class-patched.txt"
    if ! diff -q "$WORK/selftest-$class-base.txt" "$WORK/selftest-$class-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: the chained-field-assignment change's $class presentation moved"
        exit 1
    fi
done
ICF=$ROOT/tests/fixtures/recover-inline-conditional-concat-operands
( cd "$ICF/v8" && jar cf "$WORK/selftest-icm.jar" ICM.class ICN.class )
for class in ICM ICN; do
    try_render "$BASE" plain-jar "$WORK/selftest-icm.jar" "$class" "$WORK/selftest-$class-base.txt"
    try_render "$PATCHED" plain-jar "$WORK/selftest-icm.jar" "$class" "$WORK/selftest-$class-patched.txt"
    if ! diff -q "$WORK/selftest-$class-base.txt" "$WORK/selftest-$class-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: the inline-conditional-concat change's $class presentation moved"
        exit 1
    fi
done
echo "SELF-TEST OK: BW refusals 2 -> 0 with both anchors; BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical"

# ---- pass A: the single-class posture, the whole loose corpus --------------------------
cd "$ROOT"
: >"$WORK/candidates-class.txt"
for root in openspec/evidence tests/fixtures; do
    find "$root" -name '*.class' >>"$WORK/candidates-class.txt"
done
loose=$(wc -l <"$WORK/candidates-class.txt" | tr -d ' ')
echo "pass A loose candidate classes: $loose"

moved_a=0
unrendered_a=0
while IFS= read -r path; do
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render_loose "$BASE" "$path" "$WORK/$key.a-base.txt" || { unrendered_a=$((unrendered_a + 1)); continue; }
    render_loose "$PATCHED" "$path" "$WORK/$key.a-patched.txt" || { unrendered_a=$((unrendered_a + 1)); continue; }
    if ! diff -q "$WORK/$key.a-base.txt" "$WORK/$key.a-patched.txt" >/dev/null 2>&1; then
        moved_a=$((moved_a + 1))
        echo "MOVED (single-class): $path"
        diff "$WORK/$key.a-base.txt" "$WORK/$key.a-patched.txt" | sed -n '1,8p' | sed 's/^/    /'
    fi
done <"$WORK/candidates-class.txt"
echo "pass A: moved=$moved_a unrendered=$unrendered_a"

# ---- pass C: every committed jar's own entries ----------------------------------------
: >"$WORK/candidates-jar.txt"
find openspec/evidence tests/fixtures -name '*.jar' | while IFS= read -r jar; do
    unzip -Z1 "$jar" 2>/dev/null | grep '\.class$' | while IFS= read -r entry; do
        printf '%s!%s\n' "$jar" "$entry" >>"$WORK/candidates-jar.txt"
    done
done
packed=$(wc -l <"$WORK/candidates-jar.txt" | tr -d ' ')
echo "archive candidate classes: $packed"

moved_c=0
unrendered_c=0
while IFS= read -r candidate; do
    jar=${candidate%%!*}
    entry=${candidate#*!}
    key=$(printf 'jar_%s_%s' "$jar" "$entry" | tr '/' '_')
    render_jar "$BASE" "$jar" "$entry" "$WORK/$key.c-base.txt" || { unrendered_c=$((unrendered_c + 1)); continue; }
    render_jar "$PATCHED" "$jar" "$entry" "$WORK/$key.c-patched.txt" || { unrendered_c=$((unrendered_c + 1)); continue; }
    if ! diff -q "$WORK/$key.c-base.txt" "$WORK/$key.c-patched.txt" >/dev/null 2>&1; then
        moved_c=$((moved_c + 1))
        echo "MOVED (jar): $candidate"
        diff "$WORK/$key.c-base.txt" "$WORK/$key.c-patched.txt" | sed -n '1,8p' | sed 's/^/    /'
    fi
done <"$WORK/candidates-jar.txt"
echo "pass C: moved=$moved_c unrendered=$unrendered_c"

echo "moved classes: single-class=$moved_a jar=$moved_c total=$((moved_a + moved_c))"
echo "unrendered candidates: A=$unrendered_a C=$unrendered_c"
