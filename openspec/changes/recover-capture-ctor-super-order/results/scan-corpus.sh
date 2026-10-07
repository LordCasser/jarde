#!/bin/sh
# Corpus two-leg scan (change `recover-capture-ctor-super-order`, task 3.2).
#
# Renders every `tests/fixtures/**/*.class` through `jarde-cli class-source --policy single-class`
# twice — once with the mainline binary (commit 8519530e, built from `git archive HEAD` into
# /tmp/jarde-baseline) and once with the change binary — capturing stdout and the exit status per
# class into two trees, then `diff -r` over them.
#
#   sh openspec/changes/recover-capture-ctor-super-order/results/scan-corpus.sh <baseline-binary> <change-binary> <out-dir>
#
# The scan is self-tested: it fails if either binary is missing, if the two trees do not hold the
# same number of captures, or if any capture is empty (a class that bound nothing is recorded as
# `unbound` rather than silently skipped).
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

find "$root/tests/fixtures" -name '*.class' | sort | while IFS= read -r class; do
    relative=${class#"$root/tests/fixtures/"}
    name=$(basename "$class" .class)
    key=$(printf '%s' "$relative" | tr '/$' '__')
    for leg in a b; do
        if [ "$leg" = a ]; then binary=$baseline; else binary=$change; fi
        status=0
        "$binary" class-source --policy single-class --class "$name" --input "$class" \
            --format text >"$out/$leg/$key.txt" 2>"$out/$leg/$key.err" || status=$?
        if [ "$status" -ne 0 ] && [ ! -s "$out/$leg/$key.txt" ]; then
            printf 'unbound (exit %s)\n' "$status" >"$out/$leg/$key.txt"
        fi
        printf '\n[exit %s]\n' "$status" >>"$out/$leg/$key.txt"
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
changed=$(grep -c '^diff -r' "$out/diff.txt" || true)
echo "differing classes: $changed"
grep '^diff -r' "$out/diff.txt" || true
