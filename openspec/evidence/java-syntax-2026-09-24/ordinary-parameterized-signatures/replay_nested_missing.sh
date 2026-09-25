#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-24/ordinary-parameterized-signatures"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-nested-missing.XXXXXX")
export CARGO_TARGET_DIR="$WORK/cargo-target"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/original-g" "$WORK/original-none" "$WORK/patched-g" "$WORK/patched-none" \
  "$WORK/runtime" "$WORK/payload-only" "$WORK/stub" "$WORK/jadx-missing" "$WORK/jadx-provided" \
  "$WORK/compile-jadx-missing" "$WORK/compile-jadx-provided" "$WORK/compile-jarde-single" \
  "$WORK/compile-jarde-jar-missing" "$WORK/compile-jarde-jar-provided" "$WORK/compile-jarde-none" \
  "$WORK/direct-missing" \
  "$WORK/direct-provided"

for debug in g none; do
  if [ "$debug" = g ]; then DEBUG_FLAG=-g; else DEBUG_FLAG=-g:none; fi
  javac --release 8 -Xlint:-options "$DEBUG_FLAG" -d "$WORK/original-$debug" \
    "$EVIDENCE/NestedMissingSignature.java" "$EVIDENCE/Payload.java" "$EVIDENCE/DirectMissingControl.java"
  mkdir -p "$WORK/patched-$debug/probe"
  python3 "$EVIDENCE/patch_nested_signature.py" \
    "$WORK/original-$debug/probe/NestedMissingSignature.class" "$WORK/patched-$debug/probe/NestedMissingSignature.class"
done
javac --release 8 -Xlint:-options -cp "$WORK/original-g" -d "$WORK/runtime" "$EVIDENCE/NestedSignatureRunner.java"
javac --release 8 -Xlint:-options -d "$WORK/stub" "$EVIDENCE/Missing.java"
mkdir -p "$WORK/payload-only/probe"
cp "$WORK/original-g/probe/Payload.class" "$WORK/payload-only/probe/Payload.class"

printf '%s\n' '== javap original and patched Signature/descriptor =='
javap -v "$WORK/original-g/probe/NestedMissingSignature.class" > "$WORK/original.javap"
javap -v "$WORK/patched-g/probe/NestedMissingSignature.class" > "$WORK/patched.javap"
 javap -v "$WORK/original-none/probe/NestedMissingSignature.class" > "$WORK/original-none.javap"
 javap -v "$WORK/patched-none/probe/NestedMissingSignature.class" > "$WORK/patched-none.javap"
rg -n -A2 -B2 'descriptor: \(Ljava/util/List;\)Ljava/util/List;|Signature: #|Ljava/util/List<L(probe/Payload|probe/Missing);>' "$WORK/original.javap" "$WORK/patched.javap"

printf '%s\n' '== original reflection, patched execution, patched reflection without/with Missing =='
java -Xverify:all -cp "$WORK/original-g:$WORK/runtime" probe.NestedSignatureRunner reflect
java -Xverify:all -cp "$WORK/patched-g:$WORK/original-g:$WORK/runtime" probe.NestedSignatureRunner execute
set +e
java -Xverify:all -cp "$WORK/patched-g:$WORK/original-g:$WORK/runtime" probe.NestedSignatureRunner reflect > "$WORK/reflection-missing.txt" 2>&1
REFLECT_MISSING_STATUS=$?
set -e
printf 'patched reflection without Missing exit=%s\n' "$REFLECT_MISSING_STATUS"
cat "$WORK/reflection-missing.txt"
java -Xverify:all -cp "$WORK/patched-g:$WORK/original-g:$WORK/stub:$WORK/runtime" probe.NestedSignatureRunner reflect
java -Xverify:all -cp "$WORK/original-none:$WORK/runtime" probe.NestedSignatureRunner reflect
java -Xverify:all -cp "$WORK/patched-none:$WORK/original-none:$WORK/runtime" probe.NestedSignatureRunner execute
set +e
java -Xverify:all -cp "$WORK/patched-none:$WORK/original-none:$WORK/runtime" probe.NestedSignatureRunner reflect > "$WORK/reflection-none-missing.txt" 2>&1
REFLECT_NONE_STATUS=$?
set -e
printf 'patched -g:none reflection without Missing exit=%s\n' "$REFLECT_NONE_STATUS"
rg -n 'TypeNotPresentException|ClassNotFoundException' "$WORK/reflection-none-missing.txt"
java -Xverify:all -cp "$WORK/patched-none:$WORK/original-none:$WORK/stub:$WORK/runtime" probe.NestedSignatureRunner reflect

