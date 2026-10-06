#!/bin/sh
# The change's corpus render differential (`recover-inline-conditional-concat-operands`).
#
# Two passes over every class the repository commits under `openspec/evidence` and `tests/fixtures`:
#
#   A. single-class posture, every loose `.class` file;
#   C. plain-jar posture, every `.class` entry of every committed jar.
#
# Each pass renders with two binaries — the baseline built from this change's parent commit `HEAD`
# (into /tmp/icco/base-target) and the patched one — and diffs the two texts.
#
# Self-tests, before any count is believed:
#   * the known positive: this change's own `ICC.class` writes the source-form chain on the patched
#     binary and not on the baseline;
#   * the known negative: this change's `ICN.class` renders the same refusals on both binaries;
#   * the unrelated negative: the postfix patrol's `NG.class` is byte-identical on both binaries.
#
# Every render's first line must carry jarde's own self-header: a render of nothing is not a render,
# and counting moves in one would be a false reading. A candidate that renders under no name it
# states is counted and printed, never dropped silently.
set -eu

BASE=${BASE:-/tmp/icco/base-target/debug/jarde-cli}
PATCHED=${PATCHED:-/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11224-3550-7e00-96b0-a031bb36109f/target/debug/jarde-cli}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
WORK=${1:-/tmp/icco/inline-concat-corpus}
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
FIXTURES=$ROOT/tests/fixtures/recover-inline-conditional-concat-operands
ICC_JAR=$WORK/selftest-icc.jar
( cd "$FIXTURES/v8" && jar cf "$ICC_JAR" ICC.class 'ICC$Inner.class' )
try_render "$BASE" plain-jar "$ICC_JAR" ICC "$WORK/selftest-icc-base.txt"
try_render "$PATCHED" plain-jar "$ICC_JAR" ICC "$WORK/selftest-icc-patched.txt"
base_chain=$(grep -c '== n))' "$WORK/selftest-icc-base.txt" || true)
patched_chain=$(grep -c '== n))' "$WORK/selftest-icc-patched.txt" || true)
if [ "$base_chain" -ne 0 ] || [ "$patched_chain" -ne 1 ]; then
    echo "SELF-TEST FAILED: ICC source-form chain base=$base_chain patched=$patched_chain (want 0 then 1)"
    exit 1
fi
ICN_JAR=$WORK/selftest-icn.jar
( cd "$FIXTURES/v8" && jar cf "$ICN_JAR" ICN.class )
try_render "$BASE" plain-jar "$ICN_JAR" ICN "$WORK/selftest-icn-base.txt"
try_render "$PATCHED" plain-jar "$ICN_JAR" ICN "$WORK/selftest-icn-patched.txt"
if ! diff -q "$WORK/selftest-icn-base.txt" "$WORK/selftest-icn-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: this change's ICN refusals moved"
    exit 1
fi
NG_JAR=$WORK/selftest-ng.jar
( cd "$ROOT/tests/fixtures/recover-postfix-old-value-snapshot/v8" && jar cf "$NG_JAR" NG.class )
try_render "$BASE" plain-jar "$NG_JAR" NG "$WORK/selftest-postfix-ng-base.txt"
try_render "$PATCHED" plain-jar "$NG_JAR" NG "$WORK/selftest-postfix-ng-patched.txt"
if ! diff -q "$WORK/selftest-postfix-ng-base.txt" "$WORK/selftest-postfix-ng-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: the postfix patrol's NG negatives moved"
    exit 1
fi
echo "SELF-TEST OK: ICC chain 0 -> 1; this change's ICN byte-identical; postfix NG byte-identical"

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
