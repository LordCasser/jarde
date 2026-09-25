#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
EVIDENCE="$ROOT/openspec/changes/recover-class-type-variable-signatures/evidence"
FIXTURES="$ROOT/tests/fixtures/recover-class-type-variable-signatures"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-class-type-variables.XXXXXX")
export CARGO_TARGET_DIR="$WORK/cargo-target"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/original-g" "$WORK/original-none" "$WORK/runners" "$WORK/patched" \
  "$WORK/jadx" "$WORK/jarde" "$WORK/jarde-compiled" "$WORK/jadx-compiled"

find "$FIXTURES" -name '*.java' -print0 | xargs -0 javac --release 8 -Xlint:-options -g -d "$WORK/original-g"
find "$FIXTURES" -name '*.java' -print0 | xargs -0 javac --release 8 -Xlint:-options -g:none -d "$WORK/original-none"
javac --release 8 -Xlint:-options -cp "$WORK/original-g" -d "$WORK/runners" \
  "$FIXTURES/positive/ClassVariableRunner.java" \
  "$FIXTURES/positive/BoundedClassVariableRunner.java" \
  "$FIXTURES/positive/InterfaceVariableRunner.java" \
  "$FIXTURES/negative/NegativeRunner.java"

printf '%s\n' '== original legal Java 8 class files: verify, execute, reflection =='
java -Xverify:all -cp "$WORK/original-g:$WORK/runners" classvars.ClassVariableRunner
java -Xverify:all -cp "$WORK/original-g:$WORK/runners" classvars.BoundedClassVariableRunner
java -Xverify:all -cp "$WORK/original-g:$WORK/runners" classvars.InterfaceVariableRunner
java -Xverify:all -cp "$WORK/original-none:$WORK/runners" classvars.ClassVariableRunner
java -Xverify:all -cp "$WORK/original-none:$WORK/runners" classvars.BoundedClassVariableRunner
java -Xverify:all -cp "$WORK/original-none:$WORK/runners" classvars.InterfaceVariableRunner
for case in parent interface unbound shadow inner; do
  java -Xverify:all -cp "$WORK/original-g:$WORK/runners" classvars.NegativeRunner "$case"
done

printf '%s\n' '== original Signature attributes =='
for class in ClassVariableBoundary BoundedClassVariableBoundary ClassVariableContract ParentMismatch InterfaceMismatch UnboundClassVariable MethodShadowBoundary; do
  javap -v "$WORK/original-g/classvars/$class.class" > "$WORK/$class-original.javap"
  echo "--- $class"
  rg -n 'descriptor:|Signature:' "$WORK/$class-original.javap" | tail -8
done
javap -v "$WORK/original-none/classvars/ClassVariableBoundary.class" > "$WORK/ClassVariableBoundary-none.javap"
javap -v "$WORK/original-none/classvars/BoundedClassVariableBoundary.class" > "$WORK/BoundedClassVariableBoundary-none.javap"
javap -v "$WORK/original-none/classvars/ClassVariableContract.class" > "$WORK/ClassVariableContract-none.javap"
javap -v "$WORK/original-g/classvars/OuterScopeBoundary\$Inner.class" > "$WORK/OuterScopeBoundary-Inner-original.javap"

printf '%s\n' '== verifier-accepted Signature-only mutations =='
mkdir -p "$WORK/patched/classvars"
python3 "$EVIDENCE/patch_signature.py" \
  "$WORK/original-g/classvars/ParentMismatch.class" "$WORK/patched/classvars/ParentMismatch.class" \
  class '<U:Ljava/lang/Object;>Ljava/lang/Object;' '<U:Ljava/lang/Object;>Ljava/lang/Thread;'
python3 "$EVIDENCE/patch_signature.py" \
  "$WORK/original-g/classvars/InterfaceMismatch.class" "$WORK/patched/classvars/InterfaceMismatch.class" \
  class '<U:Ljava/lang/Object;>Ljava/lang/Object;Ljava/lang/Runnable;' \
  '<U:Ljava/lang/Object;>Ljava/lang/Object;Ljava/util/concurrent/Callable;'
python3 "$EVIDENCE/patch_signature.py" \
  "$WORK/original-g/classvars/UnboundClassVariable.class" "$WORK/patched/classvars/UnboundClassVariable.class" \
  method:identity '(TU;)TU;' '(TV;)TV;'
for class in ParentMismatch InterfaceMismatch UnboundClassVariable; do
  javap -v "$WORK/patched/classvars/$class.class" > "$WORK/$class-patched.javap"
  echo "--- patched $class"
  rg -n 'descriptor:|Signature:' "$WORK/$class-patched.javap" | tail -8