printf '%s\n' '== JADX 1.5.6: target alone and target with reference class in input JAR =='
jadx -d "$WORK/jadx-missing" "$WORK/patched-g/probe/NestedMissingSignature.class"
jadx -d "$WORK/jadx-none" "$WORK/patched-none/probe/NestedMissingSignature.class"
jar --create --file "$WORK/target-only.jar" -C "$WORK/patched-g" probe/NestedMissingSignature.class -C "$WORK/original-g" probe/Payload.class
jar --create --file "$WORK/target-plus-reference.jar" -C "$WORK/patched-g" probe/NestedMissingSignature.class -C "$WORK/original-g" probe/Payload.class -C "$WORK/stub" probe/Missing.class
jadx -d "$WORK/jadx-provided" "$WORK/target-plus-reference.jar"
find "$WORK/jadx-missing" "$WORK/jadx-provided" -name 'NestedMissingSignature.java' -exec sh -c 'echo "--- $1"; rg -n "List<|echo\(" "$1"' sh {} \;

printf '%s\n' '== Jarde CLI single-class and plain-jar (reference absent/present) =='
cargo run -q -p jarde-cli -- class-source --input "$WORK/patched-g/probe/NestedMissingSignature.class" \
  --class probe.NestedMissingSignature --policy single-class > "$WORK/jarde-single.java" 2> "$WORK/jarde-single.log"
cargo run -q -p jarde-cli -- class-source --input "$WORK/target-only.jar" \
  --class probe.NestedMissingSignature --policy plain-jar > "$WORK/jarde-jar-missing.java" 2> "$WORK/jarde-jar-missing.log"
cargo run -q -p jarde-cli -- class-source --input "$WORK/target-plus-reference.jar" \
  --class probe.NestedMissingSignature --policy plain-jar > "$WORK/jarde-jar-provided.java" 2> "$WORK/jarde-jar-provided.log"
for output in "$WORK/jarde-single.java" "$WORK/jarde-jar-missing.java" "$WORK/jarde-jar-provided.java"; do
  echo "--- $output"
  rg -n 'class NestedMissingSignature|List<|echo\(' "$output" || true
done
cargo run -q -p jarde-cli -- class-source --input "$WORK/patched-none/probe/NestedMissingSignature.class" \
  --class probe.NestedMissingSignature --policy single-class > "$WORK/jarde-none.java" 2> "$WORK/jarde-none.log"
rg -n 'List<|echo\(' "$WORK/jarde-none.java"

