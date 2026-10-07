#!/bin/sh
# Renders every input of this slice's gating matrix with one binary, into one directory.
#
# Usage: 02-render.sh <binary> <output-directory>
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

render ml-patrol-jar "$EVID/java-syntax-2026-10-08/multi-lock-patrol/fixture/ml.jar" plain-jar ML
render ml-v8 "$FIX/recover-nested-lock-finally-bodies/v8/ML.class" single-class ML
render ml-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/ML.class" single-class ML
render mlorder-v8 "$FIX/recover-nested-lock-finally-bodies/v8/MLOrder.class" single-class MLOrder
render mlorder-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/MLOrder.class" single-class MLOrder
render mlnegatives-v8 "$FIX/recover-nested-lock-finally-bodies/v8/MLNegatives.class" single-class MLNegatives
render mlnegatives-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/MLNegatives.class" single-class MLNegatives
render mlprobe-v8 "$FIX/recover-nested-lock-finally-bodies/v8/MLProbe.class" single-class MLProbe
render mlprobe-v8-javac8 "$FIX/recover-nested-lock-finally-bodies/v8-javac8/MLProbe.class" single-class MLProbe
render lk-patrol-jar "$EVID/java-syntax-2026-10-05/explicit-lock-patrol/fixture/lk.jar" plain-jar LK
render lk-v8 "$FIX/recover-lock-guard-loop-finally/v8/LK.class" single-class LK
render lk-v8-javac8 "$FIX/recover-lock-guard-loop-finally/v8-javac8/LK.class" single-class LK
render lknegatives-v8 "$FIX/recover-lock-guard-loop-finally/v8/LockGuardNegatives.class" single-class LockGuardNegatives
render lknegatives-v8-javac8 "$FIX/recover-lock-guard-loop-finally/v8-javac8/LockGuardNegatives.class" single-class LockGuardNegatives
render lkprobe-v8 "$FIX/recover-lock-guard-loop-finally/v8/LockGuardProbe.class" single-class LockGuardProbe
render lkprobe-v8-javac8 "$FIX/recover-lock-guard-loop-finally/v8-javac8/LockGuardProbe.class" single-class LockGuardProbe
render io-patrol-jar "$EVID/java-syntax-2026-10-05/io-wrapping-patrol/fixture/io.jar" plain-jar IO
render io-v8 "$FIX/recover-io-resource-finally/v8/IO.class" single-class IO
render io-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/IO.class" single-class IO
render iomidread-v8 "$FIX/recover-io-resource-finally/v8/IOMidRead.class" single-class IOMidRead
render iomidread-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/IOMidRead.class" single-class IOMidRead
render ionegeatives-v8 "$FIX/recover-io-resource-finally/v8/IONegatives.class" single-class IONegatives
render ionegeatives-v8-javac8 "$FIX/recover-io-resource-finally/v8-javac8/IONegatives.class" single-class IONegatives
render guarded-v8 "$FIX/p3-handlers/v8/Guarded.class" single-class Guarded
render p3-straight "$FIX/p3-finally-straight/v8/FinallyNormal.class" single-class FinallyNormal
render p3-concat-saved "$FIX/p3-concat-saved-finally/v8/FinallyOnce.class" single-class FinallyOnce
