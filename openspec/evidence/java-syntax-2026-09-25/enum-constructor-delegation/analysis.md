# Enum constructor delegation: independent three-way audit

This is a standalone Java 8 audit inspired by JADX's `TestEnums6`. It does not modify production code, fixture registration, or an OpenSpec change. Run `./replay.sh` from any directory to regenerate the retained text evidence; class files, jars, and compiler outputs are built under a `mktemp` directory and removed on exit.

## Source and behavior

The source in `source/` keeps the important source-level shape:

```java
ZERO,
ONE(1);

DelegatingEnum() {
    this(0);
}
```

`ConstructorEffects` is a separate ordinary class. Each terminal `int` constructor calls it once, so the counter and ordered values are observable without accessing enum static state during enum initialization. `EnumRunner` checks `values()` length/order/identity, `name()`, `ordinal()`, the stored instance values, and exactly two constructor effects in `0,1` order. It also reads `DelegatingEnum.class.getDeclaredConstructors()` and checks the parameter-count set `{2,3}`. Those counts include the JVM-injected `String name, int ordinal` prefix: they distinguish the source no-argument overload plus source `int` overload from a projection that rewrites `ZERO` as `ZERO(0)` and drops the no-argument overload (which would leave only the three-parameter physical constructor).

The original and JADX-recompiled runners both print:

```text
values=ZERO:0,ONE:1
effects=2:0,1
declared-constructors=2,3
```

These outputs were captured independently for both `-g` and `-g:none`, after `javac --release 8`, then executed with `java -Xverify:all`. See `original-run-*.txt` and `jadx-run-*.txt`.

## JADX test coverage

The source test is from local JADX checkout `2fb1b16386941660fda07e9017285aec40fcb37f`, where `jadx --version` reports `1.5.6`: `jadx-core/src/test/java/jadx/tests/integration/enums/TestEnums6.java` (SHA-256 `b168f0a27e354fec21d206bb8c431b839f6e7ee8a9b0b4808c5b2dc1ca2a9a48`). Its `@Test` asserts only that generated code contains one each of `ZERO,`, `Numbers() {`, and `ONE(1);`. The file also has a `check()` method with value assertions, but `test()` does not call it. Therefore the integration test checks those source fragments; it does not establish that the complete emitted class recompiles or preserves runtime behavior. This audit compiled all JADX-emitted sources as Java 8 and ran its class under `-Xverify:all`.

## Class-file evidence

The compiler was `javac 23.0.1` targeting release 8. `classfile-sha256.txt` records every input class hash from the `-g` and `-g:none` builds. Of particular interest, `DelegatingEnum.class` hashes to:

| Build | SHA-256 |
| --- | --- |
| `-g` | `4c8ae5fea5ad7d4403a3d7c61b8927c4b8c4c215a9c17a22449be1a3ec543804` |
| `-g:none` | `f0742ab942d501f6ed5e9fdcfa185cf7510e13d84b92fa12455d509f10b55c07` |

`javap-g.txt` and `javap-g-none.txt` are full `javap -v -c -p` captures. The `-g` constructor bytecode shows the source no-argument constructor as physical `(String,int)`, delegating with `this(name, ordinal, 0)` to `(String,int,int)`. The latter forwards name/ordinal to `java.lang.Enum.<init>`, records the one side effect, and stores the source integer field. The classfile `Signature` attributes report source signatures `()V` and `(I)V`, even though the physical descriptors contain enum's injected arguments.

## Three outputs and recompilation

`decompiled/jadx-DelegatingEnum-*.java` and `decompiled/jarde-DelegatingEnum-*.java` retain each tool's full enum-class output. JADX 1.5.6 emits `ZERO, ONE(1)`, keeps the no-argument constructor's `this(0)` delegation, and compiles with the helper and runner. Its behavior matches the original for both debug modes.

The frozen Jarde executable is `/tmp/jarde-generic-accepted-cli`, SHA-256 `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`. Jarde's `class-source` result retains `ACC_ENUM` on the class header but presents `ZERO` and `ONE` as ordinary static fields, emits `$VALUES` and `<clinit>` as source members, and exposes physical constructor arguments. It also marks both constructor signatures as an erasure mismatch because source signatures omit the JVM-injected enum parameters. Java 8 recompilation exits `1` for both debug variants at the first field (`此处需要枚举常量` / “enum constant expected”); no Jarde class was produced, so its `java -Xverify:all` run is correctly recorded as not run. This is a whole-enum projection failure before the compiler can assess constructor delegation in isolation.

## Reproduction details and scope

`tool-versions.txt` records Java, javac, JADX, and the frozen CLI hash. `replay.sh` checks that CLI hash before use, recompiles from the checked-in source, compares runner output, and deletes its temporary build tree. `SHA256SUMS.txt` hashes the checked-in Java sources and complete emitted enum classes. No Cargo command was run.

The first compilation failure is the base enum constant gap tracked by active `openspec/changes/recover-proved-enum-constants/`. That change deliberately proves one physical constructor whose only source effect is storing one `int`; it does not cover this input's **two constructors, `this(0)` delegation, and observable helper call**. Once the base declaration is recovered, preserving the source no-argument overload, delegated value, side-effect order, and reflected physical constructor set needs its own bounded proof. Keep this as a separate follow-up OpenSpec rather than broadening the base change while its 2.1 proof is in progress. Root independently reran `replay.sh` from `/tmp` on 2026-09-25: both debug modes passed the script's Java 8 compile, verifier, output and checksum gates; Jarde javac exited 1 in both modes, as recorded.
