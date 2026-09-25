# Task 1.2 fixture freeze and replay

Task 1.2 freezes the verifier-valid Boolean initializer and a proved effectful positive sample. The independent pre-fix observations are cited from the frozen audit logs; no post-fix Jarde output is used as a pre-fix claim.

## BoolInit

The frozen input is `tests/fixtures/p3-boolean-array-initializers/v8/BoolInit.class` (189 bytes, major 52, SHA-256 `e0e8cf5cf99fdb3e402db16e6d0b86db1b8b8672df2e684523c72a67df724a3f`). It is produced by compiling the permanent source with `javac --release 8 -g:none`, then applying `patch_class.py`; the source int[] class hash is `5304fa21e9594a0ab6e04e0f79ee0da3bbcdcbc85e3019183d7cdd5e7b22da7d`. The patch changes one `()[I` descriptor to `()[Z`, one `newarray` atype from 10 to 4, and five `iastore` opcodes to `bastore` at code offsets/BCIs 6, 10, 14, 18, and 22. Length and all other code bytes remain unchanged.

The original patched class passed `java -Xverify:all` and printed `[false, true, false, true, true]`; output SHA-256 is `5eb544d91affd86d74eb4a839f6c966e822e83055515f9a01b88687bdfcdc6a6`. The permanent expected file has the same hash. The pre-fix JADX source and Java 8 `javac` rejection, plus the Jarde refusal at overall consumer BCI 23, are preserved verbatim in `openspec/evidence/java-syntax-2026-09-25/boolean-array-initializer/reproduced/logs/jadx.json`, `javac-jadx.json`, and `jarde.json`. The Jarde log belongs to the frozen audit CLI and is the only pre-fix Jarde claim here.

The ignored JDK integration test compiles the complete recovered class and the runner with Java 8, executes both the patched fixture and generated class under `-Xverify:all`, and compares output exactly. It checks that all five store BCIs appear in the source map and that the three raw conversions (2, 3, and -1 at stores 14, 18, and 22) have distinct element-sized `% 2 != 0` spans. The 0/1 constants become direct boolean literals, so their store BCIs can be represented by the encompassing `NewArray` span rather than separate conversion spans. Because `source_map.of_bci(store)` can return that aggregate span, the test does not treat its mere presence as proof of element pairing; the store-to-element contract test checks the paired conversion origins. The complete report has no bytecode or unrecovered markers. The test passed.

## BoolInitEffectful

A second, minimal source-only sample uses a side-effecting helper in each initializer element. Its patched class `v8/BoolInitEffectful.class` is major 52 and SHA-256 `ea2175192c0967da85ef5d78253bd458fb8a5543bcaf21f622b20a2caf59a8d0`. The original int[] class is SHA-256 `b7e8c4936adab5af0e18f7df86480e1ef72ff79e22e935d8a6e66a99177214bd`. Patching descriptor `(I)[I` to `(I)[Z`, atype 10 to 4, and three stores to `bastore` changes no class length. The corresponding real store BCIs are 10, 18, and 26.

Jarde recovers every method and the complete source compiles with `javac --release 8 -g:none`; neither presentation contains bytecode or unrecovered markers. Original and generated runs both use `java -Xverify:all`. Their output SHA-256 is `9af9e18526f357ba9f2288520cd69520b2a7085de9fb7a5f2264d7d4f9f129e7`, and the lines are:

```text
-1:array=[false, true, true]:trace=abc
0:exception=java.lang.ArithmeticException:message=/ by zero:trace=a
1:exception=java.lang.ArithmeticException:message=/ by zero:trace=ab
2:exception=java.lang.ArithmeticException:message=/ by zero:trace=abc
3:array=[true, true, false]:trace=abc
```

This confirms left-to-right single evaluation and the partial trace at each thrown element for this closed chain. The integration test checks that all three store BCIs have distinct element-specific conversion spans after filtering out the aggregate `NewArray` span, then compares the whole output. It passed.

## Scope and rerun

Permanent files are limited to the two source/runner pairs, expected outputs, patch script, README and the two patched subject classes under `tests/fixtures/p3-boolean-array-initializers/`. Runner classfiles, generated classes and compilation output were kept under `/tmp/jarde-bool-init-fixture-target/`; no runner `.class` is committed.

Replay the tests with:

```sh
CARGO_TARGET_DIR=/tmp/jarde-bool-init-fixture-target cargo test --locked --test p3_boolean_array_initializers -- --ignored
```

The test is ignored in ordinary runs because it requires a JDK (`javac --release 8` and `java -Xverify:all`) on PATH. This is a JDK-runtime acceptance check, not an always-available Rust-only unit test.
