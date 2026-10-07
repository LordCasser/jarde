#!/bin/sh
# Evidence-corpus two-leg scan (change `recover-capture-ctor-super-order`, task 3.2).
#
# The same two-leg render as `scan-corpus.sh`, over the frozen evidence corpus instead of the
# fixtures: every `openspec/evidence/**/*.jar` (each of its `.class` entries, through
# `--policy plain-jar --input <jar> --class <internal name>`) and every loose
# `openspec/evidence/**/*.class` (`--policy single-class`). Two trees of captures, `diff -r`, and
# the same self-tests (equal counts, no empty capture).
#
#   sh openspec/changes/recover-capture-ctor-super-order/results/scan-evidence.sh <baseline-binary> <change-binary> <out-dir>
set -eu

baseline=$1
change=$2
out=$3
root=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)

for binary in "$baseline" "$change"; do
    [ -x "$binary" ] || { echo "not executable: $binary" >&2; exit 2; }
done

rm -rf "$out"
mkdir -p "$out/a" "$out/b"

render() {
    binary=$1
    leg=$2
    key=$3
    subject=$4
    shift 4
    status=0
    "$binary" class-source --format text --output "$out/$leg/$key.txt" "$@" 2>"$out/$leg/$key.err" || status=$?
    if [ "$status" -ne 0 ] && [ ! -s "$out/$leg/$key.txt" ]; then
        printf 'unbound (exit %s)\n' "$status" >"$out/$leg/$key.txt"
    fi
    printf '\n[exit %s]\n' "$status" >>"$out/$leg/$key.txt"
    printf '# %s\n' "$subject" >>"$out/$leg/$key.txt"
}

# A key bounded to a filesystem-safe length: a readable head plus a short digest of the whole
# subject, so a long jar path plus entry name cannot overflow the name limit.
key_of() {
    head=$(printf '%s' "$1" | tr '/$' '__' | cut -c1-100)
    digest=$(printf '%s' "$1" | shasum -a 1 | cut -c1-8)
    printf '%s__%s' "$head" "$digest"
}

find "$root/openspec/evidence" -name '*.jar' | sort | while IFS= read -r jar; do
    relative=${jar#"$root/openspec/evidence/"}
    unzip -Z1 "$jar" | grep '\.class$' | sort | while IFS= read -r entry; do
        name=${entry%.class}
        for leg in a b; do
            if [ "$leg" = a ]; then binary=$baseline; else binary=$change; fi
            render "$binary" "$leg" "$(key_of "$relative::$entry")" "$relative::$entry" \
                --policy plain-jar --input "$jar" --class "$name"
        done
    done
done

find "$root/openspec/evidence" -name '*.class' | sort | while IFS= read -r class; do
    relative=${class#"$root/openspec/evidence/"}
    name=$(basename "$class" .class)
    for leg in a b; do
        if [ "$leg" = a ]; then binary=$baseline; else binary=$change; fi
        render "$binary" "$leg" "$(key_of "$relative")" "$relative" \
            --policy single-class --input "$class" --class "$name"
    done
done

count_a=$(find "$out/a" -name '*.txt' | wc -l | tr -d ' ')
count_b=$(find "$out/b" -name '*.txt' | wc -l | tr -d ' ')
[ "$count_a" = "$count_b" ] || { echo "capture counts differ: $count_a vs $count_b" >&2; exit 3; }
[ "$count_a" -gt 0 ] || { echo "no captures" >&2; exit 4; }
empty=$(find "$out" -name '*.txt' -size 0 | wc -l | tr -d ' ')
[ "$empty" = 0 ] || { echo "$empty empty capture(s)" >&2; exit 5; }

echo "captures: $count_a per leg"
diff -r "$out/a" "$out/b" >"$out/diff.txt" || true
echo "differing captures: $(grep -c '^diff -r' "$out/diff.txt" || true)"
grep '^diff -r' "$out/diff.txt" | grep '\.txt' || true
