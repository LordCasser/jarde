#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
HERE="$ROOT/openspec/evidence/java-syntax-2026-09-26/boxed-sam-adaptations"
SRC="$HERE/BoxedSamProbe.java"
JARDE_CLI=${JARDE_CLI:-/tmp/jarde-cli-audit-baseline}
WORK=$(mktemp -d "${TMPDIR:-/tmp}/boxed-sam-replay.XXXXXX")
trap 'find "$WORK" -depth -delete' EXIT HUP INT TERM
mkdir -p "$HERE/v8"
javac --release 8 -g:none -d "$HERE/v8" "$SRC" > /dev/null 2> "$HERE/original-javac.stderr"
(cd "$HERE/v8" && shasum -a 256 BoxedSamProbe.class) > "$HERE/original-class.sha256"
(cd "$HERE" && shasum -a 256 BoxedSamProbe.java) > "$HERE/source.sha256"
javap -v -c -p "$HERE/v8/BoxedSamProbe.class" > "$HERE/javap.txt"
java -Xverify:all -cp "$HERE/v8" BoxedSamProbe > "$HERE/original-runtime.txt"
if command -v jadx >/dev/null 2>&1; then
  jadx --version > "$HERE/jadx-version.txt"
  jadx --no-res -d "$WORK/jadx" "$HERE/v8/BoxedSamProbe.class" > "$HERE/jadx.stdout" 2> /dev/null
  JAVADX_SOURCE=$(find "$WORK/jadx/sources" -name BoxedSamProbe.java -print -quit)
  cp "$JAVADX_SOURCE" "$HERE/jadx.java.txt"
  javac --release 8 -g:none -d "$WORK/jadx-classes" "$JAVADX_SOURCE" > /dev/null 2> "$HERE/jadx-javac.stderr"
  java -Xverify:all -cp "$WORK/jadx-classes" defpackage.BoxedSamProbe > "$HERE/jadx-runtime.txt"
fi
python3 - "$HERE" <<'PY'
from pathlib import Path
import sys
root = Path(sys.argv[1])
for name in ('jarde-cli.sha256', 'jarde.java.txt', 'jarde-summary.txt', 'jarde-javac.stderr', 'jarde-javac.status'):
    (root / name).unlink(missing_ok=True)
PY
if [ -x "$JARDE_CLI" ]; then
  shasum -a 256 "$JARDE_CLI" > "$HERE/jarde-cli.sha256"
  "$JARDE_CLI" class-source --input "$HERE/v8/BoxedSamProbe.class" --class BoxedSamProbe --policy single-class --release 8 --format text > "$HERE/jarde.java.txt" 2> "$WORK/jarde.report.txt"
  printf '0\n' > "$HERE/jarde.status"
  mkdir -p "$WORK/jarde"
  cp "$HERE/jarde.java.txt" "$WORK/jarde/BoxedSamProbe.java"
  if javac --release 8 -g:none -d "$WORK/jarde-classes" "$WORK/jarde/BoxedSamProbe.java" > /dev/null 2> "$HERE/jarde-javac.stderr"; then
    printf '0\n' > "$HERE/jarde-javac.status"
  else
    status=$?
    printf '%s\n' "$status" > "$HERE/jarde-javac.status"
  fi
  python3 - "$WORK/jarde.report.txt" "$HERE/jarde-summary.txt" <<'PY'
from pathlib import Path
import sys
src=Path(sys.argv[1]).read_text().splitlines()
selected=[]
methods={6:'supplier',7:'minimumSupplier',8:'function',9:'arrayCtor',11:'chainedSites'}
for i,name in methods.items():
    prefix=f'methods.{i}.'
    declaration=next((line for line in src if line.startswith(prefix+'declaration = ')),None)
    if declaration: selected.append(f'[{name}] {declaration}')
    for line in src:
        if line.startswith(prefix+'outcome.report.diagnostics.') and ('.code =' in line or '.message =' in line):
            selected.append(line)
        elif line.startswith(prefix+'outcome.report.fallbacks ='):
            selected.append(line)
Path(sys.argv[2]).write_text('\n'.join(selected)+'\n')
PY
else
  printf 'unavailable\n' > "$HERE/jarde.status"
fi
if [ -f "$HERE/jadx-runtime.txt" ]; then
  if cmp -s "$HERE/original-runtime.txt" "$HERE/jadx-runtime.txt"; then
    printf 'equal\n' > "$HERE/runtime-comparison.txt"
  else
    printf 'different\n' > "$HERE/runtime-comparison.txt"
  fi
fi
java -version > "$HERE/java-version.txt" 2>&1
javac -version > "$HERE/javac-version.txt" 2>&1