printf '%s\n' '== compile generated sources without/with reference class =='
for case in jadx-missing jadx-provided jarde-single jarde-jar-missing jarde-jar-provided jarde-none; do
  if [ "$case" = jadx-missing ] || [ "$case" = jadx-provided ]; then
    source=$(find "$WORK/$case" -name NestedMissingSignature.java -print -quit)
  else
    source="$WORK/compile-$case/NestedMissingSignature.java"
    if [ "$case" = jarde-none ]; then
      cp "$WORK/jarde-none.java" "$source"
    else
      cp "$WORK/$case.java" "$source"
    fi
  fi
  case "$case" in
    *provided) classpath="$WORK/payload-only:$WORK/stub" ;;
    *) classpath="$WORK/payload-only" ;;
  esac
  set +e
  javac --release 8 -Xlint:-options -cp "$classpath" -d "$WORK/compile-$case" "$source" > "$WORK/compile-$case.log" 2>&1
  compile_status=$?
  set -e
  printf '%s source compile exit=%s\n' "$case" "$compile_status"
  if [ "$compile_status" -ne 0 ]; then cat "$WORK/compile-$case.log"; fi
  if [ "$compile_status" -eq 0 ] && { [ "$case" = jadx-provided ] || [ "$case" = jarde-jar-provided ]; }; then
    java -Xverify:all -cp "$WORK/compile-$case:$WORK/payload-only:$WORK/stub:$WORK/runtime" probe.NestedSignatureRunner execute
    java -Xverify:all -cp "$WORK/compile-$case:$WORK/payload-only:$WORK/stub:$WORK/runtime" probe.NestedSignatureRunner reflect
  fi
done

mkdir -p "$WORK/compile-jarde-none-provided"
javac --release 8 -Xlint:-options -cp "$WORK/payload-only:$WORK/stub" \
  -d "$WORK/compile-jarde-none-provided" "$WORK/compile-jarde-none/NestedMissingSignature.java"
java -Xverify:all -cp "$WORK/compile-jarde-none-provided:$WORK/payload-only:$WORK/stub:$WORK/runtime" \
  probe.NestedSignatureRunner execute
java -Xverify:all -cp "$WORK/compile-jarde-none-provided:$WORK/payload-only:$WORK/stub:$WORK/runtime" \
  probe.NestedSignatureRunner reflect

printf '%s\n' '== direct descriptor-reference control =='
cargo run -q -p jarde-cli -- class-source --input "$WORK/original-g/probe/DirectMissingControl.class" \
  --class probe.DirectMissingControl --policy single-class > "$WORK/direct-source.java" 2> "$WORK/direct-source.log"
rg -n 'Payload|echo\(' "$WORK/direct-source.java"
cp "$WORK/direct-source.java" "$WORK/direct-missing/DirectMissingControl.java"
cp "$WORK/direct-source.java" "$WORK/direct-provided/DirectMissingControl.java"
set +e
javac --release 8 -Xlint:-options -d "$WORK/direct-missing" "$WORK/direct-missing/DirectMissingControl.java" > "$WORK/direct-missing.log" 2>&1
DIRECT_MISSING_STATUS=$?
set -e
printf 'direct descriptor source compile without Payload exit=%s\n' "$DIRECT_MISSING_STATUS"
cat "$WORK/direct-missing.log"
javac --release 8 -Xlint:-options -cp "$WORK/payload-only" -d "$WORK/direct-provided" "$WORK/direct-provided/DirectMissingControl.java"

printf '%s\n' '== tool versions and SHA-256 =='
java -version 2>&1 | head -3
javac -version
jadx --version
cargo --version
shasum -a 256 "$EVIDENCE/NestedMissingSignature.java" "$EVIDENCE/Payload.java" \
  "$EVIDENCE/DirectMissingControl.java" "$EVIDENCE/NestedSignatureRunner.java" \
  "$EVIDENCE/Missing.java" "$EVIDENCE/patch_nested_signature.py" \
  "$WORK/original-g/probe/NestedMissingSignature.class" "$WORK/patched-g/probe/NestedMissingSignature.class" \
  "$WORK/original-none/probe/NestedMissingSignature.class" "$WORK/patched-none/probe/NestedMissingSignature.class" \
  "$(find "$WORK/jadx-missing" -name NestedMissingSignature.java -print -quit)" \
  "$(find "$WORK/jadx-provided" -name NestedMissingSignature.java -print -quit)" \
  "$WORK/jarde-single.java" "$WORK/jarde-jar-missing.java" "$WORK/jarde-jar-provided.java" "$WORK/jarde-none.java"
printf 'replay completed; private cargo target removed on exit\n'
