#!/bin/sh
set -eu
export PYTHONDONTWRITEBYTECODE=1

ROOT=$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)
EVIDENCE="$ROOT/openspec/evidence/java-syntax-2026-09-27/dt29-family/p2-generic-void"
TMP=$(mktemp -d "${TMPDIR:-/tmp}/jarde-dt29-p2.XXXXXX")
export CARGO_TARGET_DIR="$TMP/cargo-target"
trap 'rm -rf "$TMP"' EXIT HUP INT TERM

mkdir -p "$TMP/original/classes" "$TMP/original/src" "$TMP/family"
javac --release 8 -g:none -d "$TMP/original/classes" \
    "$EVIDENCE/Bound.java" "$EVIDENCE/Setter.java"
cp "$EVIDENCE/Runner.java" "$TMP/original/src/Runner.java"
javac --release 8 -g:none -cp "$TMP/original/classes" -d "$TMP/original/classes" \
    "$TMP/original/src/Runner.java"
javac --release 8 -g:none -cp "$TMP/original/classes" -d "$TMP/original/classes" \
    "$EVIDENCE/negative/IncompatibleSetter.java" "$EVIDENCE/negative/OverloadedSetter.java"
jar --create --file "$TMP/family/dt29p2.jar" -C "$TMP/original/classes" dt29p2/Bound.class -C "$TMP/original/classes" dt29p2/Setter.class

run_runner() {
    classes=$1
    java -Xverify:all -cp "$classes" dt29p2.Runner
}

printf 'original: '
run_runner "$TMP/original/classes"

jadx -d "$TMP/jadx" "$TMP/family/dt29p2.jar" >/dev/null
mkdir -p "$TMP/jadx/src/dt29p2" "$TMP/jadx/classes"
cp "$TMP/jadx/sources/dt29p2/Bound.java" "$TMP/jadx/src/dt29p2/Bound.java"
cp "$TMP/jadx/sources/dt29p2/Setter.java" "$TMP/jadx/src/dt29p2/Setter.java"
cp "$EVIDENCE/Runner.java" "$TMP/jadx/src/dt29p2/Runner.java"
javac --release 8 -g:none -d "$TMP/jadx/classes" "$TMP/jadx/src/dt29p2"/*.java
printf 'jadx: '
run_runner "$TMP/jadx/classes"

cargo build --locked -q -p jarde-cli
"$CARGO_TARGET_DIR/debug/jarde-cli" class-source --input "$TMP/family/dt29p2.jar" \
    --class dt29p2.Bound --policy plain-jar --format text > "$TMP/jarde-Bound.java" 2> "$TMP/jarde-Bound.stderr"
"$CARGO_TARGET_DIR/debug/jarde-cli" class-source --input "$TMP/family/dt29p2.jar" \
    --class dt29p2.Setter --policy plain-jar --format text > "$TMP/jarde-Setter.java" 2> "$TMP/jarde-Setter.stderr"
mkdir -p "$TMP/jarde/src/dt29p2" "$TMP/jarde/classes"
cp "$TMP/jarde-Bound.java" "$TMP/jarde/src/dt29p2/Bound.java"
cp "$TMP/jarde-Setter.java" "$TMP/jarde/src/dt29p2/Setter.java"
cp "$EVIDENCE/Runner.java" "$TMP/jarde/src/dt29p2/Runner.java"
javac --release 8 -g:none -d "$TMP/jarde/classes" "$TMP/jarde/src/dt29p2"/*.java
printf 'jarde: '
run_runner "$TMP/jarde/classes"
rg -n 'public <T extends dt29p2\.Bound> void set\(T arg1, boolean arg2\)' "$TMP/jarde-Setter.java"

for class_name in Bound Setter; do
    javap -v -p -classpath "$TMP/original/classes" "dt29p2.$class_name" \
        | rg 'descriptor:|Signature:' > "$TMP/original-$class_name-signatures.txt"
    javap -v -p -classpath "$TMP/jarde/classes" "dt29p2.$class_name" \
        | rg 'descriptor:|Signature:' > "$TMP/jarde-$class_name-signatures.txt"
done
printf 'Setter descriptor and Signature lines (original / Jarde):\n'
cat "$TMP/original-Setter-signatures.txt" "$TMP/jarde-Setter-signatures.txt"

python3 "$EVIDENCE/negative/make_classes.py" "$ROOT" "$TMP/original/classes" "$TMP/negative"
for case in wrong-bound unbound-variable IncompatibleSetter OverloadedSetter; do
    case_class=$case
    if [ "$case" = wrong-bound ] || [ "$case" = unbound-variable ]; then
        case_class=Setter
    fi
    "$CARGO_TARGET_DIR/debug/jarde-cli" class-source --input "$TMP/negative/$case.class" \
        --class "dt29p2.$case_class" --policy single-class --format text \
        > "$TMP/$case.java" 2> "$TMP/$case.stderr"
    rg -q 'generic Signature projection refused for' "$TMP/$case.java"
    rg -q 'public void set\(dt29p2\.Bound arg1, boolean arg2\)' "$TMP/$case.java"
    printf 'negative %s: descriptor declaration retained; generic projection refused\n' "$case"
done

if "$CARGO_TARGET_DIR/debug/jarde-cli" class-source --input "$TMP/family/dt29p2.jar" \
    --class dt29p2.Setter --policy plain-jar --budget method_bodies=1 --format json \
    > "$TMP/budget.json" 2> "$TMP/budget.stderr"; then
    budget_exit=0
else
    budget_exit=$?
fi
budget_status=$(jq -r '.execution.status' "$TMP/budget.json")
if [ "$budget_status" != partial ] || [ "$budget_exit" -ne 4 ]; then
    printf 'expected method body budget stop, got status=%s exit=%s\n' "$budget_status" "$budget_exit" >&2
    exit 1
fi
printf 'budget stop: %s\n' "$budget_status"
