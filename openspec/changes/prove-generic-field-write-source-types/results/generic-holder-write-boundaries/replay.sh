#!/bin/bash
ROOT="$(cd "$(dirname "$0")/../../../../.." && pwd)"
EVIDENCE="$ROOT/openspec/evidence/generic-holder-write-boundaries"
JDK8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home
JDK23=/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home
JADX=/opt/homebrew/bin/jadx
JARDE=/tmp/jarde-generic-baseline-cli
RUNLOG="$ROOT/openspec/changes/prove-generic-field-write-source-types/results/generic-holder-write-boundaries"
mkdir -p "$RUNLOG"
cp "$RUNLOG/ReflectDriver.java" "$EVIDENCE/ReflectDriver.java"
TMP="$(mktemp -d)"
trap 'rm -rf "$TMP"' EXIT
: > "$RUNLOG/status.tsv"
: > "$RUNLOG/focus-compile.tsv"
for leg in javac8 javac23; do
  if [ "$leg" = javac8 ]; then JDK="$JDK8"; else JDK="$JDK23"; fi
  DRIVER="$TMP/$leg-driver"
  mkdir -p "$DRIVER"
  if [ "$leg" = javac8 ]; then
    "$JDK/bin/javac" -source 8 -target 8 -g:none -d "$DRIVER" "$RUNLOG/ReflectDriver.java" >"$RUNLOG/$leg-driver-javac.stdout" 2>"$RUNLOG/$leg-driver-javac.stderr"
  else
    "$JDK/bin/javac" --release 8 -g:none -d "$DRIVER" "$RUNLOG/ReflectDriver.java" >"$RUNLOG/$leg-driver-javac.stdout" 2>"$RUNLOG/$leg-driver-javac.stderr"
  fi
  driver_rc=$?
  printf '%s\tdriver_javac=%s\n' "$leg" "$driver_rc" >> "$RUNLOG/status.tsv"
  focus_success=0
  focus_total=0
  for srcdir in "$EVIDENCE"/*/source; do
    [ -d "$srcdir" ] || continue
    name="$(basename "$(dirname "$srcdir")")"
    out="$EVIDENCE/$leg/$name"
    rm -rf "$out"
    mkdir -p "$out/source" "$out/classes" "$out/jadx" "$out/recompiled" "$out/jarde"
    cp "$srcdir/$name.java" "$out/source/"
    if [ "$leg" = javac8 ]; then
      "$JDK/bin/javac" -source 8 -target 8 -g:none -d "$out/classes" "$out/source/$name.java" >"$out/javac.stdout" 2>"$out/javac.stderr"
    else
      "$JDK/bin/javac" --release 8 -g:none -d "$out/classes" "$out/source/$name.java" >"$out/javac.stdout" 2>"$out/javac.stderr"
    fi
    rc=$?
    printf '%s\t%s\traw_javac=%s\n' "$leg" "$name" "$rc" >> "$RUNLOG/status.tsv"
    [ "$rc" -eq 0 ] || continue
    (cd "$out/classes" && "$JDK/bin/jar" cf "$out/$name.jar" "$name.class")
    "$JDK/bin/jar" tf "$out/$name.jar" > "$out/jar.contents"
    python3 - "$out/jar.contents" "$name" <<'PYJAR'
from pathlib import Path
import sys
entries=[x for x in Path(sys.argv[1]).read_text().splitlines() if x.endswith('.class')]
assert entries==[sys.argv[2]+'.class'], entries
PYJAR
    "$JDK/bin/javap" -classpath "$out/$name.jar" -p -v -c "$name" > "$out/$name.javap" 2>&1
    "$JADX" -d "$out/jadx" "$out/$name.jar" > "$out/jadx.stdout" 2> "$out/jadx.stderr"
    jrc=$?
    rm -rf "$out/jadx/resources"
    if [ "$jrc" -eq 0 ]; then
      python3 - "$out/jadx/sources" "$name" <<'PYJADX'
from pathlib import Path
import sys
files=list(Path(sys.argv[1]).rglob('*.java'))
assert len(files)==1 and files[0].name==sys.argv[2]+'.java', [str(x) for x in files]
PYJADX
    fi
    printf '%s\t%s\tjadx=%s\n' "$leg" "$name" "$jrc" >> "$RUNLOG/status.tsv"

    case "$name" in
      Hold|ObjectHold) ctor=ctor; methods=() ;;
      SCGA|SCGB) ctor=default; methods=(main) ;;
      NullSetter) ctor=default; methods=(put clear) ;;
      MixedSetter) ctor=default; methods=(putT putObject) ;;
      ObjectSetter|CrossSetter|TypedSetter|ShadowSetter|ArraySetter|ArrayObjectSetter|RawListField|DeferredSetter) ctor=default; methods=(put) ;;
      *) ctor=default; methods=() ;;
    esac
    "$JDK/bin/java" -Xverify:all -cp "$DRIVER:$out/classes" ReflectDriver "$name" "$ctor" plain "${methods[@]}" > "$out/original.run.stdout" 2> "$out/original.run.stderr"
    original_rc=$?
    printf '%s\t%s\toriginal_run=%s\n' "$leg" "$name" "$original_rc" >> "$RUNLOG/status.tsv"

    if [ "$driver_rc" -eq 0 ] && [ "$jrc" -eq 0 ]; then
      python3 - "$out/jadx/sources" "$out/recompiled/sources" <<'PYCP'
from pathlib import Path
import shutil, sys
src=Path(sys.argv[1]); dst=Path(sys.argv[2])
for f in src.rglob('*.java'):
    target=dst/f.relative_to(src); target.parent.mkdir(parents=True, exist_ok=True); shutil.copy2(f,target)
PYCP
      mkdir -p "$TMP/$leg-$name-jadx-classes"
      find "$out/recompiled/sources" -name '*.java' -print0 | xargs -0 "$JDK/bin/javac" -g:none -d "$TMP/$leg-$name-jadx-classes" > "$out/recompiled/javac.stdout" 2> "$out/recompiled/javac.stderr"
      crc=$?
      printf '%s\t%s\tjadx_javac=%s\n' "$leg" "$name" "$crc" >> "$RUNLOG/status.tsv"
      if [ "$crc" -eq 0 ]; then
        "$JDK/bin/java" -Xverify:all -cp "$DRIVER:$TMP/$leg-$name-jadx-classes" ReflectDriver "$name" "$ctor" defpackage "${methods[@]}" > "$out/recompiled/run.stdout" 2> "$out/recompiled/run.stderr"
        rrc=$?
        diff -u "$out/original.run.stdout" "$out/recompiled/run.stdout" > "$out/behavior.diff"
        drc=$?
        printf '%s\t%s\tjadx_run=%s\tstdout_diff=%s\n' "$leg" "$name" "$rrc" "$drc" >> "$RUNLOG/status.tsv"
      fi
      rm -rf "$TMP/$leg-$name-jadx-classes"
    fi

    if [ "$driver_rc" -eq 0 ]; then
      "$JARDE" class-source --input "$out/$name.jar" --class "$name" --format text --output "$out/jarde/$name.java" > "$out/jarde/class-source.stdout" 2> "$out/jarde/class-source.stderr"
      jarde_rc=$?
      if [ "$jarde_rc" -eq 0 ]; then
        python3 - "$out/jarde/$name.java" "$name" <<'PYASSERT'
from pathlib import Path
import re, sys
source=Path(sys.argv[1]).read_text()
name=sys.argv[2]
assert source.strip(), 'empty class-source output'
assert re.search(r'\bclass\s+'+re.escape(name)+r'\b', source), 'missing nonempty class-source header'
PYASSERT
        header_rc=$?
      else
        header_rc=1
      fi
      printf '%s\t%s\tjarde=%s\theader_assert=%s\n' "$leg" "$name" "$jarde_rc" "$header_rc" >> "$RUNLOG/status.tsv"
      mkdir -p "$TMP/$leg-$name-jarde-classes"
      if [ "$leg" = javac8 ]; then
        "$JDK/bin/javac" -source 8 -target 8 -g:none -d "$TMP/$leg-$name-jarde-classes" "$out/jarde/$name.java" > "$out/jarde/javac.stdout" 2> "$out/jarde/javac.stderr"
      else
        "$JDK/bin/javac" --release 8 -g:none -d "$TMP/$leg-$name-jarde-classes" "$out/jarde/$name.java" > "$out/jarde/javac.stdout" 2> "$out/jarde/javac.stderr"
      fi
      jcrc=$?
      printf '%s\t%s\tjarde_javac=%s\n' "$leg" "$name" "$jcrc" >> "$RUNLOG/status.tsv"
      if [ "$jcrc" -eq 0 ]; then
        "$JDK/bin/java" -Xverify:all -cp "$DRIVER:$TMP/$leg-$name-jarde-classes" ReflectDriver "$name" "$ctor" plain "${methods[@]}" > "$out/jarde/run.stdout" 2> "$out/jarde/run.stderr"
        jrun=$?
        diff -u "$out/original.run.stdout" "$out/jarde/run.stdout" > "$out/jarde/behavior.diff"
        jdiff=$?
        printf '%s\t%s\tjarde_run=%s\tstdout_diff=%s\n' "$leg" "$name" "$jrun" "$jdiff" >> "$RUNLOG/status.tsv"
      fi
      rm -rf "$TMP/$leg-$name-jarde-classes"
      case "$name" in Hold|ObjectHold|ObjectSetter|CrossSetter)
        focus_total=$((focus_total+1))
        [ "$jcrc" -eq 0 ] && focus_success=$((focus_success+1))
        printf '%s\t%s\tcompile=%s\n' "$leg" "$name" "$jcrc" >> "$RUNLOG/focus-compile.tsv"
        ;;
      esac
    fi
  done
  printf '%s\tfocus_compile_success=%s\tfocus_total=%s\n' "$leg" "$focus_success" "$focus_total" >> "$RUNLOG/focus-compile.tsv"
done

python3 - "$RUNLOG/focus-compile.tsv" <<'PYFOCUS'
from pathlib import Path
import sys
rows=Path(sys.argv[1]).read_text().splitlines()
for leg in ('javac8','javac23'):
    subset=[line for line in rows if line.startswith(leg+'\t')]
    assert len([line for line in subset if '\tcompile=' in line])==4, subset
    summary=[line for line in subset if 'focus_compile_success=' in line]
    assert len(summary)==1 and 'focus_compile_success=0\tfocus_total=4' in summary[0], summary
PYFOCUS
