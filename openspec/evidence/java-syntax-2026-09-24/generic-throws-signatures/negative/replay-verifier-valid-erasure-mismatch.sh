#!/bin/sh
set -eu
ROOT=$(git rev-parse --show-toplevel)
FIXTURE="$ROOT/openspec/evidence/java-syntax-2026-09-24/generic-throws-signatures/fixture"
WORK=$(mktemp -d "${TMPDIR:-/tmp}/jarde-generic-throws-negative.XXXXXX")
export CARGO_TARGET_DIR="$WORK/cargo-target"
trap 'python3 -c "import shutil,sys; shutil.rmtree(sys.argv[1], ignore_errors=True)" "$WORK"' EXIT
mkdir -p "$WORK/original" "$WORK/patched" "$WORK/jarde"
javac --release 8 -Xlint:-options -g:none -d "$WORK/original" \
  "$FIXTURE/GenericThrowsBoundary.java" "$FIXTURE/GenericThrowsCaller.java"
cp -R "$WORK/original/." "$WORK/patched/"
python3 - "$WORK/patched/genericthrows/GenericThrowsBoundary.class" <<'PY'
import pathlib, sys
path = pathlib.Path(sys.argv[1])
data = path.read_bytes()
old = b'E:Ljava/lang/Exception;'
new = b'E:Ljava/lang/Throwable;'
assert len(old) == len(new)
assert data.count(old) == 1, data.count(old)
path.write_bytes(data.replace(old, new, 1))
PY
# The class Signature now erases throws E to Throwable, while the method descriptor and
# Exceptions attribute remain untouched. Signature metadata does not participate in verification.
java -Xverify:all -cp "$WORK/patched" genericthrows.GenericThrowsCaller
javap -v "$WORK/patched/genericthrows/GenericThrowsBoundary.class" > "$WORK/patched.javap"
 javap -v "$WORK/original/genericthrows/GenericThrowsBoundary.class" > "$WORK/original.javap"
awk '/Exceptions:/{print; getline; print}' "$WORK/original.javap" > "$WORK/original.exceptions"
awk '/Exceptions:/{print; getline; print}' "$WORK/patched.javap" > "$WORK/patched.exceptions"
cmp "$WORK/original.exceptions" "$WORK/patched.exceptions"
rg -n 'Signature:|Exceptions:|invoke\(' "$WORK/patched.javap"
cargo run -q -p jarde-cli -- class-source \
  --input "$WORK/patched/genericthrows/GenericThrowsBoundary.class" \
  --class genericthrows.GenericThrowsBoundary --policy single-class \
  > "$WORK/jarde/GenericThrowsBoundary.java" 2> "$WORK/jarde/run.log"
rg -n 'invoke\(|generic Signature projection refused|jvm_signature_erasure_mismatch' "$WORK/jarde/GenericThrowsBoundary.java"
rg -n 'invoke\(\) throws java.lang.Exception;' "$WORK/jarde/GenericThrowsBoundary.java"
