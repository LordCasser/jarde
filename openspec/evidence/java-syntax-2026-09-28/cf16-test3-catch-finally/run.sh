#!/bin/sh
set -eu
ROOT=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
TMP_ROOT=$(mktemp -d /private/tmp/cf16-test3.XXXXXX)
trap 'rm -rf "$TMP_ROOT"' EXIT HUP INT TERM
compile_run() {
    name=$1
    source=$2
    mkdir -p "$TMP_ROOT/$name/classes"
    if ! (
        cd "$ROOT"
        find standins -name '*.java' -print > "$TMP_ROOT/$name/sources.list"
        find runners -name '*.java' -print >> "$TMP_ROOT/$name/sources.list"
        printf '%s\n' "$source" >> "$TMP_ROOT/$name/sources.list"
        javac --release 8 -g:none -Xlint:-options -d "$TMP_ROOT/$name/classes" @"$TMP_ROOT/$name/sources.list"
    ) > "$ROOT/results/$name-javac.stdout" 2> "$ROOT/results/$name-javac.stderr"; then
        return 1
    fi
    javap -classpath "$TMP_ROOT/$name/classes" -c -v 'jadx.tests.integration.trycatch.TestTryCatchFinally3$TestCls' \
        | sed -E -e '1s@^Classfile .*@Classfile <compiled-class>@' -e '2s@Last modified .*; size @Last modified <filesystem timestamp>; size @' \
        > "$ROOT/bytecode/$name-java8.javap.txt"
    java -Xverify:all -cp "$TMP_ROOT/$name/classes" jadx.tests.integration.trycatch.Runner > "$ROOT/results/$name-run.txt" 2> "$ROOT/results/$name-verify.stderr"
}
compile_run original 'original-src/TestTryCatchFinally3$TestCls.java'
compile_run jadx 'jadx-src/TestTryCatchFinally3$TestCls.java'
if compile_run jarde 'jarde-out/TestTryCatchFinally3$TestCls.java'; then
    printf '%s\n' 'UNEXPECTED: Jarde full class-source compiled; inspect before treating as evidence.' > "$ROOT/results/jarde-compile-result.txt"
else
    printf '%s\n' 'EXPECTED REFUSAL: Jarde complete class-source did not compile; no java -Xverify:all execution was attempted.' > "$ROOT/results/jarde-compile-result.txt"
fi

mkdir -p "$TMP_ROOT/original-class/classes/jadx/tests/integration/trycatch"
cp "$ROOT/TestTryCatchFinally3\$TestCls.class" "$TMP_ROOT/original-class/classes/jadx/tests/integration/trycatch/"
(
    cd "$ROOT"
    find standins -name '*.java' -print > "$TMP_ROOT/original-class/sources.list"
    find runners -name '*.java' -print >> "$TMP_ROOT/original-class/sources.list"
    javac --release 8 -g:none -Xlint:-options -classpath "$TMP_ROOT/original-class/classes" -d "$TMP_ROOT/original-class/classes" @"$TMP_ROOT/original-class/sources.list"
)
java -Xverify:all -cp "$TMP_ROOT/original-class/classes" jadx.tests.integration.trycatch.Runner > "$ROOT/results/original-class-run.txt" 2> "$ROOT/results/original-class-verify.stderr"