done
for case in parent interface unbound; do
  java -Xverify:all -cp "$WORK/patched:$WORK/original-g:$WORK/runners" classvars.NegativeRunner "$case"
done

printf '%s\n' '== JADX 1.5.6 declarations for positive and negative inputs =='
for class in ClassVariableBoundary BoundedClassVariableBoundary ClassVariableContract ParentMismatch InterfaceMismatch UnboundClassVariable MethodShadowBoundary; do
  input="$WORK/original-g/classvars/$class.class"
  case "$class" in
    ParentMismatch|InterfaceMismatch|UnboundClassVariable) input="$WORK/patched/classvars/$class.class" ;;
  esac
  set +e
  jadx -d "$WORK/jadx/$class" "$input" > "$WORK/jadx/$class.stdout" 2> "$WORK/jadx/$class.log"
  jadx_status=$?
  set -e
  printf 'JADX %s exit=%s\n' "$class" "$jadx_status"
  if [ "$jadx_status" -ne 0 ]; then cat "$WORK/jadx/$class.log"; fi
done
set +e
jadx -d "$WORK/jadx/OuterScopeBoundary-Inner" "$WORK/original-g/classvars/OuterScopeBoundary\$Inner.class" \
  > "$WORK/jadx/OuterScopeBoundary-Inner.stdout" 2> "$WORK/jadx/OuterScopeBoundary-Inner.log"
jadx_inner_status=$?
set -e
printf 'JADX OuterScopeBoundary$Inner exit=%s\n' "$jadx_inner_status"
for source in "$WORK"/jadx/*/sources/classvars/*.java; do
  [ -f "$source" ] || continue
  echo "--- JADX ${source##*/}"
  rg -n 'class |interface |<U|<T|identity\(' "$source" | head -20 || true
done

printf '%s\n' '== JADX no-debug generic caller recompilation/execution =='
for class in ClassVariableBoundary BoundedClassVariableBoundary ClassVariableContract; do
  jadx -d "$WORK/jadx-none/$class" "$WORK/original-none/classvars/$class.class" \
    > "$WORK/jadx-none-$class.stdout" 2> "$WORK/jadx-none-$class.log"
  mkdir -p "$WORK/jadx-none-compiled/$class"
  case "$class" in
    ClassVariableBoundary) runner_source=ClassVariableRunner ;;
    BoundedClassVariableBoundary) runner_source=BoundedClassVariableRunner ;;
    ClassVariableContract) runner_source=InterfaceVariableRunner ;;
  esac
  if [ "$class" = ClassVariableContract ]; then
    javac --release 8 -Xlint:-options -d "$WORK/jadx-none-compiled/$class" \
      "$WORK/jadx-none/$class/sources/classvars/$class.java" \
      "$FIXTURES/positive/StringContract.java" "$FIXTURES/positive/$runner_source.java"
  else
    javac --release 8 -Xlint:-options -d "$WORK/jadx-none-compiled/$class" \
      "$WORK/jadx-none/$class/sources/classvars/$class.java" \
      "$FIXTURES/positive/$runner_source.java"
  fi
  java -Xverify:all -cp "$WORK/jadx-none-compiled/$class" "classvars.$runner_source"
done

printf '%s\n' '== Jarde class-source declarations =='
for class in ClassVariableBoundary BoundedClassVariableBoundary ClassVariableContract ParentMismatch InterfaceMismatch UnboundClassVariable MethodShadowBoundary; do
  input="$WORK/original-g/classvars/$class.class"
  case "$class" in
    ParentMismatch|InterfaceMismatch|UnboundClassVariable) input="$WORK/patched/classvars/$class.class" ;;
  esac
  set +e
  cargo run -q -p jarde-cli -- class-source --input "$input" --class "classvars.$class" \
    --policy single-class > "$WORK/jarde/$class.java" 2> "$WORK/jarde/$class.log"
  jarde_status=$?
  set -e
  printf 'Jarde %s exit=%s\n' "$class" "$jarde_status"
  echo "--- Jarde $class"
  rg -n 'class |interface |<U|<T|identity\(|class Signature projection refused|generic Signature projection refused' "$WORK/jarde/$class.java" | head -12 || true
done
set +e
cargo run -q -p jarde-cli -- class-source --input "$WORK/original-g/classvars/OuterScopeBoundary\$Inner.class" \
  --class 'classvars.OuterScopeBoundary$Inner' --policy single-class \
  > "$WORK/jarde/OuterScopeBoundary-Inner.java" 2> "$WORK/jarde/OuterScopeBoundary-Inner.log"
jarde_inner_status=$?
set -e
printf 'Jarde OuterScopeBoundary$Inner exit=%s\n' "$jarde_inner_status"
rg -n 'class |<T|identity\(|jarde:' "$WORK/jarde/OuterScopeBoundary-Inner.java" | head -24 || true

