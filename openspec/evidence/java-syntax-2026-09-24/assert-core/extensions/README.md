# Assertion shape extensions

This directory adds the requested assertion-syntax positive shapes and verifier-valid refusal controls without changing the frozen `assert-core/` baseline.

## Reproduce

With JDK 23.0.1 and JADX 1.5.6 available, run:

```sh
python3 openspec/evidence/java-syntax-2026-09-24/assert-core/extensions/replay.py \
  --out "$(mktemp -d /tmp/assert-core-extensions.XXXXXX)"
```

The replay builds all inputs from the adjacent `.java` files with `javac --release 8`, captures full `javap -p -c -v`, and runs every class under `java -Xverify:all` with assertions enabled and disabled. It recompiles and executes JADX 1.5.6 source for both the positive and refusal shapes. The script only accepts a new or empty output directory; it never removes existing files. `replay-output/` contains the recorded successful run, including input class files, patched files, compiler/JVM output, full `javap` and JADX source. Its `summary.json` records all source and class SHA-256 values and exact output lines.

The workspace currently has no `target/debug/jarde-cli`; the script records that fact and does not build Cargo. Root can later run the same CLI against these class files after the unified CLI is available.

## Positive control

`AssertVariants.java` includes all three requested source properties: one assertion without a message, two assertions in the same method, and a static initializer with an observable write after javac's generated assertion-status initialization. The Java 8 class SHA-256 is `cf37e61be49733e3273ee46567129eb72eb52ba2051a879f212b2eb569fe419b`.

The `<clinit>` writes `$assertionsDisabled:Z` at BCI 13, then executes the independent `effects++` at BCI 16–21. `noMessage(Z)I` uses the no-argument constructor at BCI 17 (`AssertionError.<init>:()V`). `multiple(ZZ)I` has a separate no-message assert (constructor BCI 17) and a message assert: `detail()` at BCI 38 and `AssertionError.<init>:(Object)V` at BCI 41. Original and JADX source both compile for Java 8 and run identically:

| JVM mode | Output |
| --- | --- |
| `-ea` | `init=1;plain-true=12;plain-false=AssertionError:null;multiple-true=12222;multiple-second-false=AssertionError:message;effects=12222223` |
| `-da` | `init=1;plain-true=1;plain-false=none;multiple-true=1;multiple-second-false=none;effects=1` |

Source hashes: `AssertVariants.java` `1c7542b2d63c0568fb23d906f6267e7f8181faa520478a289c14bb1e0452d26b`; runner `192845a6f4dfc2030ea35ca2938aacf1ffc5bd0070157be33b7651e1e5444c28`. JADX's package-adjusted class source SHA-256 is `b0fc9bc93391c5d672c5e3632f8bfe0d2b1178631e0b2472d7dac4ea933d30c9`.

## JVM-verifiable refusal controls

These are ordinary javac-generated classes with an explicit `$assertionsDisabled` guard. The replay changes only that field's two-byte access flags from `0x0018` (`static final`) to `0x1018` (`static final synthetic`), matching the visibility and flags of the frozen `AssertCore` compiler field; it does not touch code, stack maps, or the constant pool. Both patched classes pass enabled and disabled `-Xverify:all` execution. This makes them valid classfile controls without pretending that arbitrary byte edits preserve verification.

`AssertExtraFieldAccess.class` SHA-256 is `a7ea4f420050e3e803ba91336ad5d670f6c85d4747f9d506843976ef9da6cdbc`. In addition to the assertion-shaped guard, `statusProbe()Z` reads the synthetic status field at BCI 0 outside any assertion, and `check(Z)V` increments observable `extraWrites` at BCI 7–12 before the failure test; the `AssertionError()` call is at BCI 23. With `-ea`, the runner observes one extra write and `statusProbe=false`; with `-da`, the probe returns true. Dropping or turning every field reference into source-level `assert` would lose observable field/statement facts.

`AssertDifferentConstructor.class` SHA-256 is `d94bded16b62e9fa8af1143da531aed3b897bef27dcb9928d1cf75bda16195c5`. Its `check(Z)V` constructs `AssertionError` with descriptor `(I)V` at BCI 16, unlike the no-message `()V` or message `(Object)V` forms javac emits for Java `assert`. It verifies under both modes; the enabled runner prints message `42`, and disabled execution does not throw.

The frozen original class and both refusal classes are compiled by javac; the refusal classes also compile from JADX 1.5.6 output and preserve the same observed values. The original shape replay above freezes JVM-valid input behavior; the current Jarde CLI replay is recorded separately below.

## Current CLI replay (2026-09-26)

[`replay-current-2026-09-26.md`](replay-current-2026-09-26.md) records the deterministic three-way replay made with a private Cargo target. Run it with:

```sh
python3 openspec/evidence/java-syntax-2026-09-24/assert-core/extensions/replay-current.py \
  --cli /path/to/jarde-cli \
  --out "$(mktemp -d /tmp/assert-current.XXXXXX)"
```

The saved [`replay-output-current-2026-09-26/`](replay-output-current-2026-09-26/) contains the core/wrong-owner replay, original/JADX/Jarde extension sources and compile/run transcripts, class hashes, and full javap output. The `AssertDuplicateStatusWrite` fixture compiles with a mutable `Z` status field and two writes, then changes only the field flags to static-final-synthetic (`0x1018`); it passes `-Xverify:all`. JADX and Jarde preserve the duplicate assignment, which Java 8 source compilation rejects for a final field. The new `AssertHandlerBoundary` fixture uses a javac-generated exception table; after changing only the assertion-field access flags to `0x1018`, its class also passes `-Xverify:all`. Current Jarde explicitly marks `check(Z)V` as not recovered and explanation-only. Its partial class happens to compile, but that does not make the missing method body an equivalent recovery.

The existing `AssertCore-non01-arms.class` from `tests/fixtures/p3-assert-core/` is reused at SHA-256 `b52d39dd6ae7943d70b510c4925f23faadcc2fcc64c5cb7a2c309ae450de4e2c`; this extension does not introduce a second 2/3 patch.
