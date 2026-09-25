#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures"
WORK="${TMPDIR:-/tmp}/jarde-ordinary-parameterized-signatures"
export CARGO_TARGET_DIR="$WORK/cargo-target"
rm -rf "$WORK"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/g" "$WORK/ng"
javac --release 8 -Xlint:-options -g -d "$WORK/g" "$EVIDENCE/OrdinaryParameterizedSignatures.java" "$EVIDENCE/ReflectionRunner.java"
javac --release 8 -Xlint:-options -g:none -d "$WORK/ng" "$EVIDENCE/OrdinaryParameterizedSignatures.java" "$EVIDENCE/ReflectionRunner.java"
java -Xverify:all -cp "$WORK/g" ReflectionRunner
java -Xverify:all -cp "$WORK/ng" ReflectionRunner
jadx -d "$WORK/jadx-g" "$WORK/g/OrdinaryParameterizedSignatures.class"
jadx -d "$WORK/jadx-ng" "$WORK/ng/OrdinaryParameterizedSignatures.class"
mkdir -p "$WORK/jadx-compile"
javac --release 8 -Xlint:-options -d "$WORK/jadx-compile" "$WORK/jadx-g/sources/defpackage/OrdinaryParameterizedSignatures.java"
sed -e 's/public class ReflectionRunner/public class ReflectionRunnerJadx/' -e '1i\
package defpackage;
' "$EVIDENCE/ReflectionRunner.java" > "$WORK/ReflectionRunnerJadx.java"
javac --release 8 -Xlint:-options -cp "$WORK/jadx-compile" -d "$WORK/jadx-compile" "$WORK/ReflectionRunnerJadx.java"
java -Xverify:all -cp "$WORK/jadx-compile" defpackage.ReflectionRunnerJadx
cargo run -q -p jarde-cli -- class-source --input "$WORK/g/OrdinaryParameterizedSignatures.class" --class OrdinaryParameterizedSignatures --policy single-class > "$WORK/jarde-g.java" 2> "$WORK/jarde-g.log"
cargo run -q -p jarde-cli -- class-source --input "$WORK/ng/OrdinaryParameterizedSignatures.class" --class OrdinaryParameterizedSignatures --policy single-class > "$WORK/jarde-ng.java" 2> "$WORK/jarde-ng.log"
mkdir -p "$WORK/jarde-compile"
cp "$WORK/jarde-g.java" "$WORK/jarde-compile/OrdinaryParameterizedSignatures.java"
javac --release 8 -Xlint:-options -d "$WORK/jarde-compile" "$WORK/jarde-compile/OrdinaryParameterizedSignatures.java"
javac --release 8 -Xlint:-options -cp "$WORK/jarde-compile" -d "$WORK/jarde-compile" "$EVIDENCE/ReflectionRunner.java"
java -Xverify:all -cp "$WORK/jarde-compile" ReflectionRunner
python3 "$EVIDENCE/make_bad_signature.py" "$WORK/g/OrdinaryParameterizedSignatures.class" "$WORK/OrdinaryParameterizedSignatures-bad.class"
javap -v "$WORK/OrdinaryParameterizedSignatures-bad.class" > "$WORK/bad-javap.txt"
jadx -d "$WORK/jadx-bad" "$WORK/OrdinaryParameterizedSignatures-bad.class"
cargo run -q -p jarde-cli -- class-source --input "$WORK/OrdinaryParameterizedSignatures-bad.class" --class OrdinaryParameterizedSignatures --policy single-class > "$WORK/jarde-bad.java" 2> "$WORK/jarde-bad.log"
printf '\nTool versions\n'
java -version 2>&1 | head -3
javac -version
jadx --version
cargo --version
mkdir -p "$WORK/badrun"
cp "$WORK/OrdinaryParameterizedSignatures-bad.class" "$WORK/badrun/OrdinaryParameterizedSignatures.class"
cp "$EVIDENCE/ReflectionRunner.java" "$WORK/badrun/ReflectionRunner.java"
javac --release 8 -Xlint:-options -cp "$WORK/g" -d "$WORK/badrun" "$WORK/badrun/ReflectionRunner.java"
set +e
java -Xverify:all -cp "$WORK/badrun" ReflectionRunner > "$WORK/bad-runtime.txt" 2>&1
BAD_RUNTIME_STATUS=$?
set -e
printf 'bad runtime exit=%s\n' "$BAD_RUNTIME_STATUS"
cat "$WORK/bad-runtime.txt"
cp "$EVIDENCE/TreeSignatureReflectionRunner.java" "$WORK/badrun/TreeSignatureReflectionRunner.java"
javac --release 8 -Xlint:-options -cp "$WORK/g" -d "$WORK/badrun" "$WORK/badrun/TreeSignatureReflectionRunner.java"
set +e
java -Xverify:all -cp "$WORK/badrun" TreeSignatureReflectionRunner > "$WORK/tree-reflection-runtime.txt" 2>&1
TREE_REFLECTION_STATUS=$?
set -e
printf 'nested bad Signature reflection exit=%s (TypeNotPresent expected)\n' "$TREE_REFLECTION_STATUS"
cat "$WORK/tree-reflection-runtime.txt"
set +e
mkdir -p "$WORK/jarde-bad-compile"
cp "$WORK/jarde-bad.java" "$WORK/jarde-bad-compile/OrdinaryParameterizedSignatures.java"
javac --release 8 -Xlint:-options -d "$WORK/jarde-bad-compile" "$WORK/jarde-bad-compile/OrdinaryParameterizedSignatures.java" > "$WORK/jarde-bad-javac.txt" 2>&1
JARDE_BAD_COMPILE_STATUS=$?
javac --release 8 -Xlint:-options -d "$WORK/jadx-bad-compile" "$WORK/jadx-bad/sources/defpackage/OrdinaryParameterizedSignatures.java" > "$WORK/jadx-bad-javac.txt" 2>&1
JADX_BAD_COMPILE_STATUS=$?
set -e
printf 'Jarde List-to-Tree projected class compile exit=%s\n' "$JARDE_BAD_COMPILE_STATUS"
cat "$WORK/jarde-bad-javac.txt"
printf 'JADX List-to-Tree projected class compile exit=%s\n' "$JADX_BAD_COMPILE_STATUS"
cat "$WORK/jadx-bad-javac.txt"
shasum -a 256 "$EVIDENCE/OrdinaryParameterizedSignatures.java" "$EVIDENCE/ReflectionRunner.java" "$WORK/g/OrdinaryParameterizedSignatures.class" "$WORK/ng/OrdinaryParameterizedSignatures.class" "$WORK/jadx-g/sources/defpackage/OrdinaryParameterizedSignatures.java" "$WORK/jadx-ng/sources/defpackage/OrdinaryParameterizedSignatures.java" "$WORK/jarde-g.java" "$WORK/jarde-ng.java" "$WORK/jarde-bad.java" "$WORK/OrdinaryParameterizedSignatures-bad.class" "$EVIDENCE/TreeSignatureReflectionRunner.java"
