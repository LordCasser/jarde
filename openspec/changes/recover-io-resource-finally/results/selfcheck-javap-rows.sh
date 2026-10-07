#!/bin/sh
# The two `java.io` rows' transcription self-check (`widening-row-sources` protocol): re-run
# `javap` for the five types the section names, reformat each answer to the file's index column
# (a shorter name padded to 33 columns, a longer one separated by a single space) and compare the
# result with the committed block, byte for byte.
#
# Usage: sh selfcheck-javap-rows.sh [rt.jar] [javap]
#   defaults: the Corretto 1.8.0_432 rt.jar and its own javap, the protocol file's own tool.
#
# Output: the diff of the committed block and the re-read one (empty on success), and the verdict.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
RT=${1:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/jre/lib/rt.jar}
JAVAP=${2:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javap}
HEADERS="$ROOT/openspec/evidence/java-syntax-2026-10-05/widening-row-sources/javap-headers.txt"
WORK=$(mktemp -d)
trap 'rm -rf "$WORK"' EXIT

[ -f "$RT" ] || { echo "no rt.jar at $RT" >&2; exit 2; }
[ -x "$JAVAP" ] || { echo "no javap at $JAVAP" >&2; exit 2; }

# The committed block: the section's five transcription lines, verbatim.
awk '/^# 2026-10-07, change `recover-io-resource-finally`/{found=1; next} found && /^java\.io\./{print}' \
    "$HEADERS" > "$WORK/committed.txt"

# The re-read block: the same five types, each header line reformatted to the index column.
for type in \
    java.io.FileInputStream \
    java.io.InputStreamReader \
    java.io.BufferedReader \
    java.io.InputStream \
    java.io.Reader
do
    line=$("$JAVAP" -classpath "$RT" "$type" | sed -n '2p')
    [ -n "$line" ] || { echo "javap printed no header for $type" >&2; exit 3; }
    printf '%s%s%s\n' "$type" \
        "$(awk -v n="${#type}" 'BEGIN { for (i = n; i < 33; i++) printf " " }')" \
        "$line" >> "$WORK/read.txt"
done

# The two constructor positions the rows serve, compared against the section's commented lines.
for signature in \
    'public java.io.InputStreamReader(java.io.InputStream, java.lang.String) throws java.io.UnsupportedEncodingException;' \
    'public java.io.BufferedReader(java.io.Reader);'
do
    type=${signature#public }
    type=${type%%(*}
    name=$(basename "$type")
    "$JAVAP" -classpath "$RT" "$type" | grep -F "  $signature" > /dev/null \
        || { echo "javap does not state the constructor position: $signature" >&2; exit 4; }
    grep -F "#   $signature" "$HEADERS" > /dev/null \
        || { echo "the committed block does not state the constructor position: $signature" >&2; exit 5; }
    : "$name"
done

if diff -u "$WORK/committed.txt" "$WORK/read.txt"; then
    echo "SELF-CHECK OK: the committed java.io lines are javap's own, byte for byte"
else
    echo "SELF-CHECK FAILED: the committed block and the re-read block differ" >&2
    exit 1
fi
