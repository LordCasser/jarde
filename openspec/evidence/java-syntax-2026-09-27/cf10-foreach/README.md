# CF-10 foreach audit (2026-09-27)

Baseline: Jarde `5a187212dce9086118f63c43f35f06b8c8bd59f1` (`codex/cf10-foreach-audit`); JADX checkout `/Users/lordcasser/workspace/testzone/jadx`, commit `2fb1b16386941660fda07e9017285aec40fcb37f`.

## Frozen test and implementation inputs

SHA-256 hashes below identify the exact fixed JADX files audited.

| File | SHA-256 |
|---|---|
| `TestArrayForEach.java` | `26312025f90e53e283b7044f1c184411ca9ba5300f325f04d248ada9613326d6` |
| `TestArrayForEach3.java` | `a67f5b84ae14d3f657d2688fd37ec71c5882fb11ba1a8d0c041666cfadbf0299` |
| `TestArrayForEachNegative.java` | `d4c880e095351c3cf665c580dd7d5236178366d7584a5c6d3e474337e09c8c8f` |
| `TestIterableForEach.java` | `115ee466f2d428b5372deff6cec7aa5a0246573fd5ede80b24f74153a0728012` |
| `TestIterableForEach3.java` | `0486f737f156d1ac843056deb5be2bff3d46cafc78b77e52220fd44dcc8a989e` |
| `LoopRegionVisitor.java` | `921f09e8934fda33452a00359588b293117f160813477e7068fa1ec3b71e6515` |
| `ForEachLoop.java` | `e16fba634dbc7dda08e6a69e5a0263953f0a5b1ffdae4d12272a2bab458a8321` |
| `RegionGen.java` | `8b80ef618487e38cb09a2bbd81b70de0dd096996a321a0c9dcf55e1dc011d4ee` |

The inventory row chooses five of the eight loop files as representatives; the older ledger's directory-wide list includes three additional variants. Assertion strength is uneven: `TestArrayForEach` and `TestIterableForEach` assert output snippets; `TestArrayForEach3` asserts an exact foreach line and absence of `while`; `TestIterableForEach3` checks foreach and nested-if text. These do not establish full-class equivalence. `TestArrayForEachNegative` bundles eight shapes and only asserts the stripped output has no `:` while compilation is disabled, so it does not prove the indexed loop survives or executes. The test-specific checks do not collectively enforce a full-source compile/run for every positive.

JADX's array recognizer checks the normalized induction (start at zero, `+1`, `< length`), same array for length and element load, and loop-local uses before making a `ForEachLoop`. Its iterable path requires the expected iterator/hasNext/next structure and use constraints. `ForEachLoop` represents the iterable and element as region args; `RegionGen.makeLoop` emits the enhanced-for header.

## Three-way executable slice

`input/ForeachCases.java` has an array foreach (`sum`), an `Iterable<String>` foreach (`join`), and a terminating non-foreach loop (`everyOther`, stride `i += 2`). The checked-in complete Java source is compiled with `javac --release 8 -g -Xlint:-options`. JADX decompiles the resulting `ForeachCases.class`; both original and JADX full sources compile under Java 8 and run with `java -Xverify:all`. Their output is identical:

```
10
abc
4
```

JADX retains the stride loop as indexed `for (int i = 0; i < values.length; i += 2)`; it does not rewrite it as foreach. Jarde recovers `sum` as `for (int value : local2)` and `join` as `for (java.lang.Object iteratorElement25 : values)` with the String cast inside. It does not rewrite `everyOther` as foreach, but declines its method body with `local 1 crosses a quoted fallback region`; this alone makes the assembled Jarde class fail `javac --release 8` with a missing return. The same assembled source **also** quotes part of `main`: the invocation of `join(Iterable)` receives a `List` from `Arrays.asList`, and `invocation_argument` has no safe reference-conversion evidence. Separate complete classes isolate these as (a) a stride-index method's local definition/use presentation across a quoted region and (b) a `List → Iterable` call-argument typing gap without any foreach or overload. Full details, replay commands, and exact hashes are in the [failure-isolation report](isolation/README.md). The local Region failure's internal path remains untraced; neither result establishes that the counted-loop failure shares CF-08's cause.

Artifacts in this directory preserve the input, original compiled class, JADX source/classes, Jarde assembled source, and run outputs. Reproduction commands:

```sh
javac --release 8 -g -Xlint:-options -d original input/ForeachCases.java
jadx -d jadx original/ForeachCases.class
cargo run -q -p jarde-cli -- class-source --input original/ForeachCases.class --class ForeachCases --policy single-class --release 8 --format text > jarde/ForeachCases.java 2> jarde/report
javac --release 8 -g -Xlint:-options -d jadx-classes jadx/sources/defpackage/ForeachCases.java
java -Xverify:all -cp original ForeachCases
java -Xverify:all -cp jadx-classes defpackage.ForeachCases
javac --release 8 -g -Xlint:-options -d jarde jarde/ForeachCases.java # expected failure: missing return in everyOther; main also contains a quoted invocation
```

Jarde focused verification on this baseline:

```sh
CARGO_TARGET_DIR=/tmp/jarde-cf10-target CARGO_INCREMENTAL=0 cargo test --test p3_array_foreach --test p3_iterable_foreach
```

Result: array tests 6/6 and Iterable tests 5/5 passed. These existing tests include Jarde-positive full-source Java 8 recompilation/`-Xverify:all` behavior comparisons and explicit refusal assertions for array/index and iterator boundaries. Cargo target cleanup is performed after commit.