printf '%s\n' '== complete generic caller recompilation/execution =='
for class in ClassVariableBoundary BoundedClassVariableBoundary; do
  mkdir -p "$WORK/jadx-compiled/$class" "$WORK/jarde-compiled/$class"
  cp "$WORK/jarde/$class.java" "$WORK/jarde-compiled/$class/$class.java"
  case "$class" in
    ClassVariableBoundary) runner_source=ClassVariableRunner; runner_class=ClassVariableRunner ;;
    BoundedClassVariableBoundary) runner_source=BoundedClassVariableRunner; runner_class=BoundedClassVariableRunner ;;
  esac
  javac --release 8 -Xlint:-options -d "$WORK/jadx-compiled/$class" \
    "$WORK/jadx/$class/sources/classvars/$class.java" \
    "$FIXTURES/positive/$runner_source.java"
  javac --release 8 -Xlint:-options -d "$WORK/jarde-compiled/$class" \
    "$WORK/jarde-compiled/$class/$class.java"
  runner="classvars.$runner_class"
  java -Xverify:all -cp "$WORK/jadx-compiled/$class" "$runner"
  set +e
  javac --release 8 -Xlint:-options -cp "$WORK/jarde-compiled/$class" \
    -d "$WORK/jarde-compiled/$class" "$FIXTURES/positive/$runner_source.java" \
    > "$WORK/$class-jarde-generic-caller.log" 2>&1
  caller_status=$?
  printf 'Jarde %s generic caller compile exit=%s\n' "$class" "$caller_status"
  cat "$WORK/$class-jarde-generic-caller.log"
  if [ "$caller_status" -ne 0 ]; then exit "$caller_status"; fi
  set -e
  java -Xverify:all -cp "$WORK/jarde-compiled/$class" "$runner"
done

mkdir -p "$WORK/jadx-compiled/ClassVariableContract" "$WORK/jarde-compiled/ClassVariableContract"
cp "$WORK/jarde/ClassVariableContract.java" "$WORK/jarde-compiled/ClassVariableContract/ClassVariableContract.java"
javac --release 8 -Xlint:-options -d "$WORK/jadx-compiled/ClassVariableContract" \
  "$WORK/jadx/ClassVariableContract/sources/classvars/ClassVariableContract.java" \
  "$FIXTURES/positive/StringContract.java" "$FIXTURES/positive/InterfaceVariableRunner.java"
javac --release 8 -Xlint:-options -d "$WORK/jarde-compiled/ClassVariableContract" \
  "$WORK/jarde-compiled/ClassVariableContract/ClassVariableContract.java"
set +e
javac --release 8 -Xlint:-options -cp "$WORK/jarde-compiled/ClassVariableContract" \
  -d "$WORK/jarde-compiled/ClassVariableContract" "$FIXTURES/positive/StringContract.java" \
  "$FIXTURES/positive/InterfaceVariableRunner.java" > "$WORK/InterfaceVariableRunner-jarde-compile.log" 2>&1
interface_caller_status=$?
set -e
printf 'Jarde interface generic caller compile exit=%s\n' "$interface_caller_status"
cat "$WORK/InterfaceVariableRunner-jarde-compile.log"
if [ "$interface_caller_status" -ne 0 ]; then exit "$interface_caller_status"; fi
java -Xverify:all -cp "$WORK/jadx-compiled/ClassVariableContract" classvars.InterfaceVariableRunner
java -Xverify:all -cp "$WORK/jarde-compiled/ClassVariableContract" \
  classvars.InterfaceVariableRunner

printf '%s\n' '== Jarde no-debug class-source and generic caller recompilation/execution =='
for class in ClassVariableBoundary BoundedClassVariableBoundary ClassVariableContract; do
  mkdir -p "$WORK/jarde-none-compiled/$class"
  cargo run -q -p jarde-cli -- class-source --input "$WORK/original-none/classvars/$class.class" \
    --class "classvars.$class" --policy single-class > "$WORK/jarde-none-compiled/$class/$class.java" \
    2> "$WORK/jarde-none-compiled/$class/class-source.log"
  case "$class" in
    ClassVariableBoundary) runner_source=ClassVariableRunner; runner_class=ClassVariableRunner ;;
    BoundedClassVariableBoundary) runner_source=BoundedClassVariableRunner; runner_class=BoundedClassVariableRunner ;;
    ClassVariableContract)
      runner_source=InterfaceVariableRunner; runner_class=InterfaceVariableRunner
      cp "$FIXTURES/positive/StringContract.java" "$WORK/jarde-none-compiled/$class/"
      ;;
  esac
  if [ "$class" = ClassVariableContract ]; then
    javac --release 8 -Xlint:-options -d "$WORK/jarde-none-compiled/$class" \
      "$WORK/jarde-none-compiled/$class/$class.java" \
      "$WORK/jarde-none-compiled/$class/StringContract.java" \
      "$FIXTURES/positive/$runner_source.java"
  else
    javac --release 8 -Xlint:-options -d "$WORK/jarde-none-compiled/$class" \
      "$WORK/jarde-none-compiled/$class/$class.java" \
      "$FIXTURES/positive/$runner_source.java"
  fi
  runner="classvars.$runner_class"
  java -Xverify:all -cp "$WORK/jarde-none-compiled/$class" "$runner"
