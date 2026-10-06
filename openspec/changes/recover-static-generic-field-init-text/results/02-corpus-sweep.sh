#!/bin/sh
# The change's corpus render differential. Three passes over every class the repository commits
# under `openspec/evidence` and `tests/fixtures`:
#
#   A. single-class posture, every loose `.class` file — the posture the fold does not run in, so
#      this change must move **nothing** here. A non-zero count is a self-test failure, not a
#      finding;
#   B. family-fold posture, every loose root class (`X.class` with no `$` in its file name),
#      rendered as `--policy plain-jar` over a staged jar of the family the class files state:
#      the root under the name it declares, plus every sibling whose own declared name is a member
#      of it. This is the posture the patrol's broken text was recorded in;
#   C. plain-jar posture, every `.class` entry of every committed jar.
#
# Each pass renders with two binaries — the baseline built from this change's parent commit
# `dccd21c3` and the patched one — and diffs the two texts.
#
# Self-tests, before any count is believed:
#   * the known positive: this change's own `v8/MN.class` family renders `Holdava` twice on the
#     baseline and zero times on the patched binary (pass B's posture);
#   * the known negative: the same class in pass A's posture is byte-identical on both binaries;
#   * the unrelated negative: the binary-search patrol's `bs.jar!BS.class` is byte-identical on both
#     binaries (pass C's posture);
#   * pass A's whole-corpus count of moved classes must be zero.
#
# Every render's first line must carry jarde's own self-header: a render of nothing is not a render,
# and counting residues in one would be a false zero. A candidate that renders under no name it
# states is counted and printed, never dropped silently.
set -eu

BASE=${BASE:-/tmp/rsgfi/bin/jarde-cli-base}
PATCHED=${PATCHED:-/Users/lordcasser/.grow/worktrees/projects-jarde/subagent-01a110cf-daa9-7d12-8e4e-ee2e330ed22c/target/release/jarde-cli}
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
WORK=${1:-/tmp/rsgfi/corpus}
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

# The name one class file declares for itself, read off the file: the declaration line `javap -v`
# writes unindented first (`class pkg.Outer$Inner<T extends java.lang.Object> extends …`), whose
# token right after the keyword is the name. Empty when the file states none (a damaged probe).
# Two traps this went through: the InnerClasses table further down the same output also says
# `class` (reading *it* is how the first version lost every `$`-named sibling — the family jar then
# held only the root, the fold was refused, and both binaries answered the same unfolded text: a
# false zero), and a generic class's declaration carries a `<…>` clause and a supertype, so the
# *last* field of the line is not the name either.
stated_name() {
    javap -p -v "$1" 2>/dev/null \
        | awk '/^[^[:space:]]/ && $0 !~ /^Classfile/' \
        | grep -m1 -oE '(class|interface|enum) [^ <]+' \
        | awk '{print $2}'
}

# One loose class file in the single-class posture, under the name it declares: the file's own name
# first, then every suffix of its path (shortest first), then the name `javap` reads off the file.
render_loose() {
    local binary=$1
    local path=$2
    local out=$3
    local stem=${path%.class}
    local name
    name=$(basename "$stem")
    if try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    local candidates=$stem
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
    name=$(stated_name "$path")
    if [ -n "$name" ] && try_render "$binary" single-class "$path" "$name" "$out"; then
        return 0
    fi
    echo "UNRENDERED: $path renders under no name it states"
    return 1
}

