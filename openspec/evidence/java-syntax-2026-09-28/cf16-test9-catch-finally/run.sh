#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TMP_ROOT=$(mktemp -d /private/tmp/cf16-test9.XXXXXX)
trap 'rm -rf "$TMP_ROOT"' EXIT HUP INT TERM

mkdir -p "$TMP_ROOT/runner-classes"
javac --release 8 -g:none -Xlint:-options -d "$TMP_ROOT/runner-classes" "$ROOT/fixtures/Runner.java"

run_source() {
	name=$1
	source=$2
	mkdir -p "$TMP_ROOT/$name/classes"
	if ! javac --release 8 -g -Xlint:-options -d "$TMP_ROOT/$name/classes" "$source" > "$ROOT/results/$name-javac.stdout" 2> "$ROOT/results/$name-javac.stderr"; then
		return 1
	fi
	javap -classpath "$TMP_ROOT/$name/classes" -c -v 'jadx.tests.integration.trycatch.TestTryCatchFinally9$TestCls' \
		| sed -E -e '1s@^Classfile .*@Classfile <compiled-class>@' -e '2s@Last modified .*; size @Last modified <filesystem timestamp>; size @' \
		> "$ROOT/bytecode/$name.javap.txt"
	java -Xverify:all -cp "$TMP_ROOT/runner-classes" Runner "$TMP_ROOT/$name/classes" > "$ROOT/results/$name-run.txt" 2> "$ROOT/results/$name-verify.stderr"
}

run_source original "$ROOT/fixtures/TestTryCatchFinally9\$TestCls.java"
run_source jadx-java-input "$ROOT/jadx-java-input/TestTryCatchFinally9\$TestCls.java"
run_source jadx-dx "$ROOT/jadx-dx/TestTryCatchFinally9\$TestCls.java"

mkdir -p "$TMP_ROOT/original-class/classes/jadx/tests/integration/trycatch"
cp "$ROOT/TestTryCatchFinally9\$TestCls.class" "$TMP_ROOT/original-class/classes/jadx/tests/integration/trycatch/"
java -Xverify:all -cp "$TMP_ROOT/runner-classes" Runner "$TMP_ROOT/original-class/classes" > "$ROOT/results/original-class-run.txt" 2> "$ROOT/results/original-class-verify.stderr"

if rg -q 'jarde: not recovered:|@bytecode' "$ROOT/jarde-out/TestTryCatchFinally9\$TestCls.java"; then
	printf '%s\n' 'Jarde source contains recovery/fallback markers.' > "$ROOT/results/jarde-compile-result.txt"
	exit 1
fi
mkdir -p "$TMP_ROOT/jarde-classes"
if ! (cd "$ROOT" && javac --release 8 -g -Xlint:-options -d "$TMP_ROOT/jarde-classes" 'jarde-out/TestTryCatchFinally9$TestCls.java') > "$ROOT/results/jarde-javac.stdout" 2> "$ROOT/results/jarde-javac.stderr"; then
	printf '%s\n' 'Jarde full source compilation failed; no java -Xverify:all execution was attempted.' > "$ROOT/results/jarde-compile-result.txt"
	exit 1
fi
java -Xverify:all -cp "$TMP_ROOT/runner-classes" Runner "$TMP_ROOT/jarde-classes" > "$ROOT/results/jarde-run.txt" 2> "$ROOT/results/jarde-verify.stderr"
printf '%s\n' 'Jarde full source compiled and ran under java -Xverify:all.' > "$ROOT/results/jarde-compile-result.txt"

cmp "$ROOT/results/original-class-run.txt" "$ROOT/results/original-run.txt"
cmp "$ROOT/results/original-class-run.txt" "$ROOT/results/jadx-dx-run.txt"
cmp "$ROOT/results/original-class-run.txt" "$ROOT/results/jarde-run.txt"
rg -q '^present result=resource-data close=0 exception=none identity=none readFailure=0$' "$ROOT/results/jadx-java-input-run.txt"
