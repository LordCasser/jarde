#!/bin/sh
# Task 1.1's reflective check of the direct-edge universe, run against the JDK the widening rows
# are transcribed from (Corretto 1.8.0_432). Every `.class` entry of `rt.jar` is handed to the
# probe, which loads each one and reports the classes whose own `extends` clause names
# `java.lang.Number`, the ones that reach it through another class, and `Number`'s own superclass
# and interfaces. The output is the evidence the six `java.lang` rows are the whole direct set.
set -eu

JH=${JAVA_HOME_8:-/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home}
HERE=$(cd "$(dirname "$0")" && pwd)
WORK=${WORK:-/tmp/bn-render/probe}
rm -rf "$WORK"
mkdir -p "$WORK"

echo "rt.jar sha256: $(shasum -a 256 "$JH/jre/lib/rt.jar" | awk '{print $1}')"
echo "java -version: $("$JH/bin/java" -version 2>&1 | head -1)"

# Every top-level and nested class of the archive, as a dotted name (nested `$` names kept: the
# probe's own `isAssignableFrom` answer must see them too).
"$JH/bin/jar" tf "$JH/jre/lib/rt.jar" | grep '\.class$' | sed 's/\.class$//; s|/|.|g' >"$WORK/names.txt"
echo "class entries handed to the probe: $(wc -l <"$WORK/names.txt" | tr -d ' ')"

"$JH/bin/javac" -d "$WORK" "$HERE/NumberUniverse.java"
"$JH/bin/java" -cp "$WORK" NumberUniverse <"$WORK/names.txt"
