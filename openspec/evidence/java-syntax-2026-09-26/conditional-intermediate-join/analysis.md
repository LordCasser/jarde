# Conditional value with an intermediate join

This read-only probe isolates a Java 8 conditional whose inner value first joins, then participates in arithmetic, then feeds an outer value join:

```java
return a > 0 ? ((a > 1 ? f1() : f2()) + 3) : f3();
```

`f1`, `f2`, and `f3` append distinct trace labels and can throw distinct exceptions. This distinguishes lazy arm selection, arithmetic, and exception propagation. The frozen sources are `ConditionalIntermediateJoin.java` and `Runner.java`; both were compiled with `javac --release 8 -g:none`.

## Results

Original bytecode and the JADX 1.5.6 decompilation both compile. Their six runner lines match byte-for-byte (SHA-256 `58f6f6799fdb0e4173507a9fdb6d9f62cdc57478cdb0da41c6178048f8a887ef`):

- `a=0` calls only `f3` and returns 30.
- `a=2` calls only `f1` and returns 13; `a=1` calls only `f2` and returns 23.
- When selected, each helper's distinct exception propagates after its trace write. Unselected helpers do not run.

JADX emits `return (i > 1 ? f1() : f2()) + 3;` under the outer `if`, followed by `return f3();`. The full JADX source is `jadx.java.txt`. `frozen-input.jar` preserves the exact class and runner used as the JADX/Jarde input; `ConditionalIntermediateJoin.class` preserves the original class bytes independently.

The current Jarde full-class report completes, but `choose(I)I` is `quality=fallback`, `representation=mixed`. Its body has an empty `if (arg0 > 0)`, a quoted BCI 23 producer, and an unproduced entry stack value at BCI 26. Full-class compilation fails with one `missing return statement` diagnostic at the end of `choose`; no Jarde runtime comparison was attempted. The complete emitted class is `jarde.java.txt`, with the machine-readable report in `jarde-report.json` and the focused method report in `jarde-choose-report.json`.

The method's bytecode gives the distinction directly (`original-javap.txt`): outer test at BCI 0 branches to BCI 23; inner test at BCI 4 chooses calls at BCIs 9 and 15; both paths reach BCI 18, where the inner stack value is consumed by `iadd` at BCI 19; the result transfers at BCI 20 to outer join BCI 26. The outer false arm calls `f3` at BCI 23 and also reaches BCI 26, where `ireturn` consumes the final stack value. Thus the inner Phi and outer Phi are distinct: the first has an arithmetic consumer between joins, while the second is the method's return value. The Jarde report additionally records BCI 18 as an uncovered live block reached only through an edge omitted by its normal-flow view; this is consistent with the missing composed region, but this probe does not infer which individual check first caused rejection.

## Architecture reading

Jarde's established two-arm proof (`crates/jarde-java/src/build.rs:2479-2493`) accepts straight-line arms, proves a two-predecessor join and its stack Phi, and confirms a unique instruction consumer in that join (`:2893-2918`). Its builder then requires both complete arms to be straight regions (`:7895-7903`, `:8219-8225`). The completed nested-tree proof (`:2930-2936`) intentionally requires every internal `If` to share the root join, every leaf to be straight-line, and each leaf to supply a distinct input of that same stack Phi. Here the inner join is BCI 18, while the root join is BCI 26, so that proof's defining invariant excludes the case. Applying the two proofs independently does not suffice: the outer proof must accept a non-straight arm containing the inner conditional and the post-join `iadd`, and its Phi input is the value of that bridge operation rather than a leaf producer. Existing `conditional_values` can publish an expression keyed by Phi, but there is no single proved ownership plan that connects the child Phi, BCI 19, parent arm, and outer Phi atomically.

