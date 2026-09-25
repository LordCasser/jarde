# P3 verifier-valid boolean array initializer

`v8/BoolInit.class` is the byte-for-byte controlled patch from the independent audit. Rebuild it in a temporary directory; the fixture never stores runner classfiles:

```sh
OUT=/tmp/jarde-bool-init-fixture-rebuild
mkdir -p "$OUT/original" "$OUT/patched" "$OUT/runner"
javac --release 8 -g:none -d "$OUT/original" BoolInit.java
python3 patch_class.py "$OUT/original/BoolInit.class" "$OUT/patched/BoolInit.class" "$OUT/patch.json"
javac --release 8 -g:none -d "$OUT/runner" BoolInitRunner.java
java -Xverify:all -cp "$OUT/patched:$OUT/runner" BoolInitRunner
shasum -a 256 "$OUT/original/BoolInit.class" "$OUT/patched/BoolInit.class"
```

The original int-array class is 189 bytes, major 52, SHA-256 `5304fa21e9594a0ab6e04e0f79ee0da3bbcdcbc85e3019183d7cdd5e7b22da7d`. The patched verifier-valid class is 189 bytes, major 52, SHA-256 `e0e8cf5cf99fdb3e402db16e6d0b86db1b8b8672df2e684523c72a67df724a3f`. The only changes are descriptor `()[I` → `()[Z`, `newarray` atype `10` → `4`, and five `iastore` → `bastore` opcodes at Code offsets 6, 10, 14, 18, and 22. The original `Code` bytes and audit hashes are documented in `openspec/evidence/java-syntax-2026-09-25/boolean-array-initializer/analysis.md` and `reproduced/patch.json`.

The frozen expected runner output is `[false, true, false, true, true]`. The ignored Rust integration test compiles the complete Jarde class presentation and runner with `javac --release 8`, then compares execution with this verifier-checked class. It checks all five stores appear in the source map, and verifies distinct element-sized conversion spans for the raw 2, 3, and -1 stores. The 0/1 constants render directly as boolean literals; their store BCIs can be represented by the enclosing `NewArray` source span. An aggregate `NewArray` origin alone is not treated as evidence of element pairing.

## Frozen pre-fix boundary

The pre-fix JADX source, `javac` failure and Jarde consumer-BCI-23 refusal are frozen in the independent audit at `openspec/evidence/java-syntax-2026-09-25/boolean-array-initializer/reproduced/logs/{jadx,javac-jadx,jarde}.json`. Those logs are the pre-fix record; later runs against a changing production checkout must not be described as pre-fix evidence.

## Effectful positive sample

`BoolInitEffectful.java` is a minimal source-only analogue of the existing `effectfulInts` shape. Its patched verifier-valid class is frozen at `v8/BoolInitEffectful.class` (major 52, SHA-256 `ea2175192c0967da85ef5d78253bd458fb8a5543bcaf21f622b20a2caf59a8d0`). Rebuild in the same temporary tree as above:

```sh
javac --release 8 -g:none -d "$OUT/original" BoolInitEffectful.java
python3 patch_class.py --stores 3 --descriptor '(I)[I' \
  "$OUT/original/BoolInitEffectful.class" "$OUT/patched/BoolInitEffectful.class" "$OUT/effectful-patch.json"
javac --release 8 -g:none -cp "$OUT/original" -d "$OUT/runner" BoolInitEffectfulRunner.java
java -Xverify:all -cp "$OUT/patched:$OUT/runner" BoolInitEffectfulRunner
```

The initializer is folded by the current proved path, and the complete generated Jarde class compiles under Java 8 with no bytecode or unrecovered markers. Original patched-class and generated-class outputs match line for line under `-Xverify:all`; modes 0, 1, and 2 divide by zero after traces `a`, `ab`, and `abc`, while modes -1 and 3 produce `[false, true, true]` and `[true, true, false]`. The regression test checks distinct element-sized conversion source spans for all three store BCIs (10, 18, 26), filtering out the enclosing `NewArray` source span, as well as exception and trace preservation.
