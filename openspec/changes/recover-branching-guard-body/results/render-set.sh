#!/bin/sh
# Renders every input of this slice's gating matrix with one binary, into one directory.
#
# Usage: render-set.sh <binary> <output-directory>
#
# The set is the five guard-family anchor sets this slice's invariant names — LK, IO, nested-lock,
# loop-test-copy and the copy/dup-store family — plus this slice's own fixture (whose class files
# stand in `tests/fixtures/recover-branching-guard-body/`, skipped until the fixture lands):
#
#   LK      the explicit-lock patrol's jar, the LK anchor on both legs, LockGuardNegatives,
#           LockGuardProbe;
#   IO      the io-wrapping patrol's jar, the IO anchor on both legs, IOMidRead, IONegatives,
#           NestedDepth;
#   nested  the multi-lock patrol's jar, ML/MLOrder/MLNegatives/MLProbe on both legs;
#   looptest the loop-test-copy slice's Probe/ProbeControls/MultiCopy on both legs and the
#           postfix family's LoopTestValues;
#   copy    the dup-store family's REF/DS/NEG and the CF-06 controls.
#
# Self-test before counting: every render must start with the layer's own header, or the script
# stops — a render that failed is not an answer. Absolute paths throughout, so the caller's
# directory never leaks into the comparison.
set -eu

BINARY=$1
OUT=$2
ROOT=$(cd "$(dirname "$0")/../../../.." && pwd)
FIX="$ROOT/tests/fixtures"
EVID="$ROOT/openspec/evidence"

[ -x "$BINARY" ] || { echo "no binary at $BINARY" >&2; exit 2; }
mkdir -p "$OUT"

render() {
    # $1 = kind, $2 = input, $3 = policy, $4 = class
    "$BINARY" class-source --input "$2" --policy "$3" --class "$4" --format text 2>/dev/null \
        > "$OUT/$1.java"
    head -1 "$OUT/$1.java" | grep -q '^// jarde: presentation of' || {
        echo "SELF-TEST FAILED: $1 has no header" >&2
        exit 3
    }
}

# The nested-lock family's frozen anchor and its registered boundaries (the branching body among
# them: this slice's own anchor flips, the other two boundaries stay refused).
render ml-patrol-jar "$EVID/java-syntax-2026-10-08/multi-lock-patrol/fixture/ml.jar" plain-jar ML
render ml-v8 "$FIX/recover-nested-lock-finally-bodies/v8/ML.class" single-class ML
render ml-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/ML.class" single-class ML
render mlorder-v8 "$FIX/recover-nested-lock-finally-bodies/v8/MLOrder.class" single-class MLOrder
render mlorder-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/MLOrder.class" single-class MLOrder
render mlnegatives-v8 "$FIX/recover-nested-lock-finally-bodies/v8/MLNegatives.class" single-class MLNegatives
render mlnegatives-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/MLNegatives.class" single-class MLNegatives
render mlprobe-v8 "$FIX/recover-nested-lock-finally-bodies/v8/MLProbe.class" single-class MLProbe
render mlprobe-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/MLProbe.class" single-class MLProbe
# The lock-guard family's frozen anchor, its negatives and its probe.
render lk-patrol-jar "$EVID/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar" plain-jar LK
render lk-v8 "$FIX/recover-lock-guard-loop-finally/v8/LK.class" single-class LK
render lk-v8-javac8 "$FIX/recover-lock-guard-loop-finally/v8-javac8/LK.class" single-class LK
render lknegatives-v8 "$FIX/recover-lock-guard-loop-finally/v8/LockGuardNegatives.class" single-class LockGuardNegatives
render lknegatives-v8-javac8 "$FIX/recover-lock-guard-loop-finally/v8-javac8/LockGuardNegatives.class" single-class LockGuardNegatives
render lkprobe-v8 "$FIX/recover-lock-guard-loop-finally/v8/LockGuardProbe.class" single-class LockGuardProbe
render lkprobe-v8-javac8 "$FIX/recover-lock-guard-loop-finally/v8-javac8/LockGuardProbe.class" single-class LockGuardProbe
# The resource-guard family's frozen anchor, its siblings and its controls.
render io-patrol-jar "$EVID/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar" plain-jar IO
render io-v8 "$FIX/recover-io-resource-finally/v8/IO.class" single-class IO
render io-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/IO.class" single-class IO
render iomidread-v8 "$FIX/recover-io-resource-finally/v8/IOMidRead.class" single-class IOMidRead
render iomidread-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/IOMidRead.class" single-class IOMidRead
render ionegatives-v8 "$FIX/recover-io-resource-finally/v8/IONegatives.class" single-class IONegatives
render ionegatives-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/IONegatives.class" single-class IONegatives
render nesteddepth-v8 "$FIX/recover-io-resource-finally/v8/NestedDepth.class" single-class NestedDepth
render nesteddepth-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/NestedDepth.class" single-class NestedDepth
# The loop-test-copy family's anchor, its control and its patched negative.
if [ -f "$FIX/recover-loop-test-copy-store/v8/Probe.class" ]; then
    render looptest-probe-v8 "$FIX/recover-loop-test-copy-store/v8/Probe.class" single-class Probe
    render looptest-probe-v8-javac8 "$FIX/recover-loop-test-copy-store/v8-javac8/Probe.class" single-class Probe
    render looptest-controls-v8 "$FIX/recover-loop-test-copy-store/v8/ProbeControls.class" single-class ProbeControls
    render looptest-controls-v8-javac8 "$FIX/recover-loop-test-copy-store/v8-javac8/ProbeControls.class" single-class ProbeControls
    render looptest-multicopy-v8 "$FIX/recover-loop-test-copy-store/v8/MultiCopy.class" single-class Probe
    render looptest-multicopy-v8-javac8 "$FIX/recover-loop-test-copy-store/v8-javac8/MultiCopy.class" single-class Probe
