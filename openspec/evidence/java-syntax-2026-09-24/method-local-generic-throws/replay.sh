#!/bin/sh
set -eu

ROOT=$(git rev-parse --show-toplevel)
FIXTURE="$ROOT/openspec/evidence/java-syntax-2026-09-24/method-local-generic-throws/fixture"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-method-local-throws.XXXXXX")
export CARGO_TARGET_DIR="$WORK/cargo-target"
trap 'python3 -c "import shutil,sys; shutil.rmtree(sys.argv[1], ignore_errors=True)" "$WORK"' EXIT

for variant in g none; do
  case "$variant" in
    g) debug=-g ;;
    none) debug=-g:none ;;
  esac
  mkdir -p "$WORK/original-$variant" "$WORK/jadx-$variant" "$WORK/jarde-$variant"
  javac --release 8 -Xlint:-options "$debug" -d "$WORK/original-$variant" \
    "$FIXTURE/MethodThrowsBoundary.java" "$FIXTURE/MethodThrowsCaller.java"
  java -Xverify:all -cp "$WORK/original-$variant" methodthrows.MethodThrowsCaller
  javap -v "$WORK/original-$variant/methodthrows/MethodThrowsBoundary.class" \
    > "$WORK/$variant.javap"
  rg -n 'Signature:|Exceptions:|raise\(' "$WORK/$variant.javap"

  jadx -d "$WORK/jadx-$variant" \
    "$WORK/original-$variant/methodthrows/MethodThrowsBoundary.class" >/dev/null
  cargo run -q -p jarde-cli -- class-source \
    --input "$WORK/original-$variant/methodthrows/MethodThrowsBoundary.class" \
    --class methodthrows.MethodThrowsBoundary --policy single-class \
    > "$WORK/jarde-$variant/MethodThrowsBoundary.java" \
    2> "$WORK/jarde-$variant/run.log"

  for engine in jadx jarde; do
    if [ "$engine" = jadx ]; then
      source="$WORK/jadx-$variant/sources/methodthrows/MethodThrowsBoundary.java"
    else
      source="$WORK/jarde-$variant/MethodThrowsBoundary.java"
    fi
    printf '== %s %s declaration ==\n' "$engine" "$variant"
    rg -n 'class |raise\(|Signature projection' "$source"
    output="$WORK/$engine-$variant-compiled"
    mkdir -p "$output"
    javac --release 8 -Xlint:-options -d "$output" \
      "$source" "$FIXTURE/MethodThrowsReflect.java"
    java -Xverify:all -cp "$output" methodthrows.MethodThrowsReflect
    set +e
    javac --release 8 -Xlint:-options -cp "$output" -d "$output" \
      "$FIXTURE/MethodThrowsCaller.java" \
      > "$WORK/$engine-$variant-caller.log" 2>&1
    result=$?
    set -e
    printf '%s %s typed caller javac exit=%s\n' "$engine" "$variant" "$result"
    cat "$WORK/$engine-$variant-caller.log"
    if [ "$result" -eq 0 ]; then
      java -Xverify:all -cp "$output" methodthrows.MethodThrowsCaller
    fi
  done
done

java -version 2>&1 | head -3
jadx --version
cargo --version
shasum -a 256 "$FIXTURE"/*.java "$WORK"/original-*/methodthrows/MethodThrowsBoundary.class \
  "$WORK"/jadx-*/sources/methodthrows/MethodThrowsBoundary.java \
  "$WORK"/jarde-*/MethodThrowsBoundary.java
