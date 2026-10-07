#!/bin/sh
# The zero-regression differential over the **whole** evidence corpus: every class file under
# `openspec/evidence/**` (not only the lock-guard candidates) is rendered by the baseline binary and
# by this slice's binary, and the two texts are compared. A change that widened the certificate past
# its own shape would move a class here even when that class never names a lock.
#
# The script's own self-test: the anchor's own member set must move (the family's refusals
# disappear) and a healthy control must not. The anchor is committed as a jar, so the anchor check
# renders the jar's entry through the same two binaries.
#
# Usage: BASE=<baseline cli> PATCHED=<current cli> 02-corpus-diff.sh
set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
BASE=${BASE:-/tmp/lk/baseline-target/debug/jarde-cli}
PATCHED=${PATCHED:-$ROOT/target/debug/jarde-cli}
WORK=${WORK:-/tmp/lk/full-corpus}
PARALLEL=${PARALLEL:-8}
OUT="$HERE/02-corpus-diff.out"
rm -rf "$WORK"
mkdir -p "$WORK/out"

cat >"$WORK/render-one.sh" <<'EOF'
#!/bin/sh
# render-one.sh <binary> <class-file> <output>
#
# The class's own internal name comes from `javap`'s declaration line (the hand-built negative
# fixtures are committed under file names that differ from it); a file `javap` cannot read falls
# back to its base name and is then reported as a non-render rather than counted.
set -eu
binary=$1
path=$2
out=$3
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
"$binary" class-source --policy single-class --input "$path" --class "$name" \
    --format text >"$out" 2>"$out.err" || echo "RENDER-EXIT=$?" >>"$out"
EOF
chmod +x "$WORK/render-one.sh"

cd "$ROOT"
find openspec/evidence -name '*.class' | LC_ALL=C sort >"$WORK/classes.txt"
total=$(wc -l <"$WORK/classes.txt" | tr -d ' ')
echo "corpus classes: $total" | tee "$OUT"

while IFS= read -r path; do
    key=$(printf '%s' "$path" | tr '/' '_')
    printf '%s %s %s\n' "$BASE" "$path" "$WORK/out/$key.base.txt"
done <"$WORK/classes.txt" >"$WORK/jobs-base.txt"
while IFS= read -r path; do
    key=$(printf '%s' "$path" | tr '/' '_')
    printf '%s %s %s\n' "$PATCHED" "$path" "$WORK/out/$key.patched.txt"
done <"$WORK/classes.txt" >"$WORK/jobs-patched.txt"

xargs -P "$PARALLEL" -L1 sh "$WORK/render-one.sh" <"$WORK/jobs-base.txt"
echo "baseline renders done" | tee -a "$OUT"
xargs -P "$PARALLEL" -L1 sh "$WORK/render-one.sh" <"$WORK/jobs-patched.txt"
echo "patched renders done" | tee -a "$OUT"

# ---- self-test ------------------------------------------------------------------------
anchor=$ROOT/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar
"$BASE" class-source --policy plain-jar --input "$anchor" --class LK --format text \
    >"$WORK/anchor.base.txt" 2>/dev/null
"$PATCHED" class-source --policy plain-jar --input "$anchor" --class LK --format text \
    >"$WORK/anchor.patched.txt" 2>/dev/null
head -1 "$WORK/anchor.base.txt" | grep -q '^// jarde: presentation of' ||
    { echo "SELF-TEST FAILED: the anchor's baseline render has no header"; exit 1; }
base_anchor=$(shasum -a 256 <"$WORK/anchor.base.txt" | cut -d' ' -f1)
patch_anchor=$(shasum -a 256 <"$WORK/anchor.patched.txt" | cut -d' ' -f1)
[ "$base_anchor" != "$patch_anchor" ] ||
    { echo "SELF-TEST FAILED: the anchor did not move"; exit 1; }
echo "self-test: the anchor moved (want moved)" | tee -a "$OUT"
control=$ROOT/tests/fixtures/p3-handlers/v8/Guarded.class
key=$(printf '%s' "$control" | tr '/' '_')
"$BASE" class-source --policy single-class --input "$control" --class Guarded --format text \
    >"$WORK/out/$key.base.txt" 2>/dev/null
"$PATCHED" class-source --policy single-class --input "$control" --class Guarded --format text \
    >"$WORK/out/$key.patched.txt" 2>/dev/null
diff -q "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >/dev/null ||
    { echo "SELF-TEST FAILED: the healthy control moved"; exit 1; }
echo "self-test: the healthy control did not move (want no)" | tee -a "$OUT"

# ---- the differential -----------------------------------------------------------------
# A corpus fixture whose internal class name differs from its file name (the hand-built negative
# fixtures do) answers an error document instead of a presentation; it is reported as a non-render
# and compared byte for byte, never counted as a moved class. A one-sided non-render is an anomaly
# and fails the run.
moved=0
nonrenders=0
while IFS= read -r path; do
    key=$(printf '%s' "$path" | tr '/' '_')
    base_is_render=no
    patched_is_render=no
    head -1 "$WORK/out/$key.base.txt" | grep -q '^// jarde: presentation of' && base_is_render=yes
    head -1 "$WORK/out/$key.patched.txt" | grep -q '^// jarde: presentation of' && patched_is_render=yes
    if [ "$base_is_render" != "$patched_is_render" ]; then
        echo "ONE-SIDED NON-RENDER (investigate): $path (base=$base_is_render patched=$patched_is_render)" | tee -a "$OUT"
        exit 1
    fi
    if [ "$base_is_render" = no ]; then
        nonrenders=$((nonrenders + 1))
        diff -q "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >/dev/null 2>&1 ||
            { echo "NON-RENDER MOVED (investigate): $path" | tee -a "$OUT"; exit 1; }
        continue
    fi
    if ! diff -q "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >/dev/null 2>&1; then
        moved=$((moved + 1))
        before=$(grep -c "@bytecode" "$WORK/out/$key.base.txt" || true)
        after=$(grep -c "@bytecode" "$WORK/out/$key.patched.txt" || true)
        echo "MOVED: $path (refusal lines $before -> $after)" | tee -a "$OUT"
        diff -u "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >"$WORK/moved-$key.diff" || true
    fi
done <"$WORK/classes.txt"
echo "classes compared: $total" | tee -a "$OUT"
echo "non-render classes (identical on both binaries, not counted): $nonrenders" | tee -a "$OUT"
echo "moved classes: $moved" | tee -a "$OUT"
