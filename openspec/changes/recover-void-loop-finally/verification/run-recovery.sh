#!/bin/sh
# Regenerates this directory's Jarde records from the fixed class: the fresh CLI class-source, the
# structured recover report, and their digests. The Jarde binary is the caller's build.
# Usage: sh run-recovery.sh /path/to/jarde-cli
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-void-loop-finally/verification
FIX=$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally
JARDE=${1:?usage: run-recovery.sh /path/to/jarde-cli}

"$JARDE" class-source --input "$FIX/fixed/TestTryCatchFinally2\$TestCls.class" --policy single-class \
	--class 'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls' \
	> "$HERE/TestTryCatchFinally2\$TestCls.java" 2> "$HERE/class-source.stderr"
"$JARDE" recover --input "$FIX/fixed/TestTryCatchFinally2\$TestCls.class" --policy single-class \
	--class-name 'jadx/tests/integration/trycatch/TestTryCatchFinally2$TestCls' \
	--method-name test --descriptor '(Ljava/io/OutputStream;)V' --format json --evidence all \
	> "$HERE/recover-report.json" 2> "$HERE/recover-report.stderr"

python3 - "$HERE" <<'PY'
import json, sys
here = sys.argv[1]
report = json.load(open(f"{here}/recover-report.json"))
recovery = report["recovered"]["recovery"]
lines = [
    f"quality = {recovery.get('quality')}",
    f"fallbacks = {recovery.get('fallbacks')}",
    f"diagnostics = {[d.get('code') for d in recovery.get('diagnostics', [])]}",
]
open(f"{here}/recover-summary.txt", "w").write("\n".join(lines) + "\n")
PY

(cd "$HERE" && shasum -a 256 "TestTryCatchFinally2\$TestCls.java" recover-report.json \
	> recovery-sha256.txt)
printf 'Jarde source revision %s\n' "$(git -C "$ROOT" rev-parse HEAD)" >> "$HERE/recovery-sha256.txt"
printf 'class-source stderr bytes: %s\n' "$(wc -c < "$HERE/class-source.stderr")"
