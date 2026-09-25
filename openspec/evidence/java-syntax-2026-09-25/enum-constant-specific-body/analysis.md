# Java 8 enum constant-specific class-body evidence

## Fixtures and replay

The frozen source has three small cases: `Op` has two body constants, `Mixed` has one body constant followed by one ordinary constant, and `Plain` has two ordinary constants. Their runners check `values()` ordering, `name()`, `ordinal()`, per-constant method results, runtime class identity (`getClass() == EnumType.class`), runtime class name, and `getDeclaringClass()`. Every fixture is self-contained in this directory.

From this directory, run:

```sh
python3 replay.py --jarde-cli /tmp/jarde-generic-accepted-cli
shasum -a 256 -c manifest.sha256
```

The script also works after copying this directory elsewhere; it derives paths from `__file__`, uses a system temporary directory for all classes/JARs, and only refreshes the bounded `outputs/` evidence tree. The frozen Jarde executable is `jarde-cli 0.1.0`, SHA-256 `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`. JADX is 1.5.6, SHA-256 `64a6ee6bcf7490ea682508db2a73d6cda8b671a5211af5ee3ff098441af038a7`. The compiler/runtime is OpenJDK 23.0.1 with `javac --release 8`; the emitted classfiles have major version 52.

For each of `-g` and `-g:none`, `replay.py` compiles the original sources, saves complete original and JADX-recompiled `javap -v -c -p` output and per-class SHA-256 JSON, decompiles a temporary JAR with JADX, compiles every JADX source with `--release 8`, and runs original/JADX runners under `java -Xverify:all`. It asks the frozen Jarde CLI for complete class-source text for each enum and runner, retains those texts and compiler output, and attempts the same Java 8 compile. No class, JAR, or JADX cache is retained. `summary.json` records tool versions, executable hashes, class/source hashes, and result statuses. `manifest.sha256` covers every file except itself.

## Behavior and three-way comparison

For both debug variants, original and JADX-recompiled classes verify and run with byte-for-byte identical output:

```text
Op:
ADD:0=10/runtimeIsEnum=false/class=demo.Op$1/declaring=Op
MULTIPLY:1=21/runtimeIsEnum=false/class=demo.Op$2/declaring=Op

Mixed:
SPECIAL:0:7:runtimeIsEnum=false:class=demo.Mixed$1:declaring=Mixed
PLAIN:1:0:runtimeIsEnum=true:class=demo.Mixed:declaring=Mixed

Plain:
READY=ready:runtimeIsEnum=true
WAITING=waiting:runtimeIsEnum=true
```

Thus the mixed runner proves that only `SPECIAL` has a distinct runtime class and that both constants still report `Mixed` as their declaring enum. The runner comparisons are generated independently from the fixture source; this is not inferred from JADX's rendered Java text. The repository-local JADX `EnumVisitor` tests are source-backed and assert only selected render fragments, so this independent javac and verifier run is the behavior evidence for these fixtures.

JADX 1.5.6 renders both `Op` constants with bodies, renders only `Mixed.SPECIAL` with a body, and leaves both `Plain` constants bodyless. All three complete JADX source sets compile and the runner outputs above match the original classes in both debug variants. Full source outputs are kept under `outputs/{g,g-none}/jadx-source/`.

The frozen Jarde class-source text still reports ordinary `static final` fields for enum constants and physical `$VALUES`, `$values()`, and `<clinit>` members; it also retains the generated anonymous subclass constructors and does not emit any constant-specific body. The enum plus generated Jarde runner source fails `javac --release 8` for all three fixtures in both variants. The first diagnostic is “enum constant expected” at the ordinary field declaration; `Op` and `Mixed` also show that a physical anonymous-owner constructor parameter such as `demo.Op$1` / `demo.Mixed$1` is not a source-level enum constructor parameter. There is no Jarde runtime result because compilation rejects this presentation. The runner source returned by Jarde is retained too, so this compile attempt covers generated enum and runner classes rather than mixing a Jarde enum with the original runner.

## Classfile facts and base-change boundary

The retained `outputs/{g,g-none}/original-javap-v-c-p.txt` files are the complete classfile listing for each variant. The following `<clinit>` BCIs are stable in both variants:

