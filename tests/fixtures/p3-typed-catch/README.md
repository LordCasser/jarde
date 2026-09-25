# P3 fixture: the `try`/`catch` the exception table itself states

`v8/TypedCatch.class` is a real compiled sample: the sibling `TypedCatch.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 TypedCatch.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `TypedCatch` |
| class-file version | 52.0 (Java 8) |
| bytes | 768 |
| SHA-256 | `efc70f670fa3577c42abeaa026e8b194d196a0839578add4fa9e30311b94f4a0` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_typed_catch.rs` (the five texts, the clause order and the content plane) |

## What the defect is, and what each member is for

A row of the exception table that names a `catch` type is not a resource header. The `twr@1` rule's
`resources()` treated every row covering the block it was examining as a try-with-resources
candidate, so a plain `try`/`catch` whose row *begins at the method's first instruction* — no
instruction precedes the protected range, so there is nothing the range's start could be the end of —
came back `jre_guard_resource_init`: the `if` was written, the `throw` was quoted under that code and
the handler was quoted as an uncovered block. The classification is what changed: a row no
instruction precedes is not a resource header at all, the walk states its own reason for the
exception edge, and the named rows become the clauses of a `try` the recovery writes.

A row may also be preceded by an ordinary local assignment — `int y = 2; int x = 1;` in front of the
`try`, which is what `initialisedBeforeTry` states. There the range *is* preceded by a `Store`, but
the value it stores is no resource's: `1` is pushed by a constant and nothing in the statement that
ends at that store is a `new` or an invocation. Such a row is no header either, and the guard check
says so without examining the row (`Verdict::NotGuarded`), while a store filled by `new Res()` or by a
call (`Res r = open(…)`) keeps its verdict — a `try`-with-resources whose close order the rule cannot
prove keeps degrading as one and is never rewritten as a user `catch`. What is read is the statement
that *ends* at the store: the growth stops where `y`'s own statement begins, and that boundary is not
evidence about `x`'s value. The `try` whose range begins inside the block that holds those assignments
is written with their instructions in front of it (`Catches::lead`), not swallowed into its braces.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `namedCatch(I)I` | `iload_0; ifge 12; new; dup; invokespecial; athrow; iload_0; ireturn; astore_1; iconst_m1; ireturn`, row `[0,13) → 14 IllegalArgumentException` | `if (arg0 < 0) { <jre_guard_resource_init quote of BCI 4> } else { return arg0; }` plus BCI 14 quoted as an uncovered block | `try { if (arg0 < 0) { <quote of BCI 4> } else { return arg0; } } catch (java.lang.IllegalArgumentException local1) { return -1; }` |
| `initialisedBeforeTry(I)I` | `iconst_2; istore_1; iconst_1; istore_2; iload_0; ifge 16; new; dup; invokespecial; athrow; iload_0; ireturn; astore_3; iload_2; ireturn`, row `[4,17) → 18 IllegalArgumentException` | the `if` with `int local1 = 2; int local2 = 1;` in front of it, its then-arm quoted under `jre_guard_resource_init`, plus BCI 18 quoted as an uncovered block | `int local2; int local1 = 2; local2 = 1;` written **before** the `try`, the protected range from BCI 4 inside it, and `} catch (java.lang.IllegalArgumentException local3) { return local2; }` after it |
| `twoCatches(I)I` | the same, with a second `if` (`bipush 10; if_icmple 26`) and a second handler at 31, rows `[0,27) → 28 IllegalArgumentException` and `[0,27) → 31 IllegalStateException` | the two `if`s, each then-arm quoted under `jre_guard_resource_init`, plus BCIs 28 and 31 quoted as uncovered blocks | the same `try`, with `} catch (java.lang.IllegalArgumentException local1) { return -1; } catch (java.lang.IllegalStateException local1) { return -2; }` in table order |
| `multiCatch(I)I` | `twoCatches`' body with one handler at 28, rows `[0,27) → 28 IllegalArgumentException` and `[0,27) → 28 IllegalStateException` | the two `if`s quoted under `jre_guard_resource_init`, plus BCI 28 quoted as an uncovered block | one clause — `} catch (java.lang.IllegalArgumentException \| java.lang.IllegalStateException local1) { return -3; }` — with the handler's body walked and written once |
| `finallyIncrements(I)I` | `iconst_0; istore_1; iload_0; istore_1; iload_1; iconst_1; iadd; istore_1; goto 18; astore_2; …; athrow; iload_1; ireturn`, row `[2,4) → 11 any` | unchanged by the fix | unchanged by the fix: `int local1 = 0; local1 = arg0; local1 = local1 + 1; return local1;` — the row's `any` type is not a `catch` and never becomes one, and the range holds no instruction that may throw, so the copy the compiler made for the exceptional path is not a region of this body at all |

The four members are the directions the classification takes and the control beside them:

The five members are the directions the classification takes and the control beside them:

* **`namedCatch`** is the first shape the change is about: one named row, no instruction before its
  protected range, and a handler that returns. The `if` and the `return n` of the source's `try`
  block stay inside the `try` (the task's own acceptance), the `throw new` stays quoted ([`init.rs`'s
  `new@1`](crates/jarde-java/src/init.rs) is another change's work), and the clause names the row's
  own class and the slot the handler stores into.
* **`initialisedBeforeTry`** is the second: the range is preceded by an ordinary assignment
  (`int x = 1;`, itself preceded by another statement), which fills no resource. The row is no header,
  the two assignments are written *before* the `try` — the block's own instructions in front of the
  range are the statement's lead, not part of its braces — and the clause's body is the handler's
  `return x`, which reads the local the assignment filled.
* **`twoCatches`** states that the clauses are the table's rows **in table order**: two rows with one
  range and two different handler entries are two clauses, and the text states them in the order the
  JVM dispatches in.
* **`multiCatch`** states the other direction: two rows with one range and **one handler entry** are
  one clause, `catch (A | B n)`, and that handler's body is walked — and written — once. Spelling it
  as two clauses would run the body twice in the text the class file does not run twice.
* **`finallyIncrements`** is the control: a row whose `catch_type` is `0` is never written as a
  `catch` (nor as a `finally`). It is the same rule the guarded shapes keep —
  `catch_type == 0` is left exactly as it was.
