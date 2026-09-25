#!/bin/sh
set -eu

root=$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)
base=$(CDPATH= cd -- "$root/../.." && pwd)
tmp=$(mktemp -d /tmp/jarde-inner-invalid-controls.XXXXXX)
trap 'rm -rf "$tmp"' EXIT HUP INT TERM

rm -rf "$root/classes/nested" "$root/jadx-full" "$root/jadx-missing"
mkdir -p "$tmp/classes" "$root/classes" "$root/jadx-full" "$root/jadx-missing"
javac --release 8 -g:none -Xlint:-options -d "$tmp/classes" \
    "$base/fixture/SimpleOuter.java" \
    "$base/fixture/UseInner.java" \
    "$base/fixture/InnerRunner.java" \
    "$root/fixture/EffectOrder.java" \
    "$root/fixture/EffectRunner.java" \
    "$root/fixture/MissingRunner.java"
cp -R "$tmp/classes/nested" "$root/classes/"
jar cf "$root/full-target.jar" -C "$tmp/classes" nested
jar cf "$root/missing-target.jar" \
    -C "$tmp/classes" nested/SimpleOuter.class \
    -C "$tmp/classes" nested/UseInner.class \
    -C "$tmp/classes" nested/MissingRunner.class
jar tf "$root/missing-target.jar" > "$root/missing-target-entries.txt"

java -Xverify:all -cp "$root/full-target.jar" nested.InnerRunner > "$root/original-run.txt"
java -Xverify:all -cp "$root/full-target.jar" nested.EffectRunner > "$root/effect-run.txt"
java -Xverify:all -cp "$root/full-target.jar" nested.MissingRunner > "$root/present-target-run.txt"
java -Xverify:all -cp "$root/missing-target.jar" nested.MissingRunner > "$root/missing-target-run.txt"

javap -c -p -classpath "$root/full-target.jar" nested.UseInner > "$root/javap-UseInner.txt"
javap -c -p -classpath "$root/full-target.jar" nested.EffectOrder > "$root/javap-EffectOrder.txt"
javap -v -p -classpath "$root/full-target.jar" 'nested.SimpleOuter$Inner' > "$root/javap-Inner.txt"
jadx --version > "$root/jadx-version.txt"
jadx -d "$root/jadx-full" "$root/full-target.jar" > "$root/jadx-full-run.txt" 2>&1
jadx -d "$root/jadx-missing" "$root/missing-target.jar" > "$root/jadx-missing-run.txt" 2>&1
mkdir -p "$tmp/jadx-full-classes" "$tmp/jadx-missing-classes"
javac --release 8 -g:none -Xlint:-options -d "$tmp/jadx-full-classes" \
    "$root/jadx-full/sources/nested/"*.java > "$root/jadx-full-compile.txt" 2>&1
java -Xverify:all -cp "$tmp/jadx-full-classes" nested.EffectRunner > "$root/jadx-effect-run.txt"
java -Xverify:all -cp "$tmp/jadx-full-classes" nested.InnerRunner > "$root/jadx-original-run.txt"
if javac --release 8 -g:none -Xlint:-options -d "$tmp/jadx-missing-classes" \
    "$root/jadx-missing/sources/nested/"*.java > "$root/jadx-missing-compile.txt" 2>&1; then
    printf 'exit=0\n' >> "$root/jadx-missing-compile.txt"
else
    printf 'exit=nonzero\n' >> "$root/jadx-missing-compile.txt"
fi

(
    cd "$root"
    shasum -a 256 classes/nested/*.class full-target.jar missing-target.jar
) > "$root/SHA256.txt"
