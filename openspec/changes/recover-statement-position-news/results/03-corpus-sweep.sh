#!/bin/sh
# The change's corpus render differential (`recover-statement-position-news`).
#
# Two passes over every class the repository commits under `openspec/evidence` and `tests/fixtures`:
#
#   A. single-class posture, every loose `.class` file;
#   C. plain-jar posture, every `.class` entry of every committed jar.
#
# Each pass renders with two binaries — the baseline built from this change's parent commit `HEAD`
# (a separate worktree, /tmp/jarde-spn-baseline, target dir /tmp/jarde-spn-baseline-target) and the
# patched one — and diffs the two texts.
#
# Self-tests, before any count is believed:
#   * the known positive: the patrol's own `B5.class` carries three statement-position refusals on
#     the baseline and none on the patched binary, with all three constructions written;
#   * the known ordering pin: this change's assembled `SPC.class` (the statement-position twin of
#     the frozen counterexample) is byte-identical on both binaries;
#   * the known negatives: this change's `SPN.class` is byte-identical on both binaries, and so is
#     the patrol's `B6.class` outside the two shapes that recover;
#   * the precedent families: the bitwise change's `BW.class`/`BWN.class`, the conditional-rhs
#     field-compound change's `RC.class`/`RCN.class`, the chained-field-assignment change's
#     `CF.class`/`NEG.class` and the inline-conditional-concat change's `ICM.class`/`ICN.class` are
#     byte-identical on both binaries.
#
# Every render's first line must carry jarde's own self-header: a render of nothing is not a render,
# and counting moves in one would be a false reading. A candidate that renders under no name it
# states is counted and printed, never dropped silently.
set -eu

ROOT=/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a11450-b786-7b93-b480-e65b9cc75b09
BASE=${BASE:-/tmp/jarde-spn-baseline-target/debug/jarde-cli}
PATCHED=${PATCHED:-$ROOT/target/debug/jarde-cli}
WORK=${1:-/tmp/statement-position-news/corpus}
rm -rf "$WORK"
mkdir -p "$WORK"

# Both binaries must exist before a single render is counted: the patched one is rebuilt with
# `cargo build -p jarde-cli --locked`, and the baseline one with
# `git worktree add /tmp/jarde-spn-baseline HEAD` + `CARGO_TARGET_DIR=/tmp/jarde-spn-baseline-target
# cargo build -p jarde-cli --locked`.
for binary in "$BASE" "$PATCHED"; do
    if [ ! -x "$binary" ]; then
        echo "MISSING BINARY: $binary — build it first (see the comment above)"
        exit 1
    fi
done

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
PATROL=$ROOT/openspec/evidence/java-syntax-2026-10-03/statement-new-patrol/fixture
try_render "$BASE" single-class "$PATROL/B5.class" B5 "$WORK/selftest-b5-base.txt"
try_render "$PATCHED" single-class "$PATROL/B5.class" B5 "$WORK/selftest-b5-patched.txt"
# The baseline refuses `main` whole (the statement-position shapes carry no place the builder could
# write), and the patched binary writes all three constructions and refuses nothing.
base_refused=$(grep -c 'jarde_refused_body' "$WORK/selftest-b5-base.txt" || true)
patched_refused=$(grep -c 'jarde_refused_body' "$WORK/selftest-b5-patched.txt" || true)
patched_statements=$(grep -c -F -e '        new B5();' -e '        new B5(7);' -e '        new B5$Sub();' "$WORK/selftest-b5-patched.txt" || true)
if [ "$base_refused" -ne 1 ] || [ "$patched_refused" -ne 0 ] || [ "$patched_statements" -ne 3 ]; then
    echo "SELF-TEST FAILED: B5 refused bodies base=$base_refused patched=$patched_refused, statements=$patched_statements (want 1 then 0/3)"
    exit 1
fi

FIX=$ROOT/tests/fixtures/recover-statement-position-news
for class in SPC; do
    try_render "$BASE" single-class "$FIX/$class.class" "$class" "$WORK/selftest-$class-base.txt"
    try_render "$PATCHED" single-class "$FIX/$class.class" "$class" "$WORK/selftest-$class-patched.txt"
    if ! diff -q "$WORK/selftest-$class-base.txt" "$WORK/selftest-$class-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: this change's $class presentation moved"
        exit 1
    fi
done
for leg in v8 v8-javac8; do
    try_render "$BASE" single-class "$FIX/$leg/SPN.class" SPN "$WORK/selftest-spn-$leg-base.txt"
    try_render "$PATCHED" single-class "$FIX/$leg/SPN.class" SPN "$WORK/selftest-spn-$leg-patched.txt"
    if ! diff -q "$WORK/selftest-spn-$leg-base.txt" "$WORK/selftest-spn-$leg-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: this change's SPN ($leg) presentation moved"
        exit 1
    fi
done

BW=$ROOT/openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.class
BWN=$ROOT/tests/fixtures/recover-boolean-int-bitwise-operands/v8/BWN.class
for pair in "$BW:BW" "$BWN:BWN"; do
    path=${pair%%:*}
    name=${pair#*:}
    try_render "$BASE" single-class "$path" "$name" "$WORK/selftest-$name-base.txt"
    try_render "$PATCHED" single-class "$path" "$name" "$WORK/selftest-$name-patched.txt"
    if ! diff -q "$WORK/selftest-$name-base.txt" "$WORK/selftest-$name-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: the bitwise change's $name presentation moved"
        exit 1
    fi
done

RCF=$ROOT/tests/fixtures/recover-conditional-rhs-field-compound
( cd "$RCF/v8" && jar cf "$WORK/selftest-rc.jar" BI.class RC.class RCN.class )
CFF=$ROOT/tests/fixtures/recover-chained-field-assignment
( cd "$CFF/v8" && jar cf "$WORK/selftest-cf.jar" CF.class NEG.class )
ICF=$ROOT/tests/fixtures/recover-inline-conditional-concat-operands
( cd "$ICF/v8" && jar cf "$WORK/selftest-icm.jar" ICM.class ICN.class )
for pair in "$WORK/selftest-rc.jar:BI" "$WORK/selftest-rc.jar:RC" "$WORK/selftest-rc.jar:RCN" \
            "$WORK/selftest-cf.jar:CF" "$WORK/selftest-cf.jar:NEG" \
            "$WORK/selftest-icm.jar:ICM" "$WORK/selftest-icm.jar:ICN"; do
    jar=${pair%%:*}
    name=${pair#*:}
    try_render "$BASE" plain-jar "$jar" "$name" "$WORK/selftest-$name-base.txt"
    try_render "$PATCHED" plain-jar "$jar" "$name" "$WORK/selftest-$name-patched.txt"
    if ! diff -q "$WORK/selftest-$name-base.txt" "$WORK/selftest-$name-patched.txt" >/dev/null; then
        echo "SELF-TEST FAILED: the precedent family's $name presentation moved"
        exit 1
    fi
done
echo "SELF-TEST OK: B5 refusals 3 -> 0 with all three statements; SPC/SPN byte-identical; BW/BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical"

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
