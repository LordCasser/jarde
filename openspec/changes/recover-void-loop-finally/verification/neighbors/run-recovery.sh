#!/bin/sh
# Replays both readings of every neighbor: the pre-change baseline binary's refusal and the
# current binary's refusal. Both outputs must keep the `@bytecode` quote and must not present a
# `finally`. Usage: sh run-recovery.sh /path/to/baseline-jarde-cli /path/to/current-jarde-cli
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-void-loop-finally/verification/neighbors
BASELINE=${1:?usage: run-recovery.sh /path/to/baseline-jarde-cli /path/to/current-jarde-cli}
CURRENT=${2:?usage: run-recovery.sh /path/to/baseline-jarde-cli /path/to/current-jarde-cli}

mkdir -p "$HERE/baseline" "$HERE/recovery"
for name in different-receiver different-target resource-definition-rewritten \
	self-row-expanded throwable-rewritten loop-extra-exit; do
	"$BASELINE" recover \
		--input "$HERE/$name.class" \
		--class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls' \
		--method-name test --descriptor '(Ljava/io/OutputStream;)V' \
		--policy single-class --release 8 --format text \
		--output "$HERE/baseline/$name.txt" 2> "$HERE/baseline/$name.stderr"
	"$CURRENT" recover \
		--input "$HERE/$name.class" \
		--class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls' \
		--method-name test --descriptor '(Ljava/io/OutputStream;)V' \
		--policy single-class --release 8 --format text \
		--output "$HERE/recovery/$name.txt" 2> "$HERE/recovery/$name.stderr"
	for side in baseline recovery; do
		grep -q '@bytecode' "$HERE/$side/$name.txt"
		if grep -q 'finally {' "$HERE/$side/$name.txt"; then
			echo "$side/$name presented a finally" >&2
			exit 1
		fi
	done
done
echo "all six neighbors refuse on both sides"
