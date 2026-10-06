#!/bin/sh
# Present every fixture input of `recover-conditional-rhs-field-compound` with **two** jarde
# binaries — a baseline and the change's own — and diff the texts.
#
# The gating evidence: which inputs flip (the anchors, whose materialised right-hand side the
# change admits) and which stay byte-identical (the negatives, whose refusals must keep their
# verbatim text, and every frozen anchor of the two precedent families).
#
# Usage: dual-leg-diff.sh <baseline jarde-cli> <patched jarde-cli> <fixtures dir> <out dir>
set -eu

BASE=$1
NEW=$2
FIXTURES=$3
OUT=$4
test -x "$BASE"
test -x "$NEW"
test -d "$FIXTURES"
rm -rf "$OUT"
mkdir -p "$OUT/base" "$OUT/new"

present() {
    cli=$1
    dir=$2
    label=$3
    class=$4
    input=$5
    policy=$6
    "$cli" class-source --input "$input" --policy "$policy" --class "$class" \
        --format text --output "$dir/$label-$class.txt" 2>/dev/null
}

for leg in frozen v8 v8-javac8; do
    case $leg in
    frozen)
        present "$BASE" "$OUT/base" frozen BI "$FIXTURES/bi.jar" plain-jar
        present "$NEW" "$OUT/new" frozen BI "$FIXTURES/bi.jar" plain-jar
        ;;
    *)
        for class in BI RC RCN; do
            present "$BASE" "$OUT/base" "$leg" "$class" "$FIXTURES/$leg/$class.class" single-class
            present "$NEW" "$OUT/new" "$leg" "$class" "$FIXTURES/$leg/$class.class" single-class
        done
        ;;
    esac
done

for file in "$OUT"/base/*.txt; do
    name=$(basename "$file")
    if cmp -s "$file" "$OUT/new/$name"; then
        echo "IDENTICAL  $name"
    else
        echo "CHANGED    $name"
        diff "$file" "$OUT/new/$name" | sed 's/^/    /' || true
    fi
done
