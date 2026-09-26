# Anonymous superclass arguments and capture

This Java 8 fixture distinguishes the arguments passed to an anonymous class's direct superclass constructor from the trailing argument used only to initialize a compiler-generated capture field. The argument-producing methods and `Base` constructor append to an event log, and the anonymous body reads the captured value and invokes `super.render()`.

The frozen `.class` files were produced with:

```sh
javac --release 8 -g:none -d . Base.java AnonymousSuperArgs.java
```

Replay the original, JADX and Jarde comparisons with:

```sh
python3 openspec/evidence/java-syntax-2026-09-27/anonymous-super-args/replay.py
```

The script verifies the frozen source-oracle output under `java -Xverify:all`, records SHA-256 and `javap`, builds Jarde using a temporary `CARGO_TARGET_DIR`, invokes the local JADX source checkout, and compiles/runs complete source sets. The comparison and generated sources are in `openspec/evidence/java-syntax-2026-09-27/anonymous-super-args/`.
