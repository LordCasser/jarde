#!/bin/sh
# 2.3: the three-way acceptance with the new CLI. For each legal shape the original class, the
# frozen JADX render and the CLI's recovered text are compiled with `javac --release 8` **and** the
# real javac 8, run under `java -Xverify:all`, and their traces compared (normal and exceptional
# inputs, an escaping `Error` included). The refused shape is not counted as passing Java: the CLI
# text of a class with a refused member must stay uncompilable, and the refusals' own acceptance is
# the bytecode/origin coverage the test file pins.
#
# Self-tests before counting: every CLI render starts with the layer's own header, and each leg's
# javac/java must exist. Usage: 03-three-way.sh [cli]
set -eu
HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../../.." && pwd)
CLI=${1:-$ROOT/target/debug/jarde-cli}
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-three-way.XXXXXX")
trap 'rm -rf "$WORK"' EXIT
OUT="$HERE/03-three-way.out"
C8=/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home

[ -x "$CLI" ] || { echo "no CLI at $CLI" >&2; exit 2; }
[ -x "$C8/bin/javac" ] || { echo "no real javac 8 at $C8" >&2; exit 2; }

strip_comments() { grep -v '^[[:space:]]*//'; }

render() {
    # $1 = input, $2 = policy, $3 = class
    text=$("$CLI" class-source --input "$1" --policy "$2" --class "$3" --format text 2>/dev/null)
    case "$text" in
        "// jarde: presentation of"*) ;;
        *) echo "SELF-TEST FAILED: render of $3 has no header" >&2; exit 3 ;;
    esac
    printf '%s\n' "$text"
}

compile_leg() {
    # $1 = leg dir, $2 = real|installed, $3.. = sources (relative to the leg dir)
    dir=$1; kind=$2; shift 2
    if [ "$kind" = real ]; then
        (cd "$dir" && "$C8/bin/javac" -g:none -cp . -d . "$@")
    else
        (cd "$dir" && javac --release 8 -g:none -Xlint:-options -cp . -d . "$@")
    fi
}

run_leg() {
    # $1 = leg dir, $2 = real|installed, $3 = main class
    if [ "$2" = real ]; then
        (cd "$1" && "$C8/bin/java" -Xverify:all -cp . "$3")
    else
        (cd "$1" && java -Xverify:all -cp . "$3")
    fi
}

{
    echo "# shape | leg | side | trace"
    echo "shape|leg|side|trace"
} > "$OUT"

for leg in installed real; do
    # ---- ScopePlan: the declaration plan's positive class -------------------------------
    for side in original jadx jarde; do mkdir -p "$WORK/plan-$leg-$side"; done
    cp "$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlan.class" "$WORK/plan-$leg-original/"
    cp "$ROOT/tests/fixtures/preserve-local-scope-plan/ScopePlanDriver.java" "$WORK/plan-$leg-original/"
    compile_leg "$WORK/plan-$leg-original" "$leg" ScopePlanDriver.java
    cp "$ROOT/tests/fixtures/preserve-local-scope-plan/jadx/ScopePlan.java" "$WORK/plan-$leg-jadx/"
    { echo "package defpackage;"; cat "$ROOT/tests/fixtures/preserve-local-scope-plan/ScopePlanDriver.java"; } > "$WORK/plan-$leg-jadx/ScopePlanDriver.java"
    compile_leg "$WORK/plan-$leg-jadx" "$leg" ScopePlan.java ScopePlanDriver.java
    render "$ROOT/tests/fixtures/preserve-local-scope-plan/v8/ScopePlan.class" single-class ScopePlan | strip_comments > "$WORK/plan-$leg-jarde/ScopePlan.java"
    cp "$ROOT/tests/fixtures/preserve-local-scope-plan/ScopePlanDriver.java" "$WORK/plan-$leg-jarde/"
    compile_leg "$WORK/plan-$leg-jarde" "$leg" ScopePlan.java ScopePlanDriver.java
    printf '%s\n' "$(run_leg "$WORK/plan-$leg-original" "$leg" ScopePlanDriver)" > "$WORK/plan-$leg-original.trace"
    printf '%s\n' "$(run_leg "$WORK/plan-$leg-jadx" "$leg" defpackage.ScopePlanDriver)" > "$WORK/plan-$leg-jadx.trace"
    printf '%s\n' "$(run_leg "$WORK/plan-$leg-jarde" "$leg" ScopePlanDriver)" > "$WORK/plan-$leg-jarde.trace"
    for side in original jadx jarde; do
        while IFS= read -r line; do
            printf 'ScopePlan|%s|%s|%s\n' "$leg" "$side" "$line" >> "$OUT"
        done < "$WORK/plan-$leg-$side.trace"
    done

    # ---- LoopTryHandlerEntryArgs: the 1.5 computed write --------------------------------
    for side in original jadx jarde; do mkdir -p "$WORK/loop-$leg-$side"; done
    cp "$ROOT/tests/fixtures/p3-loop-try-handler-entry/v8/LoopTryHandlerEntryArgs.class" "$WORK/loop-$leg-original/"
    cp "$ROOT/tests/fixtures/p3-loop-try-handler-entry/Runner.java" "$WORK/loop-$leg-original/"
    compile_leg "$WORK/loop-$leg-original" "$leg" Runner.java
    cp "$ROOT/tests/fixtures/p3-loop-try-handler-entry/LoopTryHandlerEntryArgs.jadx.java.txt" "$WORK/loop-$leg-jadx/LoopTryHandlerEntryArgs.java"
    cp "$ROOT/tests/fixtures/p3-loop-try-handler-entry/Runner.java" "$WORK/loop-$leg-jadx/"
    compile_leg "$WORK/loop-$leg-jadx" "$leg" LoopTryHandlerEntryArgs.java Runner.java
    render "$ROOT/tests/fixtures/p3-loop-try-handler-entry/v8/LoopTryHandlerEntryArgs.class" single-class LoopTryHandlerEntryArgs | strip_comments > "$WORK/loop-$leg-jarde/LoopTryHandlerEntryArgs.java"
    cp "$ROOT/tests/fixtures/p3-loop-try-handler-entry/Runner.java" "$WORK/loop-$leg-jarde/"
    compile_leg "$WORK/loop-$leg-jarde" "$leg" LoopTryHandlerEntryArgs.java Runner.java
    for side in original jadx jarde; do
        run_leg "$WORK/loop-$leg-$side" "$leg" Runner | while IFS= read -r line; do
            printf 'LoopTryHandlerEntryArgs|%s|%s|%s\n' "$leg" "$side" "$line" >> "$OUT"
        done
    done

    # ---- the refused shape: the CLI text must stay uncompilable -------------------------
    mkdir -p "$WORK/refused-$leg"
    render "$ROOT/tests/fixtures/preserve-local-scope-refusals/v8/ScopeRefusals.class" single-class ScopeRefusals | strip_comments > "$WORK/refused-$leg/ScopeRefusals.java"
    if compile_leg "$WORK/refused-$leg" "$leg" ScopeRefusals.java 2>/dev/null; then
        echo "SELF-TEST FAILED: the refused shape compiled on $leg" >&2
        exit 4
    fi
    printf 'ScopeRefusals|%s|refused|uncompilable as stated\n' "$leg" >> "$OUT"
done

cat "$OUT"
