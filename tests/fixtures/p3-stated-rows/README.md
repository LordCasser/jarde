# P3 fixture: the exception table is the artifact's own statement of what a row protects

`v8/Stated.class` is a real compiled sample: the sibling `Stated.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Stated.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Stated` |
| class-file version | 52.0 (Java 8) |
| bytes | 443 |
| SHA-256 | `a7e449e308b1622c6eefa5f09b482d6a397ec0de3d717bad9183c61ab319e1ad` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `localN`) |
| read by | `tests/p3_stated_rows.rs` (both method texts, their members' bodies and their diagnostics) |

## What the defect is, and what each member is for

`javac --release 8` protects the instructions a `try` body runs and does **not** ask whether any of
them can raise. `try { n = n + 1; } catch (RuntimeException e)` is `iload_0; iconst_1; iadd;
istore_0` under a row whose range `[0, 4)` holds no instruction the opcode table calls throwing, and
`try { n = n * 2; } catch (IllegalStateException e)` is the same shape over `[10, 14)`. The exception
table states those rows anyway, and a graph that built its exception edges only at `may_throw`
instruction sites made both rows invisible: their handler entries were entered by no edge, and the
`catch` clauses the bytes declare were not presented at all.

The two members are the two ways that showed:

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `steps(I)I` | two sequential `try`/`catch`; rows `[0, 4) → 7 RuntimeException` and `[10, 14) → 17 IllegalStateException`, neither range holding a throwing instruction | the second handler's bytes (BCI 17, 18, 20) were in no canonical block and in no dead-node list, so the run quoted the whole body under `jre_region_unaccounted_instruction` (`// @bytecode 0 10 21 17 18 20`) | both `try {`s and both `catch (`s, each clause holding its own statement, `return arg0;` after them, and no `@bytecode` |
| `nested(I)I` | a `try`/`catch` whose `catch` body is itself a `try`/`catch`; rows `[0, 4) → 7 RuntimeException` and `[8, 10) → 13 IllegalArgumentException`, neither range holding a throwing instruction | the two ranges covered no block *start*, so the graph's only node was the fused normal path `[0, 19)` and the text was `arg0 = arg0 + 1; return arg0;` — the `catch` clauses were **silently dropped** | the outer `catch (java.lang.RuntimeException local1)` is written and the inner row is *stated* rather than dropped. P3 2.9 left that inner statement quoted — the binding store at BCI 7 was read as a resource header's initialisation (P3 2.14) and the region walk did not account the inner row as the block's way out (P3 2.15) — and with both readings in place the text is the whole statement: `try { arg0 = arg0 + 1; } catch (java.lang.RuntimeException local1) { try { arg0 = -1; } catch (java.lang.IllegalArgumentException local2) { arg0 = -2; } } return arg0;` |

The rule both members pin is one fact about the table, read once per record: a record a throwing
instruction of the body covers keeps the edges its sites feed (exactly as before), and a record
**no** such instruction covers is stated by its protected range — one `Exception` edge from every
block that range intersects. The canonical handler rows' `protected` is read out of those edges.
`throw_sites` stays the runtime fact — only instructions that may throw are listed, and a site-less
range invents no entry. `steps` is the case the change is about; `nested` is the case whose row the
graph states and whose *presentation* was completed later — the binding store is a clause parameter,
not a resource header (P3 2.14), and the inner block is accounted by the row that protects it
(P3 2.15). See `tests/p3_stated_rows.rs` for the clause levels the fixture now pins.

## The bodies, as `javap -c -p` prints them

```text
  static int steps(int);
    Code:
       0: iload_0
       1: iconst_1
       2: iadd
       3: istore_0
       4: goto          10
       7: astore_1
       8: iconst_m1
       9: istore_0
      10: iload_0
      11: iconst_2
      12: imul
      13: istore_0
      14: goto          21
      17: astore_1
      18: bipush        -2
      20: istore_0
      21: iload_0
      22: ireturn
    Exception table:
       from    to  target type
           0     4     7   Class java/lang/RuntimeException
          10    14    17   Class java/lang/IllegalStateException
```

```text
  static int nested(int);
    Code:
       0: iload_0
       1: iconst_1
       2: iadd
       3: istore_0
       4: goto          17
       7: astore_1
       8: iconst_m1
       9: istore_0
      10: goto          17
      13: astore_2
      14: bipush        -2
      16: istore_0
      17: iload_0
      18: ireturn
    Exception table:
       from    to  target type
           0     4     7   Class java/lang/RuntimeException
           8    10    13   Class java/lang/IllegalArgumentException
```

Both bodies carry a `StackMapTable` (4 entries for `steps`, 3 for `nested`) whose `same_locals_1_
stack_item` frames are the handlers' own entry states; the derivation this crate performs never reads
it, and the handler state it publishes for a site-less row is the source block's own exit with the
record's caught reference — see `crate::frame` and the analysis in the change that added this
fixture.
