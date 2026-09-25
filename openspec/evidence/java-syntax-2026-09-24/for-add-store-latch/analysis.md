# `for` add/store latch with `continue outer`

Date: 2026-09-24. This is isolated Java 8 evidence for the source pattern in
`ForAddStoreLatch.java`; it does not build or execute Jarde.

## Environment and frozen input

- `java -version`: OpenJDK 23.0.1 (2024-10-15)
- `javac -version` / `javap -version`: 23.0.1
- `jadx --version`: 1.5.6
- Compile command: `javac --release 8 -g:none -d frozen ForAddStoreLatch.java Runner.java`
- The class files report major version 52 (Java 8). SHA-256:
  - `ForAddStoreLatch.java`: `81abd7dce77b51e0c53257eac6bec661e5235f88f0f543b08abaaa2f2f157fc0`
  - `Runner.java`: `396d49a66faef3c37ccdfded7ee498657bcf1e4044cdd3bca63cafbc90779ee9`
  - `frozen/ForAddStoreLatch.class`: `c87ba91131c91059fea96008df93dd4f4a578178c30267fb1230fc9eef11a614`
  - `frozen/Runner.class`: `cbc8db4fb806ad0ca15f58924893f4cd00a817d67733a41e47fa5a3a06515e92`

## Source and bytecode correspondence

`ForAddStoreLatch.run` has the outer loop `for (index = 0; index < limit;
index = index + step)`. Its body adds `index` to `indexTotal`, then enters the
inner loop. When `index == skipAt`, `continue outer` must bypass
`afterInnerCount++` while still performing the outer update expression.

In `javap/ForAddStoreLatch.txt`, the inner condition is at BCI 22 and the
`index == skipAt` branch is at BCI 30. The taken `continue outer` goes from BCI
33 to BCI 45. BCI 42 is `iinc 5, 1`, the observable `afterInnerCount++`; thus
the continue bypasses that statement. BCI 45 through 48 are precisely:

```
45: iload_3             // index
46: iload_1             // step
47: iadd
48: istore_3            // index
```

The back edge at BCI 49 returns to the outer condition at BCI 8. This confirms
that the labeled continue retains the add/store update; it is not compiled as
`iinc` for the outer variable. The two `iinc` instructions in this method
update the inner-loop counter at BCI 36 and `afterInnerCount` at BCI 42.

## Executions

The original frozen classes were run line-by-line with
`java -Xverify:all -cp frozen Runner`. Each input has `step > 0`, so the outer
loop terminates. `indexTotal` demonstrates that iteration advances; the
10,000 multiplier makes the count of executions of the statement after the
inner loop visible in the same result.

| Input `(limit, step, skipAt)` | Iterated index values | Expected decomposition | Actual output |
|---|---|---|---|
| `(9, 2, 2)` | `0, 2, 4, 6, 8` | `20 + 4*10000` | `case1=40020` |
| `(10, 3, 3)` | `0, 3, 6, 9` | `18 + 3*10000` | `case2=30018` |
| `(11, 1, 4)` | `0` through `10` | `55 + 10*10000` | `case3=100055` |
| `(8, 2, -1)` | `0, 2, 4, 6` | `12 + 4*10000` | `case4=40012` |
| `(12, 4, 8)` | `0, 4, 8` | `12 + 2*10000` | `case5=20012` |

The original and reconstructed Runner outputs match byte-for-byte. JADX 1.5.6
was run with `jadx -d jadx-raw frozen/ForAddStoreLatch.class`. Its original
output is retained at `jadx-raw/sources/defpackage/ForAddStoreLatch.java`; it
reconstructs the control flow with nested `while` loops and `break`, and keeps
the outer add/store update after the inner loop. For the Java 8 recompilation,
that output was copied to `jadx-java8/ForAddStoreLatch.java` with only the
`package defpackage;` line removed to arrange it in the default package. The
unchanged `Runner.java` was compiled alongside it using
`javac --release 8 -g:none -d jadx-java8/classes ...`, then run with
`java -Xverify:all -cp jadx-java8/classes Runner`; it produced the same five
lines above.

There are no exception edges in this specimen: `javap` shows no exception
table, and the source operations on the exercised paths do not throw. Both
class sets pass full JVM verification (`-Xverify:all`); no runtime exception
was observed. This states the boundary of the evidence rather than inferring
exception handling behavior from this case.

