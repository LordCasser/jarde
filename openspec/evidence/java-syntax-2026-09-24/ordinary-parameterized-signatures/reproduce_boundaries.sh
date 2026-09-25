#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures"
WORK="$(mktemp -d "${TMPDIR:-/tmp}/jarde-ordinary-parameterized-boundaries.XXXXXX")"
export CARGO_TARGET_DIR="$WORK/cargo-target"
mkdir -p "$WORK"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/classes" "$WORK/runners" "$WORK/patched"
javac --release 8 -Xlint:-options -g -d "$WORK/classes" \
  "$EVIDENCE/OverloadBinding.java" "$EVIDENCE/Boundaries.java" "$EVIDENCE/ClassVariableBoundary.java" \
  "$EVIDENCE/AnnotationPathBoundary.java" "$EVIDENCE/AmbiguousInnerBoundary.java" \
  "$EVIDENCE/RawBodyMismatch.java"
javac --release 8 -Xlint:-options -cp "$WORK/classes" -d "$WORK/runners" \
  "$EVIDENCE/OverloadBindingRunner.java" "$EVIDENCE/BoundaryReflection.java" "$EVIDENCE/RawBodyOriginalRunner.java" "$EVIDENCE/GenericSignatureReflectionRunner.java" "$EVIDENCE/ClassVariableRunner.java"
java -Xverify:all -cp "$WORK/classes:$WORK/runners" Boundaries
java -Xverify:all -cp "$WORK/classes:$WORK/runners" OverloadBindingRunner
javap -c -v "$WORK/classes/OverloadBinding.class" > "$WORK/OverloadBinding-javap.txt"
java -Xverify:all -cp "$WORK/classes:$WORK/runners" BoundaryReflection
java -Xverify:all -cp "$WORK/classes:$WORK/runners" ClassVariableRunner
java -Xverify:all -cp "$WORK/classes:$WORK/runners" RawBodyOriginalRunner
mkdir -p "$WORK/raw-runner" "$WORK/patched-runners"
set +e
javac --release 8 -Xlint:unchecked -cp "$WORK/classes" -d "$WORK/raw-runner" "$EVIDENCE/GenericLieRunner.java" > "$WORK/raw-runner-javac.txt" 2>&1
RAW_RUNNER_COMPILE_STATUS=$?
set -e
printf 'raw classpath GenericLieRunner compile exit=%s (unchecked conversion expected)\n' "$RAW_RUNNER_COMPILE_STATUS"
cat "$WORK/raw-runner-javac.txt"
set +e
java -Xverify:all -cp "$WORK/classes:$WORK/raw-runner" GenericLieRunner > "$WORK/raw-generic-lie-runtime.txt" 2>&1
RAW_LIE_STATUS=$?
set -e
printf 'raw classpath GenericLieRunner runtime exit=%s\n' "$RAW_LIE_STATUS"
cat "$WORK/raw-generic-lie-runtime.txt"
python3 "$EVIDENCE/add_method_signature.py" "$WORK/classes/RawBodyMismatch.class" \
  "$WORK/patched/RawBodyMismatch.class" body \
  '(Ljava/util/List<Ljava/lang/String;>;)Ljava/util/List<Ljava/lang/String;>;'
java -Xverify:all -cp "$WORK/patched:$WORK/classes:$WORK/runners" GenericSignatureReflectionRunner
javac --release 8 -Xlint:unchecked -cp "$WORK/patched:$WORK/classes" -d "$WORK/patched-runners" "$EVIDENCE/GenericLieRunner.java" > "$WORK/patched-runner-javac.txt" 2>&1
printf 'patched Signature GenericLieRunner compile exit=0\n'
cat "$WORK/patched-runner-javac.txt"
set +e
java -Xverify:all -cp "$WORK/patched:$WORK/classes:$WORK/patched-runners" GenericLieRunner > "$WORK/generic-lie-runtime.txt" 2>&1
LIE_STATUS=$?
set -e
printf 'validly signed but body-inconsistent class runtime exit=%s (expected ClassCastException)\n' "$LIE_STATUS"
cat "$WORK/generic-lie-runtime.txt"
for cls in Boundaries ClassVariableBoundary AnnotationPathBoundary AmbiguousInnerBoundary OverloadBinding; do
  javap -v "$WORK/classes/$cls.class" > "$WORK/$cls-javap.txt"
