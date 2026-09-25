#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-24/field-generic-signatures"
FIXTURES="$EVIDENCE/fixture"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-field-signatures.XXXXXX")
export CARGO_TARGET_DIR="$WORK/cargo-target"
trap 'rm -rf "$WORK"' EXIT
mkdir -p "$WORK/original-g" "$WORK/original-none" "$WORK/runners" "$WORK/patched" \
  "$WORK/patched/fieldsig" "$WORK/patched-unbound/fieldsig" "$WORK/jadx-positive" "$WORK/jadx-mismatch" \
  "$WORK/jadx-conflict" "$WORK/jarde" "$WORK/jarde-none"

find "$FIXTURES" -name '*.java' -print0 | xargs -0 javac --release 8 -Xlint:-options -g -d "$WORK/original-g"
find "$FIXTURES" -name '*.java' -print0 | xargs -0 javac --release 8 -Xlint:-options -g:none -d "$WORK/original-none"
javac --release 8 -Xlint:-options -cp "$WORK/original-g" -d "$WORK/runners" \
  "$FIXTURES/FieldSignatureRunner.java" "$FIXTURES/FieldReflectionRunner.java" \
  "$FIXTURES/FieldSignatureConflictRunner.java" "$FIXTURES/MutatedFieldRunner.java" \
  "$FIXTURES/StandaloneFieldRunner.java" "$FIXTURES/StandaloneFieldReflectionRunner.java"

printf '%s\n' '== original Java 8 class: verifier, caller values, field reflection =='
java -Xverify:all -cp "$WORK/original-g:$WORK/runners" fieldsig.FieldSignatureRunner
java -Xverify:all -cp "$WORK/original-g:$WORK/runners" fieldsig.FieldSignatureConflictRunner
java -Xverify:all -cp "$WORK/original-g:$WORK/runners" fieldsig.StandaloneFieldRunner

printf '%s\n' '== javap field descriptors and Signatures (-g / -g:none) =='
for variant in original-g original-none; do
  javap -v "$WORK/$variant/fieldsig/FieldSignatureBoundary.class" > "$WORK/$variant-boundary.javap"
  javap -v "$WORK/$variant/fieldsig/StandaloneFieldBoundary.class" > "$WORK/$variant-standalone.javap"
  javap -v "$WORK/$variant/fieldsig/FieldSignatureConflict.class" > "$WORK/$variant-conflict.javap"
  echo "--- $variant StandaloneFieldBoundary"
  rg -n 'names;|numbers;|current;|vector;|sink;|TOKEN;|LABEL;|descriptor:|Signature:|ConstantValue:' \
    "$WORK/$variant-standalone.javap" | tail -40
  if rg -n 'Fieldref .*// fieldsig/StandaloneFieldBoundary\.' "$WORK/$variant-standalone.javap"; then
    echo "unexpected field reference in standalone fixture"
    exit 1
  else
    echo "StandaloneFieldBoundary has no self Fieldref"
  fi
  echo "--- $variant FieldSignatureBoundary"
  rg -n 'names;|numbers;|current;|vector;|sink;|TOKEN;|LABEL;|descriptor:|Signature:' "$WORK/$variant-boundary.javap" | tail -35
  echo "--- $variant FieldSignatureConflict"
  rg -n 'items;|descriptor:|Signature:' "$WORK/$variant-conflict.javap" | tail -12
done

printf '%s\n' '== verifier-valid field Signature mutations =='
python3 "$EVIDENCE/mutate_field_signature.py" \
  "$WORK/original-g/fieldsig/FieldSignatureBoundary.class" \
  "$WORK/patched/fieldsig/FieldSignatureBoundary.class" names \
  'Ljava/util/List<Ljava/lang/String;>;' 'Ljava/lang/String;'
python3 "$EVIDENCE/mutate_field_signature.py" \
  "$WORK/original-g/fieldsig/FieldSignatureConflict.class" \
  "$WORK/patched/fieldsig/FieldSignatureConflict.class" items \
  'Ljava/util/List<Ljava/lang/Object;>;' 'Ljava/util/List<Ljava/lang/String;>;'
