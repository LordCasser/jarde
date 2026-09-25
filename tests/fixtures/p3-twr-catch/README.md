# P3 fixture: a resource header with a user `catch` beside it

`v8/Combo.class` is a real compiled sample: the sibling `Combo.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Combo.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时…` / `目标值 8 已过时` /
`要隐藏有关已过时选项的警告…` warnings (3 of them) and writes the class anyway, exit code 0. The
compiler is a generation-only input: it is not needed at run time, so the sample is committed as
bytes.

| property | value |
| --- | --- |
| class | `Combo` |
| class-file version | 52.0 (Java 8) |
| bytes | 550 |
| SHA-256 | `c047a6ce98b5e224419210223b08c3a373f3aaa9aaf290a7b7bfe6ff6ade459f` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_twr_catch.rs` (the statement, the tail the walk cannot reach, and the acceptance the `returns` increment lands) |

## What the member states

`read([B)I` is the commonest resource idiom there is:

```java
static int read(byte[] data) throws IOException {
    try (InputStream in = new ByteArrayInputStream(data)) {
        return in.read();
    } catch (IOException e) {
        return -1;
    }
}
```

and javac writes **four** exception-table rows for it (`javap -c -p`):

```text
 0: new           java/io/ByteArrayInputStream
 3: dup
 4: aload_0
 5: invokespecial ByteArrayInputStream."<init>":([B)V
 8: astore_1                              <- the header's resource, in slot 1
 9: aload_1
10: invokevirtual InputStream.read:()I
13: istore_2                              <- the source's `return in.read()`, value kept in local2
14: aload_1
15: invokevirtual InputStream.close:()V   <- the normal path's close — **no** `ifnull` before it
18: iload_2
19: ireturn                               <- the return the value was kept for
20: astore_2
21: aload_1
22: invokevirtual InputStream.close:()V   <- the compiler's cleanup — no `ifnull` here either
25: goto          34
28: astore_3
29: aload_2
30: aload_3
31: invokevirtual Throwable.addSuppressed:(Ljava/lang/Throwable;)V
34: aload_2
35: athrow
36: astore_1
37: iconst_m1
38: ireturn                               <- the source's `catch`, parameter in slot 1 again
Exception table:
   from    to  target type
      9    14      20   Class java/lang/Throwable   <- the header's own cleanup
     21    25      28   Class java/lang/Throwable   <- ... and the close's own self-protection
      0    18      36   Class java/io/IOException   <- the source's `catch` ...
     20    36      36   Class java/io/IOException   <- ... split in two, for the cleanup's code
```

## What the defect was

Two things, and the first is not the row union:

* the resource's own initialisation is a `new`, so javac **knows** it cannot be null and writes no
  null test at all — neither on the normal path (`14: aload_1; 15: close`) nor in the handler
  (`20: astore_2; 21: aload_1; 22: close; 25: goto`). `guard.rs`'s `close_handler` read the checked
  shape only (`astore p; aload r; ifnull L`), so the `twr@1` rule refused the member at BCI 20 with
  `jre_guard_handler` — *"the handler's own instruction sequence is not the one this rule proves"* —
  before any row question was asked;
* the statement's rows in that one block are the **union** of two: the compiler's cleanup row begins
  at BCI 9, the source's clause row at BCI 0, and `catches()` asked for one start. So neither the
  guarded rule nor the typed `catch` read the shape either, and the member was quoted whole.

## What the tree reads now, and what stands

`close_of_level`/`normal_close` read the unchecked close on both paths; `catches()` reads the rows
that reach a handler the claim **proved** as the compiler's own (never as a `catch (Throwable)`);
and `twr` reads the enclosing rows as the clauses of a `try` the statement sits inside — but only
where that is what they are: a real `try (…) {} catch {}` emits **two** user rows, because the
clause must also cover the cleanup's rethrow, while a single row spanning `[start, handler_end)` is
the `catch` a compiler winds around the whole construct (`Guarded.withCatch`, `tests/p3_guard.rs`).
That row keeps today's `jre_guard_unexplained_row` refusal.

What stands unproven is the **tail**. With no null test there is no branch, so the whole member is
one canonical block `[0, 3, 4, 5, 8, 9, 10, 13, 14, 15, 18, 19]`: after the close the run continues
at the `iload_2; ireturn` *inside* the statement's own block, and a walk continues at a block.
Claiming the block would leave the source's `return local2;` out of the artifact with nothing naming
it — the silence P3-R7 exists to undo — so the statement is refused with that as its reason
(`jre_guard_continuation`). Writing it needs the mechanism P3 2.6 already has for `synchronized`
(a `returns` on the shape, appended **inside** the braces by `build.rs`); until then the member is
refused whole, never presented as a `try` without its resource and never as a `try (…)` without its
`catch`. `tests/p3_twr_catch.rs` states both sides: the refusal that stands today, and the
`#[ignore]`d acceptance for the presentation.
