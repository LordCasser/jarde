#!/bin/sh
set -eu
cd "$(dirname "$0")"
rm -rf build
mkdir -p build/baseline build/patched
javac --release 8 -g -d build/baseline ScalarCases.java ReflectCheck.java
cp build/baseline/ScalarCases.class build/ScalarCases.baseline.class
python3 patch_scalar_class.py build/baseline/ScalarCases.class build/patched/ScalarCases.class
javap -v -p -classpath build/baseline ScalarCases > build/baseline.javap.txt
javap -v -p -classpath build/patched:build/baseline ScalarCases > build/patched.javap.txt
(
    echo '=== baseline reflection under full verification ==='
    java -Xverify:all -cp build/baseline ReflectCheck
    echo '=== patched reflection under full verification ==='
    java -Xverify:all -cp build/patched:build/baseline ReflectCheck
) > build/reflection.txt 2>&1
shasum -a 256 build/baseline/ScalarCases.class build/patched/ScalarCases.class \
    build/baseline.javap.txt build/patched.javap.txt build/reflection.txt \
    > build/sha256.txt
cat build/reflection.txt
cat build/patched/ScalarCases.patch-report.txt
cat build/sha256.txt