python3 "$EVIDENCE/mutate_field_signature.py" \
  "$WORK/original-g/fieldsig/FieldSignatureBoundary.class" \
  "$WORK/patched-unbound/fieldsig/FieldSignatureBoundary.class" current 'TT;' 'TV;'
javap -v "$WORK/patched/fieldsig/FieldSignatureBoundary.class" > "$WORK/mismatch.javap"
javap -v "$WORK/patched/fieldsig/FieldSignatureConflict.class" > "$WORK/conflict.javap"
 javap -v "$WORK/patched-unbound/fieldsig/FieldSignatureBoundary.class" > "$WORK/unbound.javap"
rg -n 'names;|descriptor:|Signature:' "$WORK/mismatch.javap" | tail -12
rg -n 'items;|descriptor:|Signature:' "$WORK/conflict.javap" | tail -8
rg -n 'current;|descriptor:|Signature:' "$WORK/unbound.javap" | tail -8
java -Xverify:all -cp "$WORK/patched:$WORK/original-g:$WORK/runners" \
  fieldsig.MutatedFieldRunner names
java -Xverify:all -cp "$WORK/patched:$WORK/original-g:$WORK/runners" \
  fieldsig.FieldSignatureConflictRunner
java -Xverify:all -cp "$WORK/patched-unbound:$WORK/original-g:$WORK/runners" \
  fieldsig.MutatedFieldRunner current

printf '%s\n' '== JADX 1.5.6 positive and mutated field declarations =='
jadx -d "$WORK/jadx-positive" "$WORK/original-g/fieldsig/StandaloneFieldBoundary.class"
jadx -d "$WORK/jadx-body" "$WORK/original-g/fieldsig/FieldSignatureBoundary.class"
jadx -d "$WORK/jadx-mismatch" "$WORK/patched/fieldsig/FieldSignatureBoundary.class"
jadx -d "$WORK/jadx-unbound" "$WORK/patched-unbound/fieldsig/FieldSignatureBoundary.class"
jadx -d "$WORK/jadx-conflict" "$WORK/patched/fieldsig/FieldSignatureConflict.class"
for source in "$WORK/jadx-positive/sources/fieldsig/StandaloneFieldBoundary.java" \
  "$WORK/jadx-mismatch/sources/fieldsig/FieldSignatureBoundary.java" \
  "$WORK/jadx-unbound/sources/fieldsig/FieldSignatureBoundary.java" \
  "$WORK/jadx-conflict/sources/fieldsig/FieldSignatureConflict.java"; do
  echo "--- $source"
  rg -n 'class |List<|T\[\]|current|vector|sink|names|items|addInteger|first\(' "$source" | head -28
done
mkdir -p "$WORK/jadx-positive-compiled"
javac --release 8 -Xlint:-options -d "$WORK/jadx-positive-compiled" \
  "$WORK/jadx-positive/sources/fieldsig/StandaloneFieldBoundary.java" \
  "$WORK/jadx-body/sources/fieldsig/FieldSignatureBoundary.java" \
  "$FIXTURES/StandaloneFieldRunner.java"
java -Xverify:all -cp "$WORK/jadx-positive-compiled" fieldsig.StandaloneFieldRunner
mkdir -p "$WORK/jadx-body-compiled"
javac --release 8 -Xlint:-options -d "$WORK/jadx-body-compiled" \
  "$WORK/jadx-body/sources/fieldsig/FieldSignatureBoundary.java" \
  "$FIXTURES/FieldSignatureRunner.java" "$FIXTURES/FieldReflectionRunner.java"
java -Xverify:all -cp "$WORK/jadx-body-compiled" fieldsig.FieldSignatureRunner
java -Xverify:all -cp "$WORK/jadx-body-compiled" fieldsig.FieldReflectionRunner
mkdir -p "$WORK/jadx-conflict-compiled"
set +e
javac --release 8 -Xlint:-options -d "$WORK/jadx-conflict-compiled" \
  "$WORK/jadx-conflict/sources/fieldsig/FieldSignatureConflict.java" \
  "$FIXTURES/FieldSignatureConflictRunner.java" > "$WORK/jadx-conflict-compile.log" 2>&1
