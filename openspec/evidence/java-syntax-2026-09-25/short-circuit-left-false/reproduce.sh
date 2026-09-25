#!/bin/sh
set -eu
cd "$(dirname "$0")/../../../.."

evidence="openspec/evidence/java-syntax-2026-09-25/short-circuit-left-false"
fixture="tests/fixtures/proved-java-structure/short-circuit-left-false"
work="$(mktemp -d /tmp/jarde-short-circuit-left-false.XXXXXX)"
trap 'rm -rf "$work"' EXIT HUP INT TERM

export CARGO_TARGET_DIR="$work/cargo-target"
cargo build -p jarde-cli --locked > "$evidence/jarde-build.log" 2>&1
"$CARGO_TARGET_DIR/debug/jarde-cli" class-source --input "$fixture/ShortCircuitFalse.class" --class ShortCircuitFalse --policy single-class --format json --evidence all > "$evidence/jarde-report.json" 2> "$evidence/jarde-stderr.txt"
python3 -c 'import json,sys; print(json.load(open(sys.argv[1]))["text"], end="")' "$evidence/jarde-report.json" > "$evidence/jarde-ShortCircuitFalse.java"

mkdir -p "$work/input" "$work/jadx-classes" "$work/jarde-classes"
mkdir -p "$work/regenerated"
javac --release 8 -g -d "$work/regenerated" "$fixture/ShortCircuitFalse.java" > "$evidence/freeze-javac.log" 2>&1
cmp "$fixture/ShortCircuitFalse.class" "$work/regenerated/ShortCircuitFalse.class"
cp "$fixture"/*.class "$work/input/"
jar --create --file "$work/frozen.jar" -C "$work/input" .

java -Xverify:all -cp "$work/frozen.jar" ShortCircuitFalse > "$evidence/original-run.txt"
javap -classpath "$work/frozen.jar" -p -c -s ShortCircuitFalse > "$evidence/javap.txt"
shasum -a 256 "$fixture"/*.class > "$evidence/class-sha256.txt"

jadx --version > "$evidence/jadx-version.txt"
jadx -d "$work/jadx" "$work/frozen.jar" > "$evidence/jadx.log" 2>&1
cp "$work/jadx/sources/defpackage/ShortCircuitFalse.java" "$evidence/jadx-ShortCircuitFalse.java"
javac --release 8 -g -cp "$work/frozen.jar" -d "$work/jadx-classes" "$evidence/jadx-ShortCircuitFalse.java" > "$evidence/jadx-javac.log" 2>&1
java -Xverify:all -cp "$work/jadx-classes:$work/frozen.jar" defpackage.ShortCircuitFalse > "$evidence/jadx-run.txt"

javac --release 8 -g -cp "$work/frozen.jar" -d "$work/jarde-classes" "$evidence/jarde-ShortCircuitFalse.java" > "$evidence/jarde-javac.log" 2>&1
java -Xverify:all -cp "$work/jarde-classes:$work/frozen.jar" ShortCircuitFalse > "$evidence/jarde-run.txt"

{
    java -version 2>&1
    javac -version 2>&1
} > "$evidence/java-versions.txt"
