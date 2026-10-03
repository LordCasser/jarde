#!/bin/sh
# Reproduction: the reorder in recover-synthetic-ctor-super-order (daa4fb31) is
# behavior-changing when the superclass constructor virtually dispatches.
set -eu
F=tests/fixtures/proved-java-structure/anonymous-super-dispatch
J=target/debug/jarde-cli   # build first: cargo build -p jarde-cli --locked
T=$(mktemp -d); trap 'rm -rf "$T"' EXIT
echo "=== ORIGINAL (expected) ==="
java -Xverify:all -cp "$F" AnonymousSuperDispatch
$J class-source --input "$F/AnonymousSuperDispatch\$1.class" --policy single-class \
  --class 'AnonymousSuperDispatch$1' 2>/dev/null | sed 's|^\s*//.*$||' > "$T/rendered.java"
echo "=== jarde-rendered ctor ==="
sed -n '/AnonymousSuperDispatch\$1(java/,/^    }/p' "$T/rendered.java"
javac --release 8 -g:none -Xlint:-options -cp "$F" -d "$T/out" "$T/rendered.java"
echo "=== recompiled bytecode order (super now FIRST) ==="
javap -c -p "$T/out/AnonymousSuperDispatch\$1.class" | sed -n '/AnonymousSuperDispatch\$1(java.lang.String)/,/^$/p'
echo "=== RECOMPILED BEHAVIOR (regression: null / false) ==="
java -Xverify:all -cp "$T/out:$F" AnonymousSuperDispatch