## Separate minimal add/store loop

`simple/` is an independent minimal specimen for the outer-loop add/store
latch itself:

```java
for (int i = 0; i < limit; i = i + step) sum += i;
```

There is no inner loop, label, or `continue`; `step` is a read-only parameter.
This separates basic add/store loop handling from the control-flow shape in
the larger `ForAddStoreLatch` specimen. The larger specimen additionally
exercises labeled `continue` from a nested loop and the skipped observable
statement, so its Jarde baseline cannot by itself attribute a result to the
add/store latch.

The simple specimen was compiled with OpenJDK `javac --release 8 -g:none`;
the class files are Java 8 major version 52. `simple/javap/ForAddStoreSimple.txt`
shows the update at BCI 13–16 as `iload_3; iload_1; iadd; istore_3`, followed
by the backward branch at BCI 17 to the condition at BCI 4. Original classes
were run line-by-line with `java -Xverify:all -cp simple/frozen Runner`.
JADX 1.5.6 output is retained in `simple/jadx-raw/`; its source was placed in
the default package by removing only the `package defpackage;` declaration,
then compiled with Java 8 and run with `-Xverify:all`. Original and
reconstructed runs both printed:

```
case1=20
case2=18
case3=55
case4=12
case5=12
```

## Jarde baseline and independent replay

Root replayed both frozen classes with the same post-1.4 Jarde CLI (SHA-256
`95362354d3de2af1ac57f69ea9f7492731ca580afd2e5f7e3b16fc4144b3aa8c`).
The simple class (`87e243ac748f749852c6969a5d490c7ac24d6f863859977faf31202461474c3c`)
has a complete `run(II)I`, but spells the counted loop as `while` with
`local3 = local3 + arg1` in the body. Root rebuilt the source, original frozen
class and JADX's default-package-only adjusted source with Java 8, and all
three produce the same five lines above under `-Xverify:all`. This isolates a
presentation gap: the add/store operation itself is already rendered correctly.

The complex class (`c87ba91131c91059fea96008df93dd4f4a578178c30267fb1230fc9eef11a614`)
is still explanation-only. Its report first records `jre_region_loop_shape` at
outer header BCI 8, then `jre_region_uncovered_blocks [52,33]`; the method body
is finally quoted because `local 3 crosses a quoted fallback region`. Unlike
the simple class, this one cannot be compiled from Jarde output. The earliest
regional refusal, rather than the later local-scope diagnostic, is the relevant
starting point for the nested `continue` investigation. A `while` with an
ordinary Java `continue outer` would skip the update at BCI 45–48 and change
the five observed results; recovery must prove and move exactly that update
into a `for` header or preserve a local refusal.

## Implementation acceptance (3.2 add/store subcase)

Root independently replayed CLI SHA-256
`63cf72aef794786c3f34e3c232607eb6ebaa168c9ca08a517a473d95323008f2`
against both frozen classes. The simple `run(II)I` now contains exactly one
`for (local3 = 0; local3 < arg0; local3 = local3 + arg1)` and no quoted BCI.
The complex `run(III)I` contains `jarde_loop_8: for` with that same update,
an inner `for` for its independently proved `iinc`, and `continue jarde_loop_8`
at the BCI 33 transfer; BCI 42 remains `local5 = local5 + 1` after the inner
loop. It has no quoted BCI. The requested source map has entries for BCI 33,
42 and each of 45, 46, 47, 48.

Root rebuilt both original Java sources with `javac --release 8 -g:none` and
checked their class bytes equal the frozen SHA-256 values above. The original,
JADX's default-package-only adjusted source, and Jarde's unedited complete
class source were each rebuilt with Java 8 and executed under
`java -Xverify:all` with the same Runner. For each class all three five-line
outputs match byte-for-byte; the simple lines are `20,18,55,12,12`, and the
complex lines are `40020,30018,100055,40012,20012` in case order.

The frozen `ForAddStoreBoundaries.class` checks four refusal boundaries:
`changedStep`, `sharedTailEffect` and `observedAfter` remain complete `while`
methods with their effects/returns; `extraConsumer` retains a `while` but
quotes BCI 13, 14 and the dependent instruction group because the duplicated
update value has an additional consumer. None is projected to `for`. The
`present-proved-java-structure` task 3.2 remains open for its other conditions.
