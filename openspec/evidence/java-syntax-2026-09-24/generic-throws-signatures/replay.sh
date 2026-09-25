#!/bin/sh
set -eu

ROOT=$(git rev-parse --show-toplevel)
FIXTURE="$ROOT/openspec/evidence/java-syntax-2026-09-24/generic-throws-signatures/fixture"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-generic-throws.XXXXXX")
export CARGO_TARGET_DIR="$WORK/cargo-target"
trap 'python3 -c "import shutil,sys; shutil.rmtree(sys.argv[1], ignore_errors=True)" "$WORK"' EXIT

mkdir -p "$WORK/original-g" "$WORK/original-none" "$WORK/jadx-g" "$WORK/jadx-none" "$WORK/jarde"
for variant in g none; do
  case "$variant" in
    g) debug=-g ;;
    none) debug=-g:none ;;
  esac
  javac --release 8 -Xlint:-options "$debug" -d "$WORK/original-$variant" \
    "$FIXTURE/GenericThrowsBoundary.java" "$FIXTURE/GenericThrowsCaller.java"
  java -Xverify:all -cp "$WORK/original-$variant" genericthrows.GenericThrowsCaller
  javap -v "$WORK/original-$variant/genericthrows/GenericThrowsBoundary.class" \
    > "$WORK/$variant.javap"
  echo "== original $variant Signature and Exceptions =="
  rg -n 'Signature:|Exceptions:|invoke\(' "$WORK/$variant.javap"

  jadx -d "$WORK/jadx-$variant" \
    "$WORK/original-$variant/genericthrows/GenericThrowsBoundary.class" >/dev/null
  echo "== JADX $variant declaration =="
  rg -n 'class |invoke\(' "$WORK/jadx-$variant/sources/genericthrows/GenericThrowsBoundary.java"

  mkdir -p "$WORK/jarde/$variant"
  cargo run -q -p jarde-cli -- class-source \
    --input "$WORK/original-$variant/genericthrows/GenericThrowsBoundary.class" \
    --class genericthrows.GenericThrowsBoundary --policy single-class \
    > "$WORK/jarde/$variant/GenericThrowsBoundary.java" 2> "$WORK/jarde/$variant/run.log"
  echo "== Jarde $variant declaration =="
  rg -n 'class |invoke\(|Signature projection' "$WORK/jarde/$variant/GenericThrowsBoundary.java"

  for engine in jadx jarde; do
    mkdir -p "$WORK/$engine-$variant-compiled"
    case "$engine" in
      jadx) source="$WORK/jadx-$variant/sources/genericthrows/GenericThrowsBoundary.java" ;;
      jarde) source="$WORK/jarde/$variant/GenericThrowsBoundary.java" ;;
    esac
    javac --release 8 -Xlint:-options -d "$WORK/$engine-$variant-compiled" "$source"
    set +e
    javac --release 8 -Xlint:-options -cp "$WORK/$engine-$variant-compiled" \
      -d "$WORK/$engine-$variant-compiled" "$FIXTURE/GenericThrowsCaller.java" \
      > "$WORK/$engine-$variant-caller.log" 2>&1
    result=$?
    set -e
    printf '%s %s typed caller javac exit=%s\n' "$engine" "$variant" "$result"
    cat "$WORK/$engine-$variant-caller.log"
    if [ "$result" -eq 0 ]; then
      java -Xverify:all -cp "$WORK/$engine-$variant-compiled" genericthrows.GenericThrowsCaller
    fi
  done
done

echo '== Versions and SHA-256 =='
java -version 2>&1 | head -3
jadx --version
cargo --version
shasum -a 256 "$FIXTURE"/*.java "$WORK"/original-*/genericthrows/GenericThrowsBoundary.class \
  "$WORK"/jadx-*/sources/genericthrows/GenericThrowsBoundary.java \
  "$WORK"/jarde/*/GenericThrowsBoundary.java
