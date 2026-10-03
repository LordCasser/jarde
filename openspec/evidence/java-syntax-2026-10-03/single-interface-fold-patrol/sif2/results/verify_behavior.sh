#!/bin/bash
# Three-way behavior comparison for one (jar, class) under the sif2 change.
#
#   leg 1  original class file:  java -Xverify:all -cp <jar> <class>
#   leg 2  JADX dev Java input:  fixed JADX output, `javac --release 8`; the run leg is only
#                                taken when that compile succeeds — a failed compile would
#                                otherwise fall back to the original class on the classpath and
#                                report a behaviour the JADX text never produced.
#   leg 3  Jarde folded unit:    class-source text, `javac --release 8` against the original jar,
#                                then `java -Xverify:all`
#
# Usage: verify_behavior.sh <jar> <class> <outdir>
set -u
JAR="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"; CLASS="$2"; OUT="$3"
BIN=/tmp/sif2/jarde-cli-after
JADX="$HOME/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx"
rm -rf "$OUT"; mkdir -p "$OUT"
JARBASE="$(basename "$JAR")"

# leg 1
java -Xverify:all -cp "$JAR" "$CLASS" > "$OUT/original.out" 2>"$OUT/original.err"
echo "original run exit=$?" >> "$OUT/legs.txt"

# leg 3: jarde folded unit
"$BIN" class-source --input "$JAR" --class "$CLASS" --format text 2>/dev/null > "$OUT/$CLASS.jarde.java"
shasum -a 256 < "$OUT/$CLASS.jarde.java" > "$OUT/jarde.java.sha256"
mkdir -p "$OUT/jarde-build"
# The recovered text keeps the class file's own `package` line, so the source has to be written
# at the package path for `javac` to accept the public class's own file name.
PKG=$(grep -m1 '^package ' "$OUT/$CLASS.jarde.java" | sed 's/^package //;s/;//' | tr -d '[:space:]')
PKGDIR="$OUT/jarde-build${PKG:+/$PKG}"
mkdir -p "$PKGDIR"
SIMPLE="${CLASS##*.}"
cp "$OUT/$CLASS.jarde.java" "$PKGDIR/$SIMPLE.java"
cp "$JAR" "$OUT/jarde-build/$JARBASE"
( cd "$OUT/jarde-build" && javac --release 8 -cp "$JARBASE" "${PKG:+$PKG/}$SIMPLE.java" 2>javac.err )
JDK_EXIT=$?
echo "jarde javac exit=$JDK_EXIT" >> "$OUT/legs.txt"
if [ "$JDK_EXIT" -eq 0 ]; then
  ( cd "$OUT/jarde-build" && java -Xverify:all -cp ".:$JARBASE" "${PKG:+$PKG.}$SIMPLE" ) > "$OUT/jarde.out" 2>"$OUT/jarde.err"
  echo "jarde run exit=$?" >> "$OUT/legs.txt"
else
  cp "$OUT/jarde-build/javac.err" "$OUT/jarde.javac.err"
fi

# leg 2: fixed JADX dev
"$JADX" --no-res -d "$OUT/jadx-out" "$JAR" >/dev/null 2>&1
JADX_SRC=$(find "$OUT/jadx-out" -name "$CLASS.java" 2>/dev/null | head -1)
if [ -z "${JADX_SRC:-}" ]; then
  JADX_SRC=$(find "$OUT/jadx-out" -name "$(basename "${CLASS##*.}").java" 2>/dev/null | head -1)
fi
if [ -n "${JADX_SRC:-}" ]; then
  mkdir -p "$OUT/jadx-build"
  cp "$JADX_SRC" "$OUT/jadx-build/$CLASS.java"
  cp "$JAR" "$OUT/jadx-build/$JARBASE"
  ( cd "$OUT/jadx-build" && javac --release 8 -cp "$JARBASE" "$CLASS.java" 2>javac.err )
  JX_EXIT=$?
  echo "jadx javac exit=$JX_EXIT" >> "$OUT/legs.txt"
  if [ "$JX_EXIT" -eq 0 ]; then
    ( cd "$OUT/jadx-build" && java -Xverify:all -cp ".:$JARBASE" "$CLASS" ) > "$OUT/jadx.out" 2>&1
    echo "jadx run exit=$?" >> "$OUT/legs.txt"
  else
    echo "jadx behaviour leg=unavailable (JADX text does not compile)" >> "$OUT/legs.txt"
  fi
else
  echo "jadx source=not-produced" >> "$OUT/legs.txt"
fi

# comparison
{
  echo "original stdout sha256: $(shasum -a 256 < "$OUT/original.out" | cut -d' ' -f1)"
  [ -f "$OUT/jarde.out" ] && echo "jarde stdout sha256:    $(shasum -a 256 < "$OUT/jarde.out" | cut -d' ' -f1)"
  [ -f "$OUT/jadx.out" ] && echo "jadx stdout sha256:     $(shasum -a 256 < "$OUT/jadx.out" | cut -d' ' -f1)"
} >> "$OUT/legs.txt"
if [ -f "$OUT/jarde.out" ] && diff -q "$OUT/original.out" "$OUT/jarde.out" >/dev/null; then
  echo "VERDICT jarde-stdout==original-stdout" >> "$OUT/legs.txt"
else
  echo "VERDICT jarde-stdout!=original-stdout" >> "$OUT/legs.txt"
fi
if [ -f "$OUT/jadx.out" ]; then
  if diff -q "$OUT/original.out" "$OUT/jadx.out" >/dev/null; then
    echo "VERDICT jadx-stdout==original-stdout" >> "$OUT/legs.txt"
  else
    echo "VERDICT jadx-stdout!=original-stdout" >> "$OUT/legs.txt"
  fi
fi
cat "$OUT/legs.txt"