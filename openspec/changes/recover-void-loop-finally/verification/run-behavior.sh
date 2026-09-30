#!/bin/sh
# Four-way behavior comparison on the fixed CF-16 Test2 profile: the frozen class, the standalone
# Java 8 transcription, the frozen Java-input JADX source, and this change's fresh Jarde
# class-source. Every variant runs all eight probe paths under `java -Xverify:all`; the four
# outputs must be byte-identical.
set -eu
ROOT=$(git rev-parse --show-toplevel)
HERE=$ROOT/openspec/changes/recover-void-loop-finally/verification
FIX=$ROOT/openspec/evidence/java-syntax-2026-09-28/cf16-test2-loop-finally
TMP=$(mktemp -d /tmp/jarde-cf16t2-behavior.XXXXXX)
trap 'rm -rf "$TMP"' EXIT HUP INT TERM
mkdir -p "$TMP"/{fixed,original,jadx,jarde}/jadx/tests/integration/trycatch \
	"$TMP/support/jadx/core/clsp" "$TMP/support/jadx/core/dex/instructions/args"
cp "$FIX/support/jadx/core/clsp/ClspClass.java" "$TMP/support/jadx/core/clsp/"
cp "$FIX/support/jadx/core/dex/instructions/args/ArgType.java" "$TMP/support/jadx/core/dex/instructions/args/"
SUPPORT="$TMP/support/jadx/core/clsp/ClspClass.java $TMP/support/jadx/core/dex/instructions/args/ArgType.java"
FIXED_CLASS="$FIX/fixed/TestTryCatchFinally2\$TestCls.class"

cp "$FIXED_CLASS" "$TMP/fixed/jadx/tests/integration/trycatch/"
javac --release 8 -g -Xlint:-options -classpath "$TMP/fixed" -d "$TMP/fixed" $SUPPORT "$FIX/probe/Runner.java" \
	2> "$HERE/fixed.javac.stderr"

javac --release 8 -g -Xlint:-options -classpath "$TMP/original" -d "$TMP/original" $SUPPORT \
	"$FIX/original/TestTryCatchFinally2\$TestCls.java" "$FIX/probe/Runner.java" \
	2> "$HERE/original.javac.stderr"

javac --release 8 -g -Xlint:-options -classpath "$TMP/jadx" -d "$TMP/jadx" $SUPPORT \
	"$FIX/jadx/TestTryCatchFinally2\$TestCls.java" "$FIX/probe/Runner.java" \
	2> "$HERE/jadx.javac.stderr"

javac --release 8 -g -Xlint:-options -classpath "$TMP/jarde" -d "$TMP/jarde" $SUPPORT \
	"$HERE/TestTryCatchFinally2\$TestCls.java" "$FIX/probe/Runner.java" \
	2> "$HERE/jarde.javac.stderr"

for profile in fixed original jadx jarde; do
	java -Xverify:all -cp "$TMP/$profile" jadx.tests.integration.trycatch.Runner \
		> "$HERE/$profile.run.txt" 2> "$HERE/$profile.verify.stderr"
	cmp -s /dev/null "$HERE/$profile.verify.stderr"
done
cmp "$HERE/fixed.run.txt" "$HERE/original.run.txt"
cmp "$HERE/fixed.run.txt" "$HERE/jadx.run.txt"
cmp "$HERE/fixed.run.txt" "$HERE/jarde.run.txt"

(cd "$HERE" && shasum -a 256 \
	"TestTryCatchFinally2\$TestCls.java" \
	"$FIX/fixed/TestTryCatchFinally2\$TestCls.class" \
	"$FIX/original/TestTryCatchFinally2\$TestCls.java" \
	"$FIX/jadx/TestTryCatchFinally2\$TestCls.java" \
	"$FIX/probe/Runner.java") > "$HERE/behavior-sha256.txt"
printf 'paths: %s\n' "$(grep -c 'outcome=' "$HERE/fixed.run.txt")"