fi
render looptest-values-v8 "$FIX/p3-loop-test-values/v8/LoopTestValues.class" single-class LoopTestValues
# The copy/dup-store family's anchors and its CF-06 controls.
render dupstore-ref-v8 "$FIX/recover-dup-store-conditional/v8/REF.class" single-class REF
render dupstore-ds-v8 "$FIX/recover-dup-store-conditional/v8/DS.class" single-class DS
render dupstore-neg-v8 "$FIX/recover-dup-store-conditional/v8/NEG.class" single-class NEG
render dupstore-neg-v8-javac8 "$FIX/recover-dup-store-conditional/v8-javac8/NEG.class" single-class NEG
render cf06-negatives "$FIX/p3-inner-assignment/cf06/NegativeAssignments.class" single-class cf06/NegativeAssignments
render cf06-extra-copy "$FIX/p3-inner-assignment/ExtraCopy.class" single-class cf06/InnerAssignCases
render cf06-wrong-type "$FIX/p3-inner-assignment/WrongType.class" single-class cf06/InnerAssignCases
# This slice's own fixture, on both legs.
if [ -f "$FIX/recover-branching-guard-body/v8/BG.class" ]; then
    render bg-v8 "$FIX/recover-branching-guard-body/v8/BG.class" single-class BG
    render bg-v8-javac8 "$FIX/recover-branching-guard-body/v8-javac8/BG.class" single-class BG
    render bgorder-v8 "$FIX/recover-branching-guard-body/v8/BGOrder.class" single-class BGOrder
    render bgorder-v8-javac8 "$FIX/recover-branching-guard-body/v8-javac8/BGOrder.class" single-class BGOrder
    render bgnegatives-v8 "$FIX/recover-branching-guard-body/v8/BGNegatives.class" single-class BGNegatives
    render bgnegatives-v8-javac8 "$FIX/recover-branching-guard-body/v8-javac8/BGNegatives.class" single-class BGNegatives
    render bgprobe-v8 "$FIX/recover-branching-guard-body/v8/BGProbe.class" single-class BGProbe
    render bgprobe-v8-javac8 "$FIX/recover-branching-guard-body/v8-javac8/BGProbe.class" single-class BGProbe
fi
echo "rendered into $OUT"