jadx_conflict_compile=$?
set -e
printf 'JADX mutated conflict source compile exit=%s\n' "$jadx_conflict_compile"
cat "$WORK/jadx-conflict-compile.log"
mkdir -p "$WORK/jadx-unbound-compiled"
set +e
javac --release 8 -Xlint:-options -d "$WORK/jadx-unbound-compiled" \
  "$WORK/jadx-unbound/sources/fieldsig/FieldSignatureBoundary.java" \
  > "$WORK/jadx-unbound-compile.log" 2>&1
jadx_unbound_compile=$?
set -e
printf 'JADX unbound field source compile exit=%s\n' "$jadx_unbound_compile"
cat "$WORK/jadx-unbound-compile.log"

printf '%s\n' '== current Jarde class-source positive, no-debug, and mutated fields =='
for variant in original-g original-none; do
  out="$WORK/jarde/$variant"
  mkdir -p "$out"
  cargo run -q -p jarde-cli -- class-source --input "$WORK/$variant/fieldsig/FieldSignatureBoundary.class" \
    --class fieldsig.FieldSignatureBoundary --policy single-class > "$out/FieldSignatureBoundary.java" \
    2> "$out/class-source.log"
  echo "--- Jarde $variant FieldSignatureBoundary"
  rg -n 'class |List<|T\[\]|current|vector|sink|names|jvm_signature' "$out/FieldSignatureBoundary.java" | head -28
  mkdir -p "$out/compiled"
  set +e
  javac --release 8 -Xlint:-options -d "$out/compiled" "$out/FieldSignatureBoundary.java" \
    "$FIXTURES/FieldReflectionRunner.java" > "$out/class-compile.log" 2>&1
  class_compile=$?
  set -e
  printf 'Jarde %s body-bearing class compile exit=%s\n' "$variant" "$class_compile"
  cat "$out/class-compile.log"
  if [ "$class_compile" -eq 0 ]; then
    set +e
    javac --release 8 -Xlint:-options -cp "$out/compiled" -d "$out/compiled" \
      "$FIXTURES/FieldSignatureRunner.java" > "$out/generic-caller-compile.log" 2>&1
    caller_compile=$?
    set -e
    printf 'Jarde %s strict generic caller compile exit=%s\n' "$variant" "$caller_compile"
    cat "$out/generic-caller-compile.log"
    java -Xverify:all -cp "$out/compiled" fieldsig.FieldReflectionRunner
  else
    printf 'Jarde %s body-bearing runtime comparison skipped: generated source did not compile\n' "$variant"
  fi
  cargo run -q -p jarde-cli -- class-source --input "$WORK/$variant/fieldsig/StandaloneFieldBoundary.class" \
    --class fieldsig.StandaloneFieldBoundary --policy single-class > "$out/StandaloneFieldBoundary.java" \
    2> "$out/standalone-class-source.log"
  echo "--- Jarde $variant StandaloneFieldBoundary"
  rg -n 'class |List<|T\[\]|current|vector|sink|names|jvm_signature' "$out/StandaloneFieldBoundary.java" | head -28
  mkdir -p "$out/standalone-compiled"
  javac --release 8 -Xlint:-options -d "$out/standalone-compiled" "$out/StandaloneFieldBoundary.java"
  set +e
  javac --release 8 -Xlint:-options -cp "$out/standalone-compiled" -d "$out/standalone-compiled" \
    "$FIXTURES/StandaloneFieldRunner.java" > "$out/standalone-caller-compile.log" 2>&1
  standalone_caller=$?
  set -e
  printf 'Jarde %s standalone strict generic caller compile exit=%s\n' "$variant" "$standalone_caller"
  cat "$out/standalone-caller-compile.log"
  javac --release 8 -Xlint:-options -cp "$out/standalone-compiled" -d "$out/standalone-compiled" \
    "$FIXTURES/StandaloneFieldReflectionRunner.java"
  java -Xverify:all -cp "$out/standalone-compiled" fieldsig.StandaloneFieldReflectionRunner
done

