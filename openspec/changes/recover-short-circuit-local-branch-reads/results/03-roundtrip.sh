#!/bin/sh
# The behavior leg, as a transcript: render, strip, compile on both javac legs, run under
# `-Xverify:all`, compare with the frozen classes' own answers.
#
# Two inputs:
#   * the patrol's `OP2` — the recovered class's own `main` is a registered residual of the
#     concatenation chain's saved-producer family, so the one `jarde_refused_body();` marker is
#     replaced by the **frozen source's own** `main` body (asserted present in that source) before
#     compiling; the line the run prints is the recovered methods' answers;
#   * `BranchReads` — the whole class recovers, so its own `main` is what is measured.
#
# Usage: 03-roundtrip.sh [cli]
#   default: /tmp/brslice/bin/jarde-cli.current (this worktree's patched binary)
#
# Output: 03-roundtrip.out, beside this script's directory.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
CLI=${1:-/tmp/brslice/bin/jarde-cli.current}
OUT="$HERE/03-roundtrip.out"
WORK=/tmp/brslice/roundtrip
OP2_JAR="$ROOT/openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar"
OP2_SOURCE="$ROOT/openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/OP2.java"
FIXTURES="$ROOT/tests/fixtures/recover-short-circuit-local-branch-reads"
JAVAC8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac
JAVA8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/java

[ -x "$CLI" ] || { echo "no CLI at $CLI" >&2; exit 2; }
rm -rf "$WORK"
mkdir -p "$WORK"

{
    echo "# The behavior leg (change recover-short-circuit-local-branch-reads), $(date -u +%Y-%m-%dT%H:%M:%SZ)"
    echo "# cli: $CLI"
    echo
} >"$OUT"

# One render, with the layer's own self-header asserted before anything is done with it.
render() {
    # $1 = input, $2 = policy, $3 = class, $4 = output
    "$CLI" class-source --input "$1" --policy "$2" --class "$3" --format text >"$4" 2>/dev/null
    head -1 "$4" | grep -q '// jarde: presentation of' ||
        { echo "SELF-TEST FAILED: no presentation header for $3" >&2; exit 3; }
    # The comment-stripped text, the way the patrols' own stripped sources were made.
    grep -v '^[[:space:]]*//' "$4" >"$4.stripped"
}

# ---- OP2: the frozen jar's class, and the recovered class with the residual's body restored ----
mkdir -p "$WORK/op2-original"
( cd "$WORK/op2-original" && unzip -o -q "$OP2_JAR" OP2.class )
render "$OP2_JAR" plain-jar OP2 "$WORK/OP2.presentation.java"
python3 - "$WORK/OP2.presentation.java.stripped" "$OP2_SOURCE" "$WORK/OP2.java" <<'PY'
import pathlib
import re
import sys

stripped = pathlib.Path(sys.argv[1]).read_text()
source = pathlib.Path(sys.argv[2]).read_text()
driver = re.search(r"public static void main\(String\[\] a\)\{ (.*) \}", source).group(1)
assert source.count(driver) == 1, "the frozen source states the driver exactly once"
assert stripped.count("jarde_refused_body();") == 1, "exactly one refused body (the residual `main`)"
pathlib.Path(sys.argv[3]).write_text(stripped.replace("jarde_refused_body();", driver))
print("OP2: the residual `main` body substituted from the frozen source (not a transcription)")
PY
mkdir -p "$WORK/op2-recovered"
( cd "$WORK/op2-recovered" && javac --release 8 -nowarn -d . "$WORK/OP2.java" ) || {
    echo "OP2: javac 23 --release 8 FAILED"; exit 1; }
( cd "$WORK/op2-recovered" && "$JAVAC8" -nowarn -d . "$WORK/OP2.java" ) || {
    echo "OP2: javac 8 FAILED"; exit 1; }
orig_op2=$( ( cd "$WORK/op2-original" && java -Xverify:all -cp . OP2 ) )
rec_op2=$( ( cd "$WORK/op2-recovered" && java -Xverify:all -cp . OP2 ) )
{
    echo "== OP2 =="
    echo "original  (frozen jar):  $orig_op2"
    echo "recovered (javac 23/8):  $rec_op2"
    [ "$orig_op2" = "$rec_op2" ] && echo "verdict: identical (both legs compiled; condAssignOld(0) included)" ||
        echo "verdict: MISMATCH"
    echo
} >>"$OUT"

# ---- BranchReads: the whole class recovers, both legs ----
for leg in v8 v8-javac8; do
    mkdir -p "$WORK/reads-original-$leg" "$WORK/reads-recovered-$leg"
    cp "$FIXTURES/$leg/BranchReads.class" "$WORK/reads-original-$leg/"
    render "$FIXTURES/$leg/BranchReads.class" single-class BranchReads \
        "$WORK/BranchReads-$leg.presentation.java"
    cp "$WORK/BranchReads-$leg.presentation.java.stripped" \
        "$WORK/reads-recovered-$leg/BranchReads.java"
    if [ "$leg" = v8 ]; then
        ( cd "$WORK/reads-recovered-$leg" && javac --release 8 -nowarn -d . BranchReads.java ) || {
            echo "BranchReads($leg): javac 23 --release 8 FAILED"; exit 1; }
        orig_reads=$( ( cd "$WORK/reads-original-$leg" && java -Xverify:all -cp . BranchReads ) )
        rec_reads=$( ( cd "$WORK/reads-recovered-$leg" && java -Xverify:all -cp . BranchReads ) )
        {
            echo "== BranchReads ($leg, javac 23.0.1 --release 8) =="
            echo "original:  $(printf '%s' "$orig_reads" | tr '\n' ' ')"
            echo "recovered: $(printf '%s' "$rec_reads" | tr '\n' ' ')"
            [ "$orig_reads" = "$rec_reads" ] && echo "verdict: identical" || echo "verdict: MISMATCH"
            echo
        } >>"$OUT"
    else
        ( cd "$WORK/reads-recovered-$leg" && "$JAVAC8" -nowarn -d . BranchReads.java ) || {
            echo "BranchReads($leg): javac 8 FAILED"; exit 1; }
        orig_reads=$( ( cd "$WORK/reads-original-$leg" && "$JAVA8" -Xverify:all -cp . BranchReads ) )
        rec_reads=$( ( cd "$WORK/reads-recovered-$leg" && "$JAVA8" -Xverify:all -cp . BranchReads ) )
        {
            echo "== BranchReads ($leg, Corretto 1.8.0_432) =="
            echo "original:  $(printf '%s' "$orig_reads" | tr '\n' ' ')"
            echo "recovered: $(printf '%s' "$rec_reads" | tr '\n' ' ')"
            [ "$orig_reads" = "$rec_reads" ] && echo "verdict: identical" || echo "verdict: MISMATCH"
            echo
        } >>"$OUT"
    fi
done

# ---- The boundaries: the text is the safe form (it must not compile) ----
render "$FIXTURES/v8/BranchReadNegatives.class" single-class BranchReadNegatives \
    "$WORK/BranchReadNegatives.presentation.java"
mkdir -p "$WORK/negatives"
cp "$WORK/BranchReadNegatives.presentation.java.stripped" "$WORK/negatives/BranchReadNegatives.java"
{
    echo "== BranchReadNegatives (the boundaries) =="
    if ( cd "$WORK/negatives" && javac --release 8 -nowarn -d . BranchReadNegatives.java ) 2>/dev/null; then
        echo "verdict: COMPILED — the boundaries' text must not compile"
    else
        echo "verdict: does not compile, as the safe direction requires"
    fi
} >>"$OUT"

cat "$OUT"
