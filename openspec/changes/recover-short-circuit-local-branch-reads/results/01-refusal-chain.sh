#!/bin/sh
# Task 1.1 forensics: where each anchor of this slice refuses at HEAD, located by instrumentation.
#
# The gate this change extends (`proves_boolean_local_store`) has several criteria before its
# consumer whitelist: the single write, the declaration region, the one region path, the SSA
# identity of every read. Which of them an anchor meets is not readable off the source — a loop
# header is a canonical block of its own, so the same `while (b)` a reader would call one lexical
# scope is a *different* region path here. This script therefore renders every probe with an
# instrumented build of **HEAD's gate** (this worktree is at the parent commit, so the copy it
# patches has the branch arm removed again) and stores the criterion each call reaches, verbatim.
#
# It is self-restoring: `crates/jarde-java/src/build.rs` is copied aside, patched, built, measured,
# and put back; the script fails loudly if the restore does not land.
#
# Usage: 01-refusal-chain.sh
#
# Output: 01-refusal-chain.out (the transcript), beside this script's directory.

set -eu

HERE=$(cd "$(dirname "$0")" && pwd)
ROOT=$(cd "$HERE/../../../.." && pwd)
OUT="$HERE/01-refusal-chain.out"
SOURCE="$ROOT/crates/jarde-java/src/build.rs"
BACKUP="$HERE/01-refusal-chain.build.rs.orig"
CLI="$ROOT/target/debug/jarde-cli"
FIXTURES="$ROOT/tests/fixtures/recover-short-circuit-local-branch-reads"

cp "$SOURCE" "$BACKUP"

restore() {
    cp "$BACKUP" "$SOURCE"
    rm -f "$BACKUP"
    ( cd "$ROOT" && cargo build --locked -p jarde-cli >/dev/null 2>&1 ) || {
        echo "RESTORE FAILED: the worktree's CLI did not rebuild" >&2
        exit 1
    }
}
trap 'restore' EXIT INT TERM

python3 - "$SOURCE" <<'PY'
import pathlib
import sys

path = pathlib.Path(sys.argv[1])
source = path.read_text()
# The branch arm itself, taken back out so the transcript states HEAD's answers.
arm = """            // A branch that tests the loaded value reads it at its condition position: javac lowers
            // `b ? x : y`, the statement `if (b)` and the loop `while (b)` to an `ifeq`/`ifne` on
            // the local's own load. The load keeps the position it already has — the branch is its
            // consumer, so nothing is reordered — and the arms it selects between are the existing
            // conditional-value and statement presentations'. The two zero-tests are named by
            // identity, so a numeric branch (`iflt`, `ifgt`) on the same load states no boolean
            // position and keeps its refusal.
            (0x99 | 0x9a, Some(Operation::Comparison { op, .. }))
                if matches!(op, CompareOp::JumpIfZero | CompareOp::JumpIfNotZero) =>
            {
                true
            }
"""
assert source.count(arm) == 1, "the branch arm is not where this script expects it"
source = source.replace(arm, "")
anchor = """        || accesses.iter().any(|access| access.path != write.path)
    {
        return Ok(None);
    }"""
assert source.count(anchor) == 1, "the shape criterion is not where this script expects it"
source = source.replace(anchor, """        || accesses.iter().any(|access| access.path != write.path)
    {
        eprintln!(
            "BRINSTR shape-criterion slot={slot} at={at} declaration={} write_bci={} written={} stored={} crossed={} accesses={:?}",
            declaration_region(accesses, paths).is_none(),
            write.bci,
            write.written == Some(written),
            write.stored == Some(phi),
            accesses.iter().any(|access| access.path != write.path),
            accesses
                .iter()
                .map(|access| (access.bci, access.read.is_some(), access.path.clone()))
                .collect::<Vec<_>>()
        );
        return Ok(None);
    }
    eprintln!(
        "BRINSTR reached-consumers slot={slot} at={at} accesses={:?}",
        accesses
            .iter()
            .map(|access| (access.bci, access.read.is_some(), access.path.clone()))
            .collect::<Vec<_>>()
    );""")
anchor = """        if !boolean_position {
            return Ok(None);
        }"""
assert source.count(anchor) == 1, "the consumer whitelist is not where this script expects it"
source = source.replace(anchor, """        if !boolean_position {
            eprintln!(
                "BRINSTR boolean-position slot={slot} at={at} consumer_bci={consumer_bci} opcode=0x{:02x} operation={:?}",
                consumer.opcode(),
                operations.get(consumer_bci)
            );
            return Ok(None);
        }""")
path.write_text(source)
print("instrumented")
PY

( cd "$ROOT" && cargo build --locked -p jarde-cli 2>&1 | tail -1 )

render() {
    # $1 = label, $2 = input, $3 = policy, $4 = class
    echo "=== $1 ==="
    "$CLI" class-source --input "$2" --policy "$3" --class "$4" --format text \
        2>&1 >/dev/null | grep BRINSTR | LC_ALL=C sort -u || true
    echo "--- the class's own refusal lines ---"
    "$CLI" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null |
        grep -E 'not recovered|short-circuit chain|more than one owner|local [0-9]+ crosses' |
        sed -n '1,6p'
    echo
}

{
    echo "# Task 1.1: the refusal each probe meets at HEAD (instrumented build), verbatim"
    echo "# columns of a BRINSTR line: the criterion reached, the local, the store BCI, and — for"
    echo "# the consumer whitelist — the consumer's own BCI, opcode and decoded operation."
    echo "# The lines are class-wide: one render states every member's gate call, and the member a"
    echo "# line belongs to is named by its at=/consumer_bci= pair (mapped in 01-refusal-chain.md)."
    echo
    render "OP2 (the patrol anchor: condAssignOld)" \
        "$ROOT/openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar" \
        plain-jar OP2
    render "BranchReads (ternaryRead / ifStatement / midChain / main)" \
        "$FIXTURES/v8/BranchReads.class" single-class BranchReads
    render "BranchReadNegatives (loopCondition / crossCatch / main)" \
        "$FIXTURES/v8/BranchReadNegatives.class" single-class BranchReadNegatives
    render "controls/NumericBranch (BranchReads with the ifeq at BCI 18 patched to iflt)" \
        "$FIXTURES/controls/NumericBranch.class" single-class BranchReads
    render "controls/NotZeroBranch (BranchReads with the ifeq at BCI 18 patched to ifne)" \
        "$FIXTURES/controls/NotZeroBranch.class" single-class BranchReads
} >"$OUT" 2>&1

echo "wrote $OUT"