done
javap -v "$WORK/patched/RawBodyMismatch.class" > "$WORK/RawBodyMismatch-patched-javap.txt"
jar --create --file "$WORK/boundaries.jar" -C "$WORK/classes" .
jadx -d "$WORK/jadx" "$WORK/boundaries.jar"
jadx -d "$WORK/jadx-mismatch" "$WORK/patched/RawBodyMismatch.class"
for cls in Boundaries ClassVariableBoundary AnnotationPathBoundary AmbiguousInnerBoundary OverloadBinding; do
  cargo run -q -p jarde-cli -- class-source --input "$WORK/classes/$cls.class" --class "$cls" --policy single-class > "$WORK/jarde-$cls.java" 2> "$WORK/jarde-$cls.log"
done
cargo run -q -p jarde-cli -- class-source --input "$WORK/patched/RawBodyMismatch.class" --class RawBodyMismatch --policy single-class > "$WORK/jarde-RawBodyMismatch.java" 2> "$WORK/jarde-RawBodyMismatch.log"
mkdir -p "$WORK/jadx-compile" "$WORK/jarde-compile"
set +e
find "$WORK/jadx/sources" -name '*.java' -print0 | xargs -0 javac --release 8 -Xlint:-options -d "$WORK/jadx-compile" > "$WORK/jadx-compile.log" 2>&1
JADX_COMPILE_STATUS=$?
set -e
printf 'JADX whole tree compile exit=%s\n' "$JADX_COMPILE_STATUS"
cat "$WORK/jadx-compile.log"
sed -e 's/public class BoundaryReflection/public class BoundaryReflectionJadx/' -e '1i\
package defpackage;
' "$EVIDENCE/BoundaryReflection.java" > "$WORK/BoundaryReflectionJadx.java"
javac --release 8 -Xlint:-options -cp "$WORK/jadx-compile" -d "$WORK/jadx-compile" "$WORK/BoundaryReflectionJadx.java"
java -Xverify:all -cp "$WORK/jadx-compile" defpackage.Boundaries
sed -e 's/public class OverloadBindingRunner/public class OverloadBindingRunnerJadx/' -e '1i\
package defpackage;
' "$EVIDENCE/OverloadBindingRunner.java" > "$WORK/OverloadBindingRunnerJadx.java"
javac --release 8 -Xlint:-options -cp "$WORK/jadx-compile" -d "$WORK/jadx-compile" "$WORK/OverloadBindingRunnerJadx.java"
java -Xverify:all -cp "$WORK/jadx-compile" defpackage.OverloadBindingRunnerJadx
java -Xverify:all -cp "$WORK/jadx-compile" defpackage.BoundaryReflectionJadx
sed -e 's/public class ClassVariableRunner/public class ClassVariableRunnerJadx/' -e '1i\
package defpackage;
' "$EVIDENCE/ClassVariableRunner.java" > "$WORK/ClassVariableRunnerJadx.java"
javac --release 8 -Xlint:-options -cp "$WORK/jadx-compile" -d "$WORK/jadx-compile" "$WORK/ClassVariableRunnerJadx.java"
java -Xverify:all -cp "$WORK/jadx-compile" defpackage.ClassVariableRunnerJadx
set +e
javac --release 8 -Xlint:-options -d "$WORK/jadx-mismatch-compile" "$WORK/jadx-mismatch/sources/defpackage/RawBodyMismatch.java" > "$WORK/jadx-mismatch-javac.txt" 2>&1
JADX_MISMATCH_STATUS=$?
set -e
printf 'JADX body mismatch compile exit=%s (expected nonzero)\n' "$JADX_MISMATCH_STATUS"
cat "$WORK/jadx-mismatch-javac.txt"
for cls in Boundaries ClassVariableBoundary AnnotationPathBoundary AmbiguousInnerBoundary RawBodyMismatch; do
  cp "$WORK/jarde-$cls.java" "$WORK/jarde-compile/$cls.java"
