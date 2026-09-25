# P3 fixture: two protected ranges that begin at one instruction and nest

`v8/Nest.class` is a real compiled sample: the sibling `Nest.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Nest.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时…` / `目标值 8 已过时` /
`要隐藏有关已过时选项的警告…` warnings (3 of them) and writes the class anyway, exit code 0. The
compiler is a generation-only input: it is not needed at run time, so the sample is committed as
bytes.

| property | value |
| --- | --- |
| class | `Nest` |
| class-file version | 52.0 (Java 8) |
| bytes | 361 |
| SHA-256 | `839e4803e97d5ed1768dddd255ccbba794f53a0b82c4b0cc1e980f8b329a83b0` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_nested_try.rs` (the nesting, the clause order and the content plane) |

## What the member states

`nest(Ljava/lang/Runnable;)I` (`javap -c -p`, and the same bytes' exception table):

```text
 0: aload_0
 1: invokeinterface java/lang/Runnable.run:()V
 6: goto          12
 9: astore_1
10: iconst_m1
11: ireturn
12: goto          19
15: astore_1
16: bipush        -2
18: ireturn
19: iconst_0
20: ireturn
Exception table:
   from    to  target type
      0     6       9   Class java/lang/IllegalArgumentException
      0    11      15   Class java/lang/RuntimeException
```

The two rows **begin at the same instruction** (BCI 0) and end at different ones: `[0, 6)` is the
inner `try` and `[0, 11)` the outer one. The inner **handler entry (BCI 9) lies inside the wider
range** — the wider range protects the whole inner statement, handler and all — and the protected
body is `aload_0; invokeinterface run` at BCI 0 and 1: a call, not a `throw new`. The inner handler
returns `-1` (BCI 10 and 11, the `ireturn` at 11 being outside both ranges), the outer handler
returns `-2` (BCI 15 to 18), and both have their `ireturn` at BCI 11/18, so the code after the two
statements is the `return 0` at BCI 19 and 20.

## What the defect was

`crates/jarde-java/src/guard.rs`'s `catches()` answered `None` for every block whose named rows did
not declare **one** `(start, end)` — the comment named these two rows "a nesting this statement
cannot state". The member was then quoted whole: `jarde: not recovered … (explanation only)`,
`// @bytecode 0` with `a handler's shape is not part of the recoverable subset`, and the three live
blocks `[12, 9, 15]` — the inner handler, the outer handler and the code after both statements —
stated as *uncovered blocks*.

| | text |
| --- | --- |
| before the nesting | explanation only: the block at BCI 0 quoted under `jre_region_exception_edge`, and the blocks `[12, 9, 15]` quoted as uncovered |
| after the nesting (P3 2.5) | `try { try { … } catch (java.lang.IllegalArgumentException local1) { return -1; } } catch (java.lang.RuntimeException local1) { return -2; }` and then `return 0;`, with the body's own block still quoted inside the inner `try` |
| now (P3 2.7) | the same nesting, with the inner body written as its own statement: `arg0.run();` |

The body's block used to be quoted — not written — because it carries the `invokeinterface`'s own
exception edge, which the plain flow cannot leave through. P3 2.7 writes it: **both** rows of the
table protect that call, and each row's handler is one of the clauses the nesting already writes
around the block, so the call is a statement of the inner `try` rather than bytecode quoted under
it. What this fixture adds is the *nesting*: the inner statement's clause is written inside the outer
statement's body, the outer clause after it, and neither is a sibling clause of the other. The
`Guarded` sample's `syncThrowsCatching`/`secondInitFailsCatching` state the plain, un-nested half of
the same reading in `tests/p3_execution_comparison.rs`.