for case in mismatch conflict; do
  case "$case" in
    mismatch) input="$WORK/patched/fieldsig/FieldSignatureBoundary.class"; class=FieldSignatureBoundary ;;
    conflict) input="$WORK/patched/fieldsig/FieldSignatureConflict.class"; class=FieldSignatureConflict ;;
  esac
  out="$WORK/jarde/$case"
  mkdir -p "$out"
  cargo run -q -p jarde-cli -- class-source --input "$input" --class "fieldsig.$class" \
    --policy single-class > "$out/$class.java" 2> "$out/class-source.log"
  echo "--- Jarde mutated $case"
  rg -n 'class |List<|items|names|generic|Signature|jvm_signature' "$out/$class.java" | head -28
done

mkdir -p "$WORK/jarde/unbound"
cargo run -q -p jarde-cli -- class-source --input \
  "$WORK/patched-unbound/fieldsig/FieldSignatureBoundary.class" \
  --class fieldsig.FieldSignatureBoundary --policy single-class \
  > "$WORK/jarde/unbound/FieldSignatureBoundary.java" 2> "$WORK/jarde/unbound/class-source.log"
echo '--- Jarde mutated unbound field Signature'
rg -n 'class |current|java.lang.Object|generic|Signature|scope' \
  "$WORK/jarde/unbound/FieldSignatureBoundary.java" | head -24

mkdir -p "$WORK/jarde/conflict-compiled"
set +e
javac --release 8 -Xlint:-options -d "$WORK/jarde/conflict-compiled" \
  "$WORK/jarde/conflict/FieldSignatureConflict.java" \
  "$FIXTURES/FieldSignatureConflictRunner.java" > "$WORK/jarde/conflict-compile.log" 2>&1
jarde_conflict_compile=$?
set -e
printf 'Jarde mutated conflict source compile exit=%s\n' "$jarde_conflict_compile"
cat "$WORK/jarde/conflict-compile.log"

printf '%s\n' '== tool versions and frozen SHA-256 =='
java -version 2>&1 | head -3
javac -version
jadx --version
cargo --version
find "$FIXTURES" -type f -name '*.java' -print0 | xargs -0 shasum -a 256
shasum -a 256 "$EVIDENCE/mutate_field_signature.py" "$EVIDENCE/replay.sh" \
  "$WORK/original-g/fieldsig/FieldSignatureBoundary.class" \
  "$WORK/original-none/fieldsig/FieldSignatureBoundary.class" \
  "$WORK/original-g/fieldsig/StandaloneFieldBoundary.class" \
  "$WORK/original-none/fieldsig/StandaloneFieldBoundary.class" \
  "$WORK/original-g/fieldsig/FieldSignatureConflict.class" \
  "$WORK/original-none/fieldsig/FieldSignatureConflict.class" \
  "$WORK/patched/fieldsig/FieldSignatureBoundary.class" "$WORK/patched/fieldsig/FieldSignatureConflict.class" \
  "$WORK/patched-unbound/fieldsig/FieldSignatureBoundary.class" \
  "$WORK/jadx-positive/sources/fieldsig/StandaloneFieldBoundary.java" \
  "$WORK/jadx-body/sources/fieldsig/FieldSignatureBoundary.java" \
  "$WORK/jadx-mismatch/sources/fieldsig/FieldSignatureBoundary.java" \
  "$WORK/jadx-unbound/sources/fieldsig/FieldSignatureBoundary.java" \
  "$WORK/jadx-conflict/sources/fieldsig/FieldSignatureConflict.java" \
  "$WORK/jarde/original-g/FieldSignatureBoundary.java" \
  "$WORK/jarde/original-none/FieldSignatureBoundary.java" \
  "$WORK/jarde/original-g/StandaloneFieldBoundary.java" \
  "$WORK/jarde/original-none/StandaloneFieldBoundary.java" \
  "$WORK/jarde/mismatch/FieldSignatureBoundary.java" \
  "$WORK/jarde/unbound/FieldSignatureBoundary.java" \
  "$WORK/jarde/conflict/FieldSignatureConflict.java"
printf 'replay complete; private Cargo target and temporary outputs removed on exit\n'