| Fixture | Body/non-body construction | Constant field write | Values helper |
| --- | --- | --- | --- |
| `Op` | `0 new Op$1`, `7 invokespecial Op$1.<init>(String,int)`; `13 new Op$2`, `20 invokespecial Op$2.<init>(String,int)` | `10 putstatic ADD`, `23 putstatic MULTIPLY` | `26 invokestatic $values`, `29 putstatic $VALUES`, `32 return` |
| `Mixed` | `0 new Mixed$1`, `7 invokespecial Mixed$1.<init>(String,int)`; `13 new Mixed`, `20 invokespecial Mixed.<init>(String,int)` | `10 putstatic SPECIAL`, `23 putstatic PLAIN` | `26 invokestatic $values`, `29 putstatic $VALUES`, `32 return` |
| `Plain` | `0 new Plain`, `7 invokespecial Plain.<init>(String,int)`; `13 new Plain`, `20 invokespecial Plain.<init>(String,int)` | `10 putstatic READY`, `23 putstatic WAITING` | `26 invokestatic $values`, `29 putstatic $VALUES`, `32 return` |

The Java source declares no enum constructor and has zero source constructor parameters in all fixtures. Ordinary enum constant construction still uses `(Ljava/lang/String;I)V`: javac injects name and ordinal. For `Op` and `Mixed`, the main enum additionally has a synthetic constructor whose descriptor carries the specific anonymous owner as a third parameter, e.g. `(Ljava/lang/String;ILdemo/Op$1;)V`. Each anonymous class's source-shaped constructor is `(Ljava/lang/String;I)V` and invokes that synthetic owner-aware enum constructor with the owner marker. Its classfile has no instance fields, extends the enum directly, and carries `EnclosingMethod` / `InnerClasses` metadata. `Plain` has no anonymous class and no owner-aware overload. These are materially different source-level constructor sets despite similar enum initialization prefixes.

The normal Java 8 source form of a zero-argument enum constructor is `EnumType()`, while its physical constructor descriptor is `(String,int)`. A body enum introduces a distinct owner class and owner-marked synthetic constructor form. The base `recover-proved-enum-constants` tasks 2.2 and 2.3 project only the proved two-constant case with one source-level integer argument; 2.3 adds the `Measure` user-static-suffix slice. Therefore zero-source-argument enums and anonymous constant owners remain outside that base proof gate. This fixture does not claim the base implementation is complete for those shapes.

The script's explicitly hashed frozen CLI `ca04265a...` emits constants as ordinary fields for all three inputs and rejects recompilation; its output includes physical anonymous-owner constructors for `Op` and `Mixed`. Root also rebuilt the three sources under both debug modes and queried the post-2.3 CLI `2a3664ef...` against a JAR containing all sibling classes. All six enum presentations still retain ordinary constant fields and decline the base projection, confirming that the new `Measure` suffix did not admit these zero-argument or anonymous-owner shapes.

### Frozen class hashes

Class SHA-256 values for the `-g` originals are in `outputs/g/original-class-sha256.json`; the `-g:none` values are in the corresponding file under `outputs/g-none/`. As a short identity anchor, the `-g` enum classes are:

| Class | SHA-256 |
| --- | --- |
| `demo/Op.class` | `29bfd70fc4bd4cbe6bdec0ea433fe4c750bd42013bf2e4b65666a339edce10b4` |
| `demo/Mixed.class` | `e0012e3bc3b976652acde8658d4d051107e2b949354e085c27c56bde75580270` |
| `demo/Plain.class` | `a380bdfed2860f636dc32712f46acc967689d02755c3c30a19bb79935bda8f26` |

## JADX implementation notes

The local JADX 1.5.6 checkout is `/Users/lordcasser/workspace/testzone/jadx`. `jadx-core/src/main/java/jadx/core/dex/visitors/EnumVisitor.java` scans the enum initializer, resolves constructor owners in `processConstructorInsn` (lines 252–263), and routes a non-main owner into `processEnumCls` (line 258). That routine hides the child's constructor and arranges the class for inlining (around line 741). The path recognizes anonymous-owner classes and skips the first two enum-injected arguments (`createEnumFieldByConstructor` around line 449; `markArgsForSkip` around line 653); the complete output for these cases is retained above. `ProcessAnonymous.canBeAnonymous` (around line 203) uses synthetic/name/use-shape heuristics, then `checkUsage` applies its own use analysis. Those mechanisms explain JADX's successful rendering here, but they are not a substitute for Jarde's proposed same-run complete class/BCI proof.

## Replay acceptance boundary

`summary.json` and the retained outputs make the evidence reproducible. Task 1.3 is complete from the recorded zero-source-argument, descriptor, owner, metadata, and BCI facts. Root independently replayed the copied evidence directory: all 79 manifest entries passed and `summary.json` matched the frozen version exactly, completing task 1.1. The base change has since passed full Root acceptance, and the post-2.3 CLI still refuses all six zero-argument/anonymous-owner presentations, completing the task 1.2 prerequisite without claiming `Op` support.