done
set +e
javac --release 8 -Xlint:-options -cp "$WORK/classes" -d "$WORK/jarde-compile" "$WORK/jarde-compile/Boundaries.java" > "$WORK/jarde-boundaries-javac.txt" 2>&1
JARDE_BOUNDARIES_STATUS=$?
set -e
printf 'Jarde Boundaries class compile exit=%s\n' "$JARDE_BOUNDARIES_STATUS"
cat "$WORK/jarde-boundaries-javac.txt"
cp "$WORK/jarde-OverloadBinding.java" "$WORK/jarde-compile/OverloadBinding.java"
javac --release 8 -Xlint:-options -d "$WORK/jarde-compile" "$WORK/jarde-compile/OverloadBinding.java"
javac --release 8 -Xlint:-options -cp "$WORK/jarde-compile" -d "$WORK/jarde-compile" "$EVIDENCE/OverloadBindingRunner.java"
java -Xverify:all -cp "$WORK/jarde-compile" OverloadBindingRunner
javac --release 8 -Xlint:-options -d "$WORK/jarde-compile" "$WORK/jarde-compile/ClassVariableBoundary.java"
set +e
javac --release 8 -Xlint:-options -cp "$WORK/jarde-compile" -d "$WORK/jarde-compile" "$EVIDENCE/ClassVariableRunner.java" > "$WORK/jarde-class-variable-runner.txt" 2>&1
JARDE_CLASS_VAR_RUNNER_STATUS=$?
set -e
printf 'Jarde class-variable runner compile exit=%s (expected nonzero)\n' "$JARDE_CLASS_VAR_RUNNER_STATUS"
cat "$WORK/jarde-class-variable-runner.txt"
javac --release 8 -Xlint:-options -d "$WORK/jarde-compile" "$WORK/jarde-compile/RawBodyMismatch.java"
javac --release 8 -Xlint:-options -cp "$WORK/jarde-compile" -d "$WORK/jarde-compile" "$EVIDENCE/GenericLieRunner.java"
set +e
java -Xverify:all -cp "$WORK/jarde-compile" GenericLieRunner > "$WORK/jarde-generic-lie-runtime.txt" 2>&1
JARDE_LIE_STATUS=$?
set -e
printf 'Jarde raw fallback generic runner exit=%s (expected ClassCastException)\n' "$JARDE_LIE_STATUS"
cat "$WORK/jarde-generic-lie-runtime.txt"
printf '\nTool versions\n'
java -version 2>&1 | head -3
javac -version
jadx --version
cargo --version
printf '\nRelevant outputs\n'
rg -n 'Signature:|Methodref.*(Boundaries\.(bodyOverload|choose)|OverloadBinding\.choose)|RuntimeVisibleTypeAnnotations|TypePath' "$WORK"/*javap.txt | head -100
rg -n 'refused|type annotation|identity\(|left\(|right\(|List<String>|add\(42\)|class-variable' "$WORK"/jarde-*.java "$WORK"/jadx/sources/defpackage/*.java | head -150
printf '\nSHA-256\n'
shasum -a 256 "$EVIDENCE/Boundaries.java" "$EVIDENCE/BoundaryReflection.java" "$EVIDENCE/OverloadBinding.java" "$EVIDENCE/OverloadBindingRunner.java" "$EVIDENCE/ClassVariableBoundary.java" "$EVIDENCE/AnnotationPathBoundary.java" "$EVIDENCE/AmbiguousInnerBoundary.java" "$EVIDENCE/RawBodyMismatch.java" "$EVIDENCE/RawBodyOriginalRunner.java" "$EVIDENCE/GenericLieRunner.java" "$EVIDENCE/GenericSignatureReflectionRunner.java" "$EVIDENCE/ClassVariableRunner.java" "$WORK/classes/OverloadBinding.class" "$WORK/classes/Boundaries.class" "$WORK/classes/ClassVariableBoundary.class" "$WORK/classes/AnnotationPathBoundary.class" "$WORK/classes/AmbiguousInnerBoundary.class" "$WORK/classes/RawBodyMismatch.class" "$WORK/patched/RawBodyMismatch.class" "$WORK/jadx/sources/defpackage/OverloadBinding.java" "$WORK/jadx/sources/defpackage/Boundaries.java" "$WORK/jadx-mismatch/sources/defpackage/RawBodyMismatch.java" "$WORK/jadx/sources/defpackage/ClassVariableBoundary.java" "$WORK/jadx/sources/defpackage/AnnotationPathBoundary.java" "$WORK/jadx/sources/defpackage/AmbiguousInnerBoundary.java" "$WORK/jarde-OverloadBinding.java" "$WORK/jarde-Boundaries.java" "$WORK/jarde-ClassVariableBoundary.java" "$WORK/jarde-AnnotationPathBoundary.java" "$WORK/jarde-AmbiguousInnerBoundary.java" "$WORK/jarde-RawBodyMismatch.java"
printf '\nOutputs: %s\n' "$WORK"
