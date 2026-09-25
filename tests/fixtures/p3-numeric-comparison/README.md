# P3 numeric comparison conditions

This is the frozen Java 8 input for `recover-numeric-comparison-conditions` task 1.1. It is one
self-written class plus a source-only driver. The class was compiled on 2026-09-23 with javac
23.0.1 using:

```text
javac --release 8 -g:none -d v8 NumericComparisons.java
```

The compiler exited 0 with its three normal `--release 8` warnings. The committed class has one
class header and **35 Code attributes**: the default constructor plus 34 methods. Its class-file
version is 52.0, length is 2319 bytes, and SHA-256 is
`7ab3d3cc033ce1bed61b60a91a90e1ecf968495081b0578aff22bed716f44aa3`.

The first 21 methods are the existing comparisons baseline: seven long predicates, seven float
predicates and seven double predicates (`eq`, `ne`, `lt`, `le`, `gt`, `ge`, `not_lt`). `long_*`
uses `lcmp`; float and double methods cover both `fcmpg`/`fcmpl` and `dcmpg`/`dcmpl` choices. The
class also retains `int_lt` as a control and `double_boolean` as the boolean-convergence negative
shape.

The added methods pin the remaining acceptance boundaries:

| methods | bytecode fact |
| --- | --- |
| `callOrder` | left and right long operands are calls; javac emits the calls in source order, then `lcmp` and one zero branch |
| `callThrowLeft`, `callThrowRight` | one side throws; the original exception and call count identify which side was evaluated |
| `sameBlock` | positive direct same-block, adjacent `lcmp`/zero-branch consumer |
| `booleanMerge` | negative boolean result with two branch successors and a merge |

`CompareRunner.java` reuses the earlier comparisons driver shape. It runs 1309 long/float/double
input pairs including `Long.MIN_VALUE`/`Long.MAX_VALUE`, both NaNs, negative and positive zero,
both infinities and finite values; it then prints the int control, call order/count, both exception
paths, the positive same-block result and the negative boolean result.

## Pre-implementation baseline

The original class was run from the committed bytes and its output is preserved in the audit
directory `/tmp/jarde-numeric-comparison/original.txt`; its final lines are:

```text
int_lt/0/1=7
int_lt/1/0=9
callOrder=7:calls=12
callThrowLeft=IllegalStateException:left:calls=1
callThrowRight=IllegalStateException:right:calls=12
sameBlock=7
booleanMerge=true
```

The current debug CLI was run read-only with `class-source --policy single-class --release 8`. All
21 direct numeric methods still quote BCI 2 (`lcmp`, `fcmp*` or `dcmp*`) and do not produce a
condition, so the exact generated class source fails javac with missing-return errors. The call
methods retain each producer call and quote the comparison; `booleanMerge` and `sameBlock` retain
the comparison quote. The exact source, report and compiler log are in
`/tmp/jarde-numeric-comparison/jarde-before.java.txt`, `jarde-before.report.txt` and
`jarde-javac.log`. This is the required修前失败 record.

JADX 1.5.6 was run against the same class. Only its invented `package defpackage;` line was removed
before compiling the source-only copy. The JADX class verifies with `java -Xverify:all` and runs,
but its output differs from the original in 34 lines, all `float_not_lt`/`double_not_lt` cases with
at least one NaN. The diff is preserved at `/tmp/jarde-numeric-comparison/jadx-mismatches.diff`;
it is evidence, not an expected implementation result.

The Rust acceptance is in `tests/p3_numeric_comparison.rs`. Its non-JDK tests assert all 21 final
relations, compare/branch source anchors, the call operand shape, and the quoted boolean negative
shape. Its ignored JDK test compiles the recovered methods and runs the original/recovered traces
with `java -Xverify:all`, including NaN, signed zero, infinities, long extremes, call order,
single evaluation and exceptions. It does not introduce a parser or a test framework.
