#!/usr/bin/env bash
set -u
if [[ $# -lt 1 || $# -gt 2 ]]; then
  echo "usage: $0 JARDE_CLI [OUTPUT_DIR]" >&2
  exit 2
fi
HERE="$(cd "$(dirname "$0")" && pwd)"
BASE="$(cd "$HERE/.." && pwd)"
ROOT="$(git -C "$BASE" rev-parse --show-toplevel)"
CLI="$(cd "$(dirname "$1")" && pwd)/$(basename "$1")"
if [[ $# -eq 2 ]]; then OUT="$2"; else OUT="$(mktemp -d /tmp/cf16-test13-neighbors.XXXXXX)"; fi
mkdir -p "$OUT"
TMP="$(mktemp -d /tmp/cf16-test13-neighbors-work.XXXXXX)"
trap 'find "$TMP" -depth -delete' EXIT
JAVAC="${JAVAC:-$(command -v javac)}"
JAVA="${JAVA:-$(command -v java)}"
JAVAP="${JAVAP:-$(command -v javap)}"
JADX="${JADX:-/Users/lordcasser/workspace/testzone/jadx/jadx-cli/build/install/jadx/bin/jadx}"
CLASS='jadx.tests.integration.trycatch.TestTryCatchFinally13$TestCls'
EXPECTED_JADX_HEAD=2fb1b16386941660fda07e9017285aec40fcb37f
RUNNER=jadx.tests.integration.trycatch.NegativeRunner
BASE_CLASS="$BASE/TestTryCatchFinally13\$TestCls.probe.class"
JARDE_SOURCE_HEAD="${JARDE_SOURCE_HEAD:-$(git -C "$ROOT" rev-parse HEAD)}"

if [[ ! -x "$CLI" || ! -x "$JADX" ]]; then echo "Jarde CLI or JADX launcher is not executable" >&2; exit 3; fi
if [[ "$(git -C /Users/lordcasser/workspace/testzone/jadx rev-parse HEAD)" != "$EXPECTED_JADX_HEAD" ]]; then echo "pinned JADX checkout HEAD mismatch" >&2; exit 3; fi
if [[ "$("$JADX" --version 2>&1 | head -n 1)" != dev ]]; then echo "pinned JADX launcher is not dev" >&2; exit 3; fi
mkdir -p "$TMP/template" "$TMP/variants" "$TMP/jadx"
"$JAVAC" --release 8 -g -d "$TMP/template" "$HERE/TestTryCatchFinally13\$TestCls.java" "$HERE/NegativeRunner.java" > "$OUT/template-javac.stdout" 2> "$OUT/template-javac.stderr"
if [[ $? -ne 0 ]]; then echo "variant template compilation failed" >&2; exit 4; fi
TEMPLATE_CLASS="$TMP/template/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class"
cp "$TEMPLATE_CLASS" "$OUT/template.class"
javap -classpath "$TMP/template" -c -v "$CLASS" > "$TMP/template.full.javap.txt"
python3 - "$BASE/probe-class.javap.txt" "$TMP/template.full.javap.txt" > "$OUT/template-baseline-shape.txt" <<'PYSHAPE'
import re, sys
from pathlib import Path

def shape(path):
    lines=Path(path).read_text().splitlines()
    start=next(i for i,line in enumerate(lines) if re.match(r"\s+public void test\(int\);",line))
    code=[]; table=[]; in_table=False
    for line in lines[start:]:
        if "Exception table:" in line: in_table=True; continue
        if in_table:
            m=re.match(r"\s*(\d+)\s+(\d+)\s+(\d+)\s+(.+?)\s*$",line)
            if m: table.append(tuple(m.groups()))
            elif table: break
            continue
        m=re.match(r"\s*(\d+):\s+([a-z][a-z0-9_]*)",line)
        if m: code.append((int(m.group(1)),m.group(2)))
    return code,table
base, template=shape(sys.argv[1]),shape(sys.argv[2])
if base != template: raise SystemExit(f"instrumented helper template changed test shape: {base} != {template}")
print("template_vs_frozen_probe_bci_and_opcode_equal=true")
print("template_vs_frozen_probe_exception_table_equal=true")
PYSHAPE
python3 - "$BASE/TestTryCatchFinally13.fixed-source.java" "$HERE/TestTryCatchFinally13\$TestCls.java" <<'PY'
import sys
from pathlib import Path

def body(path):
    text=Path(path).read_text(); start=text.index("public void test(int i) {"); brace=text.index("{",start); depth=0
    for end in range(brace,len(text)):
        if text[end]=="{": depth+=1
        elif text[end]=="}":
            depth-=1
            if depth==0:return "".join(text[start:end+1].split())
    raise SystemExit(f"unterminated test method in {path}")
if body(sys.argv[1]) != body(sys.argv[2]): raise SystemExit("negative template test(I)V differs from fixed source")
PY

# Produce independent class copies and apply one local mutation to each.
for kind in cleanup-target branch-bypass range-expanded rethrow-changed; do
  mkdir -p "$TMP/variants/$kind"
  cp -R "$TMP/template/." "$TMP/variants/$kind/"
  python3 "$HERE/mutate_class.py" "$TMP/variants/$kind/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class" "$TMP/variants/$kind/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class" "$kind" > "$OUT/$kind.mutation.txt"
  cp "$TMP/variants/$kind/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class" "$OUT/$kind.class"
done
cp -R "$TMP/template/." "$TMP/variants/range-control/"
cp "$TMP/variants/range-control/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class" "$OUT/range-control.class"

python3 - "$TMP" "$OUT" <<'PY'
import re, subprocess, sys
from pathlib import Path
work, out = map(Path,sys.argv[1:])
variants={"cleanup-target":"-12","branch-bypass":"0","range-expanded":"-12","range-control":"-12","rethrow-changed":"0"}
for name in variants:
    cp=work/"variants"/name
    full=subprocess.run(["javap","-classpath",str(cp),"-c","-v","jadx.tests.integration.trycatch.TestTryCatchFinally13$TestCls"],text=True,stdout=subprocess.PIPE,stderr=subprocess.STDOUT,check=True).stdout
    lines=full.splitlines(); start=next(i for i,line in enumerate(lines) if re.match(r"\s+public void test\(int\);",line))
    end=next(i for i in range(start+1,len(lines)) if "LineNumberTable:" in lines[i])
    (out/f"{name}.test.javap.txt").write_text("\n".join(lines[start:end+1])+"\n")
PY

JADX_HEAD="$(git -C /Users/lordcasser/workspace/testzone/jadx rev-parse HEAD)"
JADX_VERSION="$("$JADX" --version 2>&1 | head -n 1)"
JARDE_VERSION="$("$CLI" --version 2>&1 | tail -n 1)"
JARDE_SHA="$(shasum -a 256 "$CLI" | awk '{print $1}')"
JADX_SHA="$(shasum -a 256 "$JADX" | awk '{print $1}')"
printf 'jarde_source_head=%s\njarde_cli=%s\njarde_cli_version=%s\njarde_cli_sha256=%s\njadx_head=%s\njadx_version=%s\njadx_launcher_sha256=%s\nfixed_baseline_head=dd35384c204f960e4be61b7d47adf3350f96fc71\n' \
  "$JARDE_SOURCE_HEAD" "$CLI" "$JARDE_VERSION" "$JARDE_SHA" "$JADX_HEAD" "$JADX_VERSION" "$JADX_SHA" > "$OUT/toolchain.txt"

: > "$OUT/results.txt"
for kind in cleanup-target branch-bypass range-expanded range-control rethrow-changed; do
  cp="$TMP/variants/$kind"
  scenario="$kind"
  "$JAVA" -Xverify:all -cp "$cp" "$RUNNER" "$scenario" > "$OUT/$kind.original.run.txt" 2>&1
  verify_status=$?
  jadx_status=not_run; jadx_compile=not_run; jadx_run=not_run; jarde_status=not_run; jarde_compile=not_run; jarde_run=not_run; jarde_full_refusal=false; jarde_incomplete_marker=false

  jadx_out="$TMP/jadx/$kind"
  mkdir -p "$jadx_out" "$TMP/jadx-classes/$kind"
  "$JADX" --no-res -d "$jadx_out" "$cp/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class" > "$OUT/$kind.jadx.stdout" 2> "$OUT/$kind.jadx.stderr"
  jadx_status=$?
  jadx_src="$jadx_out/sources/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.java"
  if [[ $jadx_status -eq 0 && -f "$jadx_src" ]]; then
    cp "$jadx_src" "$OUT/$kind.jadx.java"
    "$JAVAC" --release 8 -g -d "$TMP/jadx-classes/$kind" "$jadx_src" "$HERE/NegativeRunner.java" > "$OUT/$kind.jadx-javac.stdout" 2> "$OUT/$kind.jadx-javac.stderr"
    jadx_compile=$?
    if [[ $jadx_compile -eq 0 ]]; then
      "$JAVA" -Xverify:all -cp "$TMP/jadx-classes/$kind" "$RUNNER" "$scenario" > "$OUT/$kind.jadx.run.txt" 2>&1
      jadx_run=$?
    fi
  fi

  "$CLI" class-source --input "$cp/jadx/tests/integration/trycatch/TestTryCatchFinally13\$TestCls.class" --class "$CLASS" --policy single-class --release 8 --format text > "$OUT/$kind.jarde.java.txt" 2> "$TMP/$kind.jarde.report.txt"
  jarde_status=$?
  mkdir -p "$TMP/jarde-classes/$kind"
  if [[ $jarde_status -eq 0 ]]; then
    if grep -Fq 'jarde: not recovered: the recovery run for `test(I)V` produced no statement' "$OUT/$kind.jarde.java.txt"; then jarde_full_refusal=true; fi
    if [[ "$jarde_full_refusal" == true ]] || grep -Fq 'not part of the recoverable subset' "$OUT/$kind.jarde.java.txt"; then jarde_incomplete_marker=true; fi
    cp "$OUT/$kind.jarde.java.txt" "$TMP/jarde-classes/$kind/TestTryCatchFinally13\$TestCls.java"
    "$JAVAC" --release 8 -g -d "$TMP/jarde-classes/$kind" "$TMP/jarde-classes/$kind/TestTryCatchFinally13\$TestCls.java" "$HERE/NegativeRunner.java" > "$OUT/$kind.jarde-javac.stdout" 2> "$OUT/$kind.jarde-javac.stderr"
    jarde_compile=$?
    if [[ $jarde_compile -eq 0 ]]; then
      "$JAVA" -Xverify:all -cp "$TMP/jarde-classes/$kind" "$RUNNER" "$scenario" > "$OUT/$kind.jarde.run.txt" 2>&1
      jarde_run=$?
    fi
  fi
  if [[ "$kind" == range-control && "${JARDE_RANGE_CONTROL_RECOVERED:-false}" == true ]]; then
    if [[ "$verify_status" != 0 || "$jarde_incomplete_marker" != false ]] || ! grep -Fq 'finally {' "$OUT/$kind.jarde.java.txt"; then
      echo "range-control failed verifier-run or its recovered finally is incomplete" >&2
      exit 5
    fi
  elif [[ "$verify_status" != 0 || "$jarde_incomplete_marker" != true ]] || grep -Fq 'finally {' "$OUT/$kind.jarde.java.txt"; then
    echo "$kind failed verifier-run or lacks a Jarde incompleteness marker" >&2
    exit 5
  fi
  if [[ "$kind" == range-expanded ]]; then
    cmp "$OUT/range-expanded.original.run.txt" "$HERE/range-expanded.original.run.txt" || exit 5
  fi
  printf '%s_verifier_runner_exit=%s\n%s_jadx_decompile_exit=%s\n%s_jadx_javac_release8_exit=%s\n%s_jadx_runner_exit=%s\n%s_jarde_source_exit=%s\n%s_jarde_full_method_refusal=%s\n%s_jarde_incomplete_method_marker=%s\n%s_jarde_javac_release8_exit=%s\n%s_jarde_runner_exit=%s\n' \
    "$kind" "$verify_status" "$kind" "$jadx_status" "$kind" "$jadx_compile" "$kind" "$jadx_run" "$kind" "$jarde_status" "$kind" "$jarde_full_refusal" "$kind" "$jarde_incomplete_marker" "$kind" "$jarde_compile" "$kind" "$jarde_run" >> "$OUT/results.txt"
done
printf 'output_dir=%s\n' "$OUT"
