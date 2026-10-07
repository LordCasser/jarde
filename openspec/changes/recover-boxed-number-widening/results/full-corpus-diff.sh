#!/bin/sh
# The zero-regression differential over the **whole** evidence corpus: every class file under
# `openspec/evidence/**` (not only the `java.lang.Number` candidates) is rendered by the baseline
# binary and by the patched one and the two texts are compared. A change that touched a row, a
# gate or a render other than the six new rows would move a class here even when that class never
# names `java.lang.Number`.
#
# The script's own self-test: the anchor `C8` must move (its refusal disappears) and `C7` — the
# patrol's healthy control — must not.
set -eu

BASE=${BASE:-/tmp/bn-render/jarde-cli-base}
PATCHED=${PATCHED:-/tmp/bn-render/jarde-cli-patched}
WORK=${WORK:-/tmp/bn-render/full-corpus}
PARALLEL=${PARALLEL:-8}
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

find openspec/evidence -name '*.class' | LC_ALL=C sort >"$WORK/classes.txt"
total=$(wc -l <"$WORK/classes.txt" | tr -d ' ')
echo "corpus classes: $total"

while IFS= read -r path; do
    key=$(printf '%s' "$path" | tr '/' '_')
    printf '%s %s %s\n' "$BASE" "$path" "$WORK/out/$key.base.txt"
done <"$WORK/classes.txt" >"$WORK/jobs-base.txt"
while IFS= read -r path; do
    key=$(printf '%s' "$path" | tr '/' '_')
    printf '%s %s %s\n' "$PATCHED" "$path" "$WORK/out/$key.patched.txt"
done <"$WORK/classes.txt" >"$WORK/jobs-patched.txt"

xargs -P "$PARALLEL" -L1 sh "$WORK/render-one.sh" <"$WORK/jobs-base.txt"
echo "baseline renders done"
xargs -P "$PARALLEL" -L1 sh "$WORK/render-one.sh" <"$WORK/jobs-patched.txt"
echo "patched renders done"

# ---- self-test ------------------------------------------------------------------------
selftest() {
    class_path=$1
    want=$2
    key=$(printf '%s' "$class_path" | tr '/' '_')
    head -1 "$WORK/out/$key.base.txt" | grep -q '^// jarde: presentation of' ||
        { echo "SELF-TEST FAILED: $class_path is not a render"; exit 1; }
    if diff -q "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >/dev/null 2>&1; then
        moved=no
    else
        moved=yes
    fi
    if [ "$moved" != "$want" ]; then
        echo "SELF-TEST FAILED: $class_path moved=$moved (want $want)"
        exit 1
    fi
    echo "self-test: $class_path moved=$moved (want $want)"
}

selftest openspec/evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/fixture/C8.class yes
selftest openspec/evidence/java-syntax-2026-10-03/boxed-number-widening-patrol/fixture/C7.class no

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
        echo "ONE-SIDED NON-RENDER (investigate): $path (base=$base_is_render patched=$patched_is_render)"
        exit 1
    fi
    if [ "$base_is_render" = no ]; then
        nonrenders=$((nonrenders + 1))
        diff -q "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >/dev/null 2>&1 ||
            { echo "NON-RENDER MOVED (investigate): $path"; exit 1; }
        continue
    fi
    if ! diff -q "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >/dev/null 2>&1; then
        moved=$((moved + 1))
        before=$(grep -c "no safe reference conversion evidence" "$WORK/out/$key.base.txt" || true)
        after=$(grep -c "no safe reference conversion evidence" "$WORK/out/$key.patched.txt" || true)
        echo "MOVED: $path (refusals $before -> $after)"
        diff -u "$WORK/out/$key.base.txt" "$WORK/out/$key.patched.txt" >"$WORK/moved-$key.diff" || true
    fi
done <"$WORK/classes.txt"
echo "classes compared: $total"
echo "non-render classes (identical on both binaries, not counted): $nonrenders"
echo "moved classes: $moved"