# The staged jar of one class's own family, under the names the class files state: the root plus
# every sibling whose stated name is a member of it. Printed on stdout; the caller removes it.
family_jar() {
    local path=$1
    local name=$2
    local directory
    directory=$(dirname "$path")
    local stage=$WORK/stage-$$
    local entry
    entry=$(printf '%s' "$name" | tr '.' '/')
    mkdir -p "$stage/$(dirname "$entry")"
    cp "$path" "$stage/$entry.class"
    local sibling
    local sibling_name
    for sibling in "$directory"/*.class; do
        [ "$sibling" = "$path" ] && continue
        sibling_name=$(stated_name "$sibling")
        case $sibling_name in
            "$name\$"*)
                entry=$(printf '%s' "$sibling_name" | tr '.' '/')
                mkdir -p "$stage/$(dirname "$entry")"
                cp "$sibling" "$stage/$entry.class"
                ;;
        esac
    done
    local jar=$WORK/family-$$.jar
    ( cd "$stage" && jar cf "$jar" . ) >/dev/null 2>&1 || true
    rm -rf "$stage"
    printf '%s\n' "$jar"
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

# The residues the patrol's samples carried, plus the general form of the same corruption: a
# non-comment line that keeps a folded member's `$`-pool name.
holdava() {
    grep -c 'Holdava' "$1" || true
}
pool_names() {
    grep -v '^[[:space:]]*//' "$1" | grep -c '\$' || true
}

# ---- self-tests -----------------------------------------------------------------------
FIXTURES=$ROOT/tests/fixtures/recover-static-generic-field-init-text
# The name reader itself: a nested class file must state its `$` name (the false zero this script
# first shipped), and a file that is not a readable class must state nothing.
nested_name=$(stated_name "$FIXTURES/v8/MN\$Hold.class")
if [ "$nested_name" != 'MN$Hold' ]; then
    echo "SELF-TEST FAILED: stated_name on MN\$Hold.class gave '$nested_name' (want MN\$Hold)"
    exit 1
fi
if [ -n "$(stated_name "$ROOT/tests/fixtures/recover-static-generic-field-init-text/README.md")" ]; then
    echo "SELF-TEST FAILED: stated_name answered for a file that is not a class"
    exit 1
fi
echo "SELF-TEST OK: stated_name reads MN\$Hold from the nested fixture and nothing from a non-class"
MN_JAR=$WORK/selftest-mn.jar
( cd "$FIXTURES/v8" && jar cf "$MN_JAR" MN.class 'MN$Hold.class' )
try_render "$BASE" plain-jar "$MN_JAR" MN "$WORK/selftest-mn-base.txt"
try_render "$PATCHED" plain-jar "$MN_JAR" MN "$WORK/selftest-mn-patched.txt"
positive_base=$(holdava "$WORK/selftest-mn-base.txt")
positive_patched=$(holdava "$WORK/selftest-mn-patched.txt")
if [ "$positive_base" -ne 2 ] || [ "$positive_patched" -ne 0 ]; then
    echo "SELF-TEST FAILED: MN Holdava residues base=$positive_base patched=$positive_patched (want 2 then 0)"
    exit 1
fi
render_loose "$BASE" "$FIXTURES/v8/MN.class" "$WORK/selftest-mn-single-base.txt"
render_loose "$PATCHED" "$FIXTURES/v8/MN.class" "$WORK/selftest-mn-single-patched.txt"
if ! diff -q "$WORK/selftest-mn-single-base.txt" "$WORK/selftest-mn-single-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: MN's single-class render moved"
    exit 1
fi
BS_JAR=$ROOT/openspec/evidence/java-syntax-2026-10-05/binary-search-twopointer-patrol/fixture/bs.jar
render_jar "$BASE" "$BS_JAR" BS.class "$WORK/selftest-bs-base.txt"
render_jar "$PATCHED" "$BS_JAR" BS.class "$WORK/selftest-bs-patched.txt"
if ! diff -q "$WORK/selftest-bs-base.txt" "$WORK/selftest-bs-patched.txt" >/dev/null; then
    echo "SELF-TEST FAILED: the unrelated bs.jar!BS.class render moved"
    exit 1
fi
echo "SELF-TEST OK: MN family Holdava 2 -> 0; MN single-class byte-identical; bs.jar!BS.class byte-identical"

# ---- pass A: the single-class posture, the whole loose corpus --------------------------
cd "$ROOT"
: >"$WORK/candidates-class.txt"
for root in openspec/evidence tests/fixtures; do
    find "$root" -name '*.class' >>"$WORK/candidates-class.txt"
