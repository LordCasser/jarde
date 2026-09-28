#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-multi-return-loop-finally/verification/neighbors
CLI=${1:?pass the freshly built jarde-cli path}
mkdir -p "$HERE/recovery"
for name in different-receiver different-target saved-value-rewritten self-row-expanded throwable-rewritten loop-extra-exit; do
  "$CLI" recover \
    --input "$HERE/$name.class" \
    --class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls' \
    --method-name test \
    --descriptor '(Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$A;Ljadx/tests/integration/trycatch/TestTryCatchFinally5$TestCls$B;)Ljava/util/List;' \
    --policy single-class --release 8 --format text \
    --output "$HERE/recovery/$name.txt" 2> /dev/null
  grep -q '@bytecode' "$HERE/recovery/$name.txt"
done
