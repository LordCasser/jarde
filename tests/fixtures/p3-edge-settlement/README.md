# P3 fixture: how one exception edge settles a block (P3 2.15)

`v8/Settled.class` is a real compiled sample: the sibling `Settled.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Settled.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时…` / `目标值 8 已过时` /
`要隐藏有关已过时选项的警告…` warnings (3 of them) and writes the class anyway, exit code 0. The
compiler is a generation-only input: it is not needed at run time, so the sample is committed as
bytes.

| property | value |
| --- | --- |
| class | `Settled` |
| class-file version | 52.0 (Java 8) |
| bytes | 656 |
| SHA-256 | `17d173ad23124d9e6d509e9254c0cde4f3199f0810de767fbe42e3f5cd64cf9c` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `localN`) |
| read by | `tests/p3_edge_settlement.rs` (the four members' texts, their clauses and their quote of the dead handler) |

## What the two settlement questions are

One exception edge, one block, two facts about the pair. Both are facts of the same decode
(`region.rs`'s `Walker`):

* **Can the block take the edge?** The graph states the table's rows two ways (P3 2.9): a record a
  `may_throw` instruction of the body **covers** keeps the edge its sites feed, and a record **no**
  such instruction covers is stated by its protected range — one edge from every block the range
  intersects. The second kind cannot be taken: nothing in the block can raise, so no run of it ever
  enters the handler. `javac --release 8` writes exactly such a row for a `try` body that runs code
  and nothing else, and reading the edge as a way *out* of the block quoted the whole statement
  instead of writing it (`increments`/`noThrowFinally` before P3 2.15; the same shape's
  `finallyIncrements(I)I` in `tests/fixtures/p3-typed-catch/`).
* **Does the row protect this block?** The walk settles a block by the row the edge's ordinal names
  when that row's `[start_bci, end_bci)` and the block's own span **intersect**, the handler matches
  and the row names a `catch` type. The canonical graph fuses straight-line code, so a row may begin
  *inside* the block it protects: the inner range of `nested(I)I` begins at BCI 8 in the block
  `[7, 11)` that the outer clause's binding store (`astore_1` at BCI 7) opened, and the row of
  `calls(ILjava/lang/Runnable;)I` begins at BCI 2 in the block `[0, 8)` that the `int x = n;` lead
  opened. Requiring the range to cover the block's **start** quoted such a block — and the statement
  the bytes hold with it — because the graph's fusion, not the table, was what the test read.

## The members, as `javap -c -p` prints them

```text
  static int increments(int);
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
      11: ireturn
    Exception table:
       from    to  target type
           0     4     7   Class java/lang/RuntimeException

  static int nested(int);
    Code:
       0: iload_0
       1: iconst_1
       2: iadd
       3: istore_0
       4: goto          18
       7: astore_1
       8: bipush        -2
      10: istore_0
      11: goto          18
      14: astore_2
      15: bipush        -3
      17: istore_0
      18: iload_0
      19: ireturn
    Exception table:
       from    to  target type
           0     4     7   Class java/lang/RuntimeException
           8    11    14   Class java/lang/IllegalArgumentException

  static int noThrowFinally(int);
    Code:
       0: iconst_0
       1: istore_1
       2: iload_0
       3: istore_1
       4: iload_1
       5: iconst_1
       6: iadd
       7: istore_1
       8: goto          18
      11: astore_2
      12: iload_1
      13: iconst_1
      14: iadd
      15: istore_1
      16: aload_2
      17: athrow
      18: iload_1
      19: ireturn
    Exception table:
       from    to  target type
           2     4    11   any

  static int calls(int, java.lang.Runnable);
    Code:
       0: iload_0
       1: istore_2
       2: aload_1
       3: invokeinterface #11,  1           // InterfaceMethod java/lang/Runnable.run:()V
       8: goto          15
      11: astore_3
      12: bipush        -4
      14: istore_2
      15: iload_2
      16: ireturn
    Exception table:
       from    to  target type
           2     8    11   Class java/lang/IllegalArgumentException
```

## What each member is the case of

| member | rows | what it states |
| --- | --- | --- |
| `increments(I)I` | `[0, 4) → 7 RuntimeException` | the range holds `iload_0; iconst_1; iadd; istore_0` — no throwing instruction — and names a clause. The statement is written, with its clause: `try { arg0 = arg0 + 1; } catch (java.lang.RuntimeException local1) { arg0 = -1; } return arg0;`. This is the shape `steps(I)I` of `tests/fixtures/p3-stated-rows/` states on a second fixture: it must not regress. |
| `nested(I)I` | `[0, 4) → 7 RuntimeException`, `[8, 11) → 14 IllegalArgumentException` | a `try`/`catch` whose clause body is itself a `try`/`catch`, and the inner range **begins inside** the block the outer clause's binding store (`astore_1` at BCI 7) opened. Both statements are written: `try { arg0 = arg0 + 1; } catch (RuntimeException local1) { try { arg0 = -2; } catch (IllegalArgumentException local2) { arg0 = -3; } } return arg0;` — this is the mid-block range the settlement reads by intersection. |
| `noThrowFinally(I)I` | `[2, 4) → 11 any` | the same site-less range under the **catch-all** row a `finally` copy is written under (no clause is written for it, in this build or any other). The body is the method's normal flow — `int local1 = 0; local1 = arg0; local1 = local1 + 1; return local1;` — and the copy at BCI 11 (the `astore_2; …; athrow` no instruction can enter from) is named by the uncovered-blocks quote. |
| `calls(ILjava/lang/Runnable;)I` | `[2, 8) → 11 IllegalArgumentException` | the **live** half of the mid-block question: `invokeinterface run` at BCI 3 is the throwing instruction the row covers, and the range begins inside the block the `int x = n;` lead opened. The lead is written before the statement (`int local2; local2 = arg0;`), the call is the `try`'s body and the clause follows it. |

## The bodies' other facts

Every handler entry is a `StackMapTable` frame's `same_locals_1_stack_item` state; the derivation this
crate performs never reads it, and the handler state it publishes is the source block's own exit with
the record's caught reference (see `crate::frame`). No member branches, so no `StackMapTable` entry
of the protected ranges themselves is needed to read them.