done
loose=$(wc -l <"$WORK/candidates-class.txt" | tr -d ' ')
echo "pass A/B loose candidate classes: $loose"

moved_a=0
unrendered_a=0
while IFS= read -r path; do
    key=$(printf 'class_%s' "$path" | tr '/' '_')
    render_loose "$BASE" "$path" "$WORK/$key.a-base.txt" || { unrendered_a=$((unrendered_a + 1)); continue; }
    render_loose "$PATCHED" "$path" "$WORK/$key.a-patched.txt" || { unrendered_a=$((unrendered_a + 1)); continue; }
    if ! diff -q "$WORK/$key.a-base.txt" "$WORK/$key.a-patched.txt" >/dev/null 2>&1; then
        moved_a=$((moved_a + 1))
        echo "MOVED (single-class): $path"
    fi
done <"$WORK/candidates-class.txt"
echo "pass A: moved=$moved_a unrendered=$unrendered_a"
if [ "$moved_a" -ne 0 ]; then
    echo "SELF-TEST FAILED: the single-class pass moved $moved_a class(es); this change only acts inside the fold"
    exit 1
fi

# ---- pass B: the fold posture, every loose root class ---------------------------------
moved_b=0
unrendered_b=0
roots=0
while IFS= read -r path; do
    case $path in
        *'$'*) continue ;;
    esac
    roots=$((roots + 1))
    key=$(printf 'family_%s' "$path" | tr '/' '_')
    name=$(stated_name "$path")
    if [ -z "$name" ]; then
        unrendered_b=$((unrendered_b + 1))
        echo "UNRENDERED: $path states no class name"
        continue
    fi
    # One staged jar per root: both binaries read the same bytes.
    jar=$(family_jar "$path" "$name") || { unrendered_b=$((unrendered_b + 1)); continue; }
    base_ok=1
    patched_ok=1
    try_render "$BASE" plain-jar "$jar" "$name" "$WORK/$key.b-base.txt" || base_ok=0
    try_render "$PATCHED" plain-jar "$jar" "$name" "$WORK/$key.b-patched.txt" || patched_ok=0
    rm -f "$jar"
    if [ "$base_ok" -eq 0 ] || [ "$patched_ok" -eq 0 ]; then
        unrendered_b=$((unrendered_b + 1))
        echo "UNRENDERED: $path renders under no family name (base=$base_ok patched=$patched_ok)"
        continue
    fi
    if ! diff -q "$WORK/$key.b-base.txt" "$WORK/$key.b-patched.txt" >/dev/null 2>&1; then
        moved_b=$((moved_b + 1))
        echo "MOVED (fold): $path  Holdava $(holdava "$WORK/$key.b-base.txt") -> $(holdava "$WORK/$key.b-patched.txt"); pool-name lines $(pool_names "$WORK/$key.b-base.txt") -> $(pool_names "$WORK/$key.b-patched.txt")"
        diff "$WORK/$key.b-base.txt" "$WORK/$key.b-patched.txt" | sed -n '1,10p' | sed 's/^/    /'
    fi
done <"$WORK/candidates-class.txt"
echo "pass B: root classes=$roots moved=$moved_b unrendered=$unrendered_b"

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
        echo "MOVED (jar): $candidate  Holdava $(holdava "$WORK/$key.c-base.txt") -> $(holdava "$WORK/$key.c-patched.txt"); pool-name lines $(pool_names "$WORK/$key.c-base.txt") -> $(pool_names "$WORK/$key.c-patched.txt")"
        diff "$WORK/$key.c-base.txt" "$WORK/$key.c-patched.txt" | sed -n '1,10p' | sed 's/^/    /'
    fi
done <"$WORK/candidates-jar.txt"
echo "pass C: moved=$moved_c unrendered=$unrendered_c"

echo "moved classes: single-class=$moved_a fold=$moved_b jar=$moved_c total=$((moved_a + moved_b + moved_c))"
echo "unrendered candidates: A=$unrendered_a B=$unrendered_b C=$unrendered_c"
