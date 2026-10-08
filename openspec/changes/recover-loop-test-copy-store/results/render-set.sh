#!/bin/sh
# Render every anchor and negative of this slice's own matrix into one directory.
#
#   render-set.sh <output-directory>
#
# The set is the copy family's and the guard family's frozen artifacts plus this slice's own
# probe and negatives, each rendered through `render.sh` (which asserts the self-header). The
# files are the matrix: one render per artifact, named for the leg.
set -eu
out="$1"
here=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
root=$(CDPATH= cd -- "$here/../../../.." && pwd)
mkdir -p "$out"
render() {
    name="$1"
    artifact="$2"
    class="$3"
    policy="$4"
    CLI="${CLI:-$root/target/debug/jarde-cli}" sh "$here/render.sh" "$artifact" "$class" "$policy" > "$out/$name.txt"
}
# The slice's main anchor: the io-wrapping patrol's own IO, all three legs.
render io-patrol-jar "$root/openspec/evidence/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar" IO plain-jar
render io-v8 "$root/tests/fixtures/recover-io-resource-finally/v8/IO.class" IO single-class
render io-v8-javac8 "$root/tests/fixtures/recover-io-resource-finally/v8-javac8/IO.class" IO single-class
render io-midread-v8 "$root/tests/fixtures/recover-io-resource-finally/v8/IOMidRead.class" IOMidRead single-class
render io-midread-v8-javac8 "$root/tests/fixtures/recover-io-resource-finally/v8-javac8/IOMidRead.class" IOMidRead single-class
render io-negatives-v8 "$root/tests/fixtures/recover-io-resource-finally/v8/IONegatives.class" IONegatives single-class
render io-negatives-v8-javac8 "$root/tests/fixtures/recover-io-resource-finally/v8-javac8/IONegatives.class" IONegatives single-class
render nested-depth-v8 "$root/tests/fixtures/recover-io-resource-finally/v8/NestedDepth.class" NestedDepth single-class
render nested-depth-v8-javac8 "$root/tests/fixtures/recover-io-resource-finally/v8-javac8/NestedDepth.class" NestedDepth single-class
# The lock-guard family's frozen anchors.
render lk-patrol-jar "$root/openspec/evidence/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar" LK plain-jar
render lk-v8 "$root/tests/fixtures/recover-lock-guard-loop-finally/v8/LK.class" LK single-class
render lk-v8-javac8 "$root/tests/fixtures/recover-lock-guard-loop-finally/v8-javac8/LK.class" LK single-class
render lk-negatives-v8 "$root/tests/fixtures/recover-lock-guard-loop-finally/v8/LockGuardNegatives.class" LockGuardNegatives single-class
render lk-probe-v8 "$root/tests/fixtures/recover-lock-guard-loop-finally/v8/LockGuardProbe.class" LockGuardProbe single-class
# The dup-store family's anchors and negatives.
render dupstore-ref-v8 "$root/tests/fixtures/recover-dup-store-conditional/v8/REF.class" REF single-class
render dupstore-ds-v8 "$root/tests/fixtures/recover-dup-store-conditional/v8/DS.class" DS single-class
render dupstore-neg-v8 "$root/tests/fixtures/recover-dup-store-conditional/v8/NEG.class" NEG single-class
render dupstore-neg-v8-javac8 "$root/tests/fixtures/recover-dup-store-conditional/v8-javac8/NEG.class" NEG single-class
# The copy family's own frozen CF-06 controls (both patched variants declare `cf06.InnerAssignCases`).
render cf06-negatives "$root/tests/fixtures/p3-inner-assignment/cf06/NegativeAssignments.class" cf06/NegativeAssignments single-class
render cf06-extra-copy "$root/tests/fixtures/p3-inner-assignment/ExtraCopy.class" cf06/InnerAssignCases single-class
render cf06-wrong-type "$root/tests/fixtures/p3-inner-assignment/WrongType.class" cf06/InnerAssignCases single-class
# The postfix family's loop-test-values anchor.
render loop-test-values-v8 "$root/tests/fixtures/p3-loop-test-values/v8/LoopTestValues.class" LoopTestValues single-class
# This slice's own probe, its control and its negatives.
if [ -f "$root/tests/fixtures/recover-loop-test-copy-store/v8/Probe.class" ]; then
    render probe-v8 "$root/tests/fixtures/recover-loop-test-copy-store/v8/Probe.class" Probe single-class
    render probe-v8-javac8 "$root/tests/fixtures/recover-loop-test-copy-store/v8-javac8/Probe.class" Probe single-class
    render probe-controls-v8 "$root/tests/fixtures/recover-loop-test-copy-store/v8/ProbeControls.class" ProbeControls single-class
    render probe-controls-v8-javac8 "$root/tests/fixtures/recover-loop-test-copy-store/v8-javac8/ProbeControls.class" ProbeControls single-class
    render probe-multi-copy-v8 "$root/tests/fixtures/recover-loop-test-copy-store/v8/MultiCopy.class" Probe single-class
    render probe-multi-copy-v8-javac8 "$root/tests/fixtures/recover-loop-test-copy-store/v8-javac8/MultiCopy.class" Probe single-class
fi
echo "rendered into $out"
