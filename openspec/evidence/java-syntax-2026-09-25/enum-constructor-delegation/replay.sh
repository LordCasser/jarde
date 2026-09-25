#!/bin/sh
set -eu

HERE=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
CLI=${JARDE_CLI:-/tmp/jarde-generic-accepted-cli}
JADX=${JADX:-/opt/homebrew/bin/jadx}
EXPECTED_CLI_SHA=ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145
ACTUAL_CLI_SHA=$(shasum -a 256 "$CLI" | awk '{print $1}')
[ "$ACTUAL_CLI_SHA" = "$EXPECTED_CLI_SHA" ] || { echo "unexpected CLI sha256: $ACTUAL_CLI_SHA" >&2; exit 1; }
command -v javac >/dev/null
command -v java >/dev/null
command -v javap >/dev/null
[ -x "$JADX" ] || { echo "missing JADX executable: $JADX" >&2; exit 1; }

TMP=$(mktemp -d "${TMPDIR:-/tmp}/jarde-enum-ctor.XXXXXX")
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
rm -f "$HERE"/original-run-*.txt "$HERE"/jadx-run-*.txt "$HERE"/jarde-run-*.txt \
  "$HERE"/jarde-run-status-*.txt "$HERE"/jarde-javac-*.status "$HERE"/jarde-javac-*.txt
rm -f "$HERE"/javap-*.txt "$HERE"/classfile-sha256.txt "$HERE"/tool-versions.txt
rm -f "$HERE"/decompiled/*.java "$HERE"/SHA256SUMS.txt

{
  javac -version 2>&1
  java -version 2>&1 | head -n 1
  "$JADX" --version
  printf 'jarde-cli sha256 %s\n' "$ACTUAL_CLI_SHA"
} > "$HERE/tool-versions.txt"
: > "$HERE/classfile-sha256.txt"
EXPECTED='values=ZERO:0,ONE:1
effects=2:0,1
declared-constructors=2,3'
printf "$EXPECTED\n" > "$TMP/expected.txt"

for DEBUG in g g-none; do
  OUT="$TMP/$DEBUG"
  mkdir -p "$OUT/classes"
  if [ "$DEBUG" = g ]; then OPT=-g; else OPT=-g:none; fi
  javac --release 8 -Xlint:-options "$OPT" -d "$OUT/classes" "$HERE"/source/*.java
  java -Xverify:all -cp "$OUT/classes" EnumRunner > "$HERE/original-run-$DEBUG.txt"
  cmp "$TMP/expected.txt" "$HERE/original-run-$DEBUG.txt"
  javap -v -c -p -classpath "$OUT/classes" DelegatingEnum > "$HERE/javap-$DEBUG.txt"
  (cd "$OUT/classes" && shasum -a 256 *.class) | sed "s#  #  $DEBUG/#" >> "$HERE/classfile-sha256.txt"
  JAR="$OUT/input.jar"
  (cd "$OUT/classes" && jar cf "$JAR" *.class)

  JOUT="$OUT/jadx"
  "$JADX" -d "$JOUT" "$JAR" > "$OUT/jadx.stdout" 2> "$OUT/jadx.stderr"
  JENUM=$(find "$JOUT/sources" -name DelegatingEnum.java -print -quit)
  [ -n "$JENUM" ] || { echo "JADX enum source missing" >&2; exit 1; }
  cp "$JENUM" "$HERE/decompiled/jadx-DelegatingEnum-$DEBUG.java"
  mkdir -p "$OUT/jadx-classes"
  find "$JOUT/sources" -name '*.java' -print0 | xargs -0 javac --release 8 -Xlint:-options -d "$OUT/jadx-classes"
  java -Xverify:all -cp "$OUT/jadx-classes" defpackage.EnumRunner > "$HERE/jadx-run-$DEBUG.txt"
  cmp "$TMP/expected.txt" "$HERE/jadx-run-$DEBUG.txt"

  REPORT="$OUT/jarde.json"
  "$CLI" class-source --input "$JAR" --class DelegatingEnum --format json --output "$REPORT"
  python3 - "$REPORT" "$HERE/decompiled/jarde-DelegatingEnum-$DEBUG.java" <<'PY'
import json, pathlib, sys
report = json.loads(pathlib.Path(sys.argv[1]).read_text())
pathlib.Path(sys.argv[2]).write_text(report["text"])
PY
  mkdir -p "$OUT/jarde-classes"
  set +e
  javac --release 8 -Xlint:-options -cp "$JAR" -d "$OUT/jarde-classes" \
    "$HERE/decompiled/jarde-DelegatingEnum-$DEBUG.java" \
    "$HERE/source/ConstructorEffects.java" "$HERE/source/EnumRunner.java" \
    > "$OUT/jarde-javac.stdout" 2> "$OUT/jarde-javac.stderr"
  JARDE_JAVAC_STATUS=$?
  set -e
  printf '%s\n' "$JARDE_JAVAC_STATUS" > "$HERE/jarde-javac-$DEBUG.status"
  cat "$OUT/jarde-javac.stdout" "$OUT/jarde-javac.stderr" \
    | sed "s#$HERE#<EVIDENCE>#g; s#$TMP#<TMP>#g" > "$HERE/jarde-javac-$DEBUG.txt"
  if [ "$JARDE_JAVAC_STATUS" -eq 0 ]; then
    java -Xverify:all -cp "$OUT/jarde-classes:$OUT/classes" EnumRunner > "$HERE/jarde-run-$DEBUG.txt"
    cmp "$TMP/expected.txt" "$HERE/jarde-run-$DEBUG.txt"
    printf 'ran; java -Xverify:all exit=0\n' > "$HERE/jarde-run-status-$DEBUG.txt"
  else
    [ "$JARDE_JAVAC_STATUS" -eq 1 ] || { echo "unexpected Jarde javac status $JARDE_JAVAC_STATUS" >&2; exit 1; }
    [ ! -e "$OUT/jarde-classes/DelegatingEnum.class" ]
    printf 'not run; Java 8 recompilation failed with exit=%s\n' "$JARDE_JAVAC_STATUS" > "$HERE/jarde-run-status-$DEBUG.txt"
  fi
done

# Hash only generated, reviewable text; class files and build directories stay under TMP.
(cd "$HERE" && find source decompiled -type f -name '*.java' -print | sort \
  | while IFS= read -r file; do shasum -a 256 "$file"; done) > "$HERE/SHA256SUMS.txt"
