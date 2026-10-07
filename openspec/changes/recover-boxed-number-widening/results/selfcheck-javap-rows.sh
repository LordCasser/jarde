#!/bin/sh
# Task 1.1's transcription self-check: re-run `javap` on the seven release-8 types this change's
# rows are read from — the six boxed numeric classes and `java.lang.Number` itself — format each
# declaration line with the row-source file's own index column, and diff the result against the
# lines the file actually carries. A non-empty diff (or a failing `javap`) means the committed
# transcription is not verbatim.
set -eu

JAVA_HOME_8=${JAVA_HOME_8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home}
RT="$JAVA_HOME_8/jre/lib/rt.jar"
JAVAP="$JAVA_HOME_8/bin/javap"
HEADERS=openspec/evidence/java-syntax-2026-10-05/widening-row-sources/javap-headers.txt

TYPES="java.lang.Byte java.lang.Short java.lang.Integer java.lang.Long java.lang.Float \
java.lang.Double java.lang.Number"

work=${WORK:-/tmp/bn-render/selfcheck}
rm -rf "$work"
mkdir -p "$work"
trap 'rm -rf "$work"' EXIT

echo "rt.jar sha256: $(shasum -a 256 "$RT" | awk '{print $1}')"

for type in $TYPES; do
    "$JAVAP" -classpath "$RT" "$type" | sed -n '2p' |
        awk -v name="$type" -v pad=33 '
            { if (length(name) >= pad) printf "%s %s\n", name, $0;
              else printf "%-*s%s\n", pad, name, $0 }'
done >"$work/fresh.txt"

# The file's own transcription of the same seven lines: the seven non-comment lines that name one
# of the seven types (`java.lang.Number` is the exact-name anchor the six `extends` clauses are
# read against).
grep -v '^#' "$HEADERS" | grep -E '^java\.lang\.(Byte|Short|Integer|Long|Float|Double|Number) ' \
    >"$work/committed.txt"

echo "the seven types javap answered:"
wc -l <"$work/fresh.txt"
diff -u "$work/committed.txt" "$work/fresh.txt" &&
    echo "SELF-CHECK OK: the committed lines are javap's own, byte for byte"