done

printf '%s\n' '== source compilation of current Jarde boundary classes =='
for class in ParentMismatch InterfaceMismatch UnboundClassVariable MethodShadowBoundary OuterScopeBoundary-Inner; do
  case "$class" in
    OuterScopeBoundary-Inner)
      source="$WORK/jarde/OuterScopeBoundary-Inner.java"
      filename='OuterScopeBoundary$Inner.java' ;;
    *)
      source="$WORK/jarde/$class.java"
      filename="$class.java" ;;
  esac
  mkdir -p "$WORK/jarde-compiled/$class/classvars"
  cp "$source" "$WORK/jarde-compiled/$class/classvars/$filename"
  set +e
  javac --release 8 -Xlint:-options -cp "$WORK/original-g" -d "$WORK/jarde-compiled/$class" \
    "$WORK/jarde-compiled/$class/classvars/$filename" > "$WORK/$class-jarde-source-compile.log" 2>&1
  source_status=$?
  set -e
  printf 'Jarde %s source compile exit=%s\n' "$class" "$source_status"
  if [ "$source_status" -ne 0 ]; then cat "$WORK/$class-jarde-source-compile.log"; fi
done

printf '%s\n' '== tool versions and frozen SHA-256 =='
java -version 2>&1 | head -3
javac -version
jadx --version
cargo --version
find "$FIXTURES" -type f -name '*.java' -print0 | xargs -0 shasum -a 256
shasum -a 256 "$EVIDENCE/patch_signature.py" "$EVIDENCE/replay.sh" \
  "$WORK/original-g/classvars/ClassVariableBoundary.class" \
  "$WORK/original-none/classvars/ClassVariableBoundary.class" \
  "$WORK/original-g/classvars/BoundedClassVariableBoundary.class" \
  "$WORK/original-none/classvars/BoundedClassVariableBoundary.class" \
  "$WORK/original-g/classvars/ClassVariableContract.class" \
  "$WORK/original-none/classvars/ClassVariableContract.class" \
  "$WORK/patched/classvars/ParentMismatch.class" "$WORK/patched/classvars/InterfaceMismatch.class" \
  "$WORK/patched/classvars/UnboundClassVariable.class" \
  "$WORK/original-g/classvars/ParentMismatch.class" "$WORK/original-none/classvars/ParentMismatch.class" \
  "$WORK/original-g/classvars/InterfaceMismatch.class" "$WORK/original-none/classvars/InterfaceMismatch.class" \
  "$WORK/original-g/classvars/UnboundClassVariable.class" "$WORK/original-none/classvars/UnboundClassVariable.class" \
  "$WORK/original-g/classvars/MethodShadowBoundary.class" "$WORK/original-none/classvars/MethodShadowBoundary.class" \
  "$WORK/original-g/classvars/OuterScopeBoundary\$Inner.class" "$WORK/original-none/classvars/OuterScopeBoundary\$Inner.class" \
  "$WORK/jadx/ClassVariableBoundary/sources/classvars/ClassVariableBoundary.java" \
  "$WORK/jadx-none/ClassVariableBoundary/sources/classvars/ClassVariableBoundary.java" \
  "$WORK/jadx-none/BoundedClassVariableBoundary/sources/classvars/BoundedClassVariableBoundary.java" \
  "$WORK/jadx-none/ClassVariableContract/sources/classvars/ClassVariableContract.java" \
  "$WORK/jarde/ClassVariableBoundary.java" "$WORK/jarde/BoundedClassVariableBoundary.java" \
  "$WORK/jarde/ClassVariableContract.java" "$WORK/jarde/ParentMismatch.java" \
  "$WORK/jarde/InterfaceMismatch.java" "$WORK/jarde/UnboundClassVariable.java" \
  "$WORK/jarde/MethodShadowBoundary.java" "$WORK/jarde/OuterScopeBoundary-Inner.java"
printf 'replay complete; private Cargo target and temporary outputs removed on exit\n'
