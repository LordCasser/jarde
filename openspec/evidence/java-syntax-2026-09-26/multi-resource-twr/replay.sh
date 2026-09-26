#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)
HERE="$ROOT/openspec/evidence/java-syntax-2026-09-26/multi-resource-twr"
SRC="$HERE/MultiResourceTwr.java"
for variant in release8 current; do
  OUT="$HERE/$variant"
  mkdir -p "$OUT"
  if [ "$variant" = release8 ]; then
    javac --release 8 -g:none -d "$OUT" "$SRC" >"$OUT/javac.stdout" 2>"$OUT/javac.stderr"
  else
    javac -g:none -d "$OUT" "$SRC" >"$OUT/javac.stdout" 2>"$OUT/javac.stderr"
  fi
  (cd "$OUT" && shasum -a 256 MultiResourceTwr.class) > "$OUT/class.sha256"
  python3 - "$OUT" <<'PYHASH'
from pathlib import Path
import hashlib, sys
out=Path(sys.argv[1])
rows=[f"{hashlib.sha256(p.read_bytes()).hexdigest()}  {p.name}" for p in sorted(out.glob('*.class'))]
(out/'class-family.sha256').write_text('\n'.join(rows)+'\n')
PYHASH
  javap -v -c -p "$OUT/MultiResourceTwr.class" > "$OUT/javap.tmp" 2>"$OUT/javap.stderr"
  python3 - "$OUT/javap.tmp" "$OUT/run-javap.txt" "$OUT/exception-tables.txt" <<'PY'
from pathlib import Path
import sys
s=Path(sys.argv[1]).read_text()
start=s.index('  private static int run(')
end=s.index('  public static void main(', start)
body=s[start:end]
Path(sys.argv[2]).write_text(body.rstrip() + '\n')
t=body[body.index('      Exception table:'):]
t=t[:t.index('      StackMapTable:')]
Path(sys.argv[3]).write_text(t)
Path(sys.argv[1]).unlink()
PY
  : > "$OUT/runtime.txt"
  for mode in normal body inner-close outer-close suppressed; do
    printf '%s ' "$mode" >> "$OUT/runtime.txt"
    java -Xverify:all -cp "$OUT" MultiResourceTwr "$mode" >> "$OUT/runtime.txt" 2>&1
  done
done
