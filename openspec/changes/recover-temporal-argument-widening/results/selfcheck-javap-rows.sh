#!/bin/sh
# Task 1's self-check: re-run `javap` on the twelve release-8 types this change transcribes,
# format each declaration line with the row-source file's own index column, and diff the result
# against the lines the file actually carries. A non-empty diff (or a failing `javap`) means the
# transcription is not verbatim.
set -eu

JAVA_HOME_8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home
RT="$JAVA_HOME_8/jre/lib/rt.jar"
JAVAP="$JAVA_HOME_8/bin/javap"
HEADERS=openspec/evidence/java-syntax-2026-10-05/widening-row-sources/javap-headers.txt

TYPES="java.time.LocalDateTime java.time.LocalDate java.time.LocalTime java.time.Instant \
java.time.ZonedDateTime java.time.OffsetDateTime java.time.OffsetTime \
java.time.temporal.Temporal java.time.temporal.TemporalAccessor \
java.util.concurrent.CompletableFuture java.util.concurrent.CompletionStage \
java.util.concurrent.Future"

work=$(mktemp -d)
trap 'rm -rf "$work"' EXIT

for type in $TYPES; do
    "$JAVAP" -classpath "$RT" "$type" | sed -n '2p' |
        awk -v name="$type" -v pad=33 '
            { if (length(name) >= pad) printf "%s %s\n", name, $0;
              else printf "%-*s%s\n", pad, name, $0 }'
done >"$work/fresh.txt"

# The file's own transcription of the same twelve lines: the twelve non-comment lines the change
# appended after the closing line of the pre-existing java.lang/java.util set. The label above the
# block is a comment, so everything that is not a comment and names one of the twelve types is it.
grep -v '^#' "$HEADERS" | grep -E '^java\.(time|util\.concurrent)\.' >"$work/committed.txt"

echo "the twelve types javap answered:"
wc -l <"$work/fresh.txt"
diff -u "$work/committed.txt" "$work/fresh.txt" && echo "SELF-CHECK OK: the committed lines are javap's own, byte for byte"