A narrow extension can remain private: build a bounded composition plan over proved joins and the value-producing straight-line instructions between them. For this fixture that plan would record (1) inner conditional test and arms, (2) its unique stack Phi at BCI 18, (3) the single-use `iadd` at BCI 19 with constant 3, (4) the outer conditional and its BCI 23 arm, and (5) the unique outer stack Phi/`ireturn` consumer at BCI 26. Validate decoded edges and each join's actual predecessors independently; require every intermediate Phi and bridge result to have the exact expected single use; render inner Phi as an expression operand for the bridge before building the outer arm; and publish/suppress the whole composed source set atomically. Preserve current refusal boundaries for extra uses, independent effects, exception edges, ambiguous joins, or unsupported operations. This reuses existing proof primitives and expression forms, but needs a new private composition/ownership mechanism; simply relaxing the same-root-join predicate would be unsound because it would not account for the bridge operation or its evaluation order.

The local JADX `TernaryMod` algorithm is also narrow: it accepts two result-producing arm blocks only when each arm is a one-instruction block and both SSA vars have the same sole Phi use (`TernaryMod.java:84-111`). It removes the arm instructions, wraps their value expressions into a `TernaryInsn`, and replaces the branch header (`:113-145`), then shrinks code. Traversal and shrinking expose this inner ternary to the following arithmetic; the outer source form is retained as control flow. This probe shows the output is correct for the fixture, but the source-level result does not itself provide the edge, ownership, use-count, and exception-order proof Jarde needs.

## Reproduction commands

Run from the repository root. The checked-in frozen class and JAR are the long-lived inputs. To prove the class came from the source, compile into a temporary directory and compare its hash to the frozen `.class`. The Cargo target is isolated under `/tmp` and should be removed after replay.

```sh
BASE=openspec/evidence/java-syntax-2026-09-26/conditional-intermediate-join
(cd "$BASE" && shasum -a 256 -c SHA256SUMS.txt)
mkdir -p /tmp/jarde-conditional-intermediate-original
javac --release 8 -g:none -d /tmp/jarde-conditional-intermediate-original "$BASE/ConditionalIntermediateJoin.java" "$BASE/Runner.java"
cmp /tmp/jarde-conditional-intermediate-original/ConditionalIntermediateJoin.class "$BASE/ConditionalIntermediateJoin.class"
shasum -a 256 /tmp/jarde-conditional-intermediate-original/ConditionalIntermediateJoin.class
java -cp "$BASE/frozen-input.jar" Runner
javap -classpath "$BASE/frozen-input.jar" -c -p -s -v ConditionalIntermediateJoin
jadx -d /tmp/jarde-conditional-intermediate-jadx "$BASE/frozen-input.jar"
CARGO_TARGET_DIR=/tmp/jarde-conditional-intermediate-cargo-target cargo build -q -p jarde-cli
/tmp/jarde-conditional-intermediate-cargo-target/debug/jarde-cli class-source --input "$BASE/frozen-input.jar" --class ConditionalIntermediateJoin --evidence all
/tmp/jarde-conditional-intermediate-cargo-target/debug/jarde-cli class-source --input "$BASE/frozen-input.jar" --class ConditionalIntermediateJoin --format json --output /tmp/jarde-conditional-intermediate-report.json --evidence all

For generated-source compilation, use `javac --release 8 -g:none -Xlint:-options`. JADX assigns default-package classes to `defpackage`, so the temporary runner must use that package when compiled with its source. Jarde emits its own complete-class wrapper and is compiled with the original package-less Runner. `original-javac.log`, `jadx-javac.log`, and `jarde-javac.log` preserve outcomes. `tool-versions.txt` records JADX 1.5.6 and OpenJDK/javac 23.0.1.

`SHA256SUMS.txt` contains repository-relative paths only and verifies every frozen evidence file except the manifest itself. It covers the source, runner, original class, frozen input JAR, JADX source and runtime output, Jarde source/report, bytecode listing, compile/recompile logs, commands, tool versions, and this analysis. `recompile-class.log` records the `javac --release 8 -g:none` byte comparison against the frozen class. Recompiling the source with the recorded Java toolchain produced a `ConditionalIntermediateJoin.class` whose SHA-256 matches the frozen class: `d135bb8ff9515fe79713cfec396245f96e6267d7a7b95bddabb02e7a62f36e92`. The Jarde CLI is identified by its source commit and binary hash in `tool-versions.txt`; that temporary binary is intentionally outside the checksum manifest.
