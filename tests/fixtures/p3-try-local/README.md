# P3 fixture: the Java 9 resource the compiler copies into a local

`v9/Held.class` is a real compiled sample: the sibling `Held.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 9 -g:none -d v9 Held.java
```

The compiler prints nothing and writes the class, exit code 0. The compiler is a generation-only
input: it is not needed at run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Held` |
| class-file version | 53.0 (Java 9) |
| bytes | 480 |
| SHA-256 | `d7bb7b0b78dd49b5d0c1b0ad6f86cf19354d5335c37425f430f60450c0dc6a2d` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_try_local.rs` (the member's text, the copy it declares and the cleanup it must not write) |

## What the defect is, and what each member is for

`try (r)` is Java 9 syntax: the header names an **expression**, not a variable, and the compiler
keeps that expression's value in a local of its own before the protected range — `aload r; astore
copy` — which the body then reads and which the compiler's own close closes. The `twr@1` rule asked
the instruction before the row's protected range for the resource's own initialisation, and
`initialises_resource` accepts a value a `new` or an invocation produced: `aload r; astore copy`
holds neither, so the row was no header at all and the member was read as the `try`/`catch` its own
table states. The copy became a statement before the `try`, the protected range was quoted, and the
compiler's cleanup became the body of a clause the source never wrote — while the `return` the normal
path ends in was left out with the block that holds it.

The copy is a value of its own kind, and the fix reads it as one: the store before the range is the
**whole** of a statement that is one `Load` of another local (`copied_local`), the slot it writes is
not the one it read, and the declaration the header writes is the one `resource_declaration` already
writes for such a range. One more link decides the reading: javac writes no `goto` after the close
when the body returns — the close is the last instruction of its own block, the continuation is the
very next instruction, and the close's block has it as its only successor. The handler is proved
exactly as for every other resource, and nothing of it is written. A store of a load whose proof does
**not** succeed keeps the `catch` its own table names: the attempt alone does not make a row a
header.

`use(Ljava/io/Reader;)I` states `[2, 7) → 17 Throwable` and the close's own row `[22, 26) → 29`, and
its bytes are `aload_0; astore_1` (the copy the header names), `aload_0; invokevirtual
java/io/Reader.read:()I; istore_2` (the body), `aload_1; ifnull 15; aload_1; invokevirtual
java/io/Reader.close:()V` (the normal path's close — **no** `goto` after it: the next instruction is
the continuation itself), `iload_2; ireturn` (the return the normal path ends in), and afterwards
the handler: `astore_2; aload_1; ifnull 35; aload_1; invokevirtual close:()V; goto 35`, then
`astore_3; aload_2; aload_3; invokevirtual Throwable.addSuppressed:(Ljava/lang/Throwable;)V;
aload_2; athrow`.

| member | bytecode | pre-fix text | post-fix text |
| --- | --- | --- | --- |
| `use(Ljava/io/Reader;)I` | `aload_0; astore_1; aload_0; invokevirtual read; istore_2; aload_1; ifnull 15; aload_1; invokevirtual close; iload_2; ireturn`, row `[2,7) → 17` | `java.io.Reader local1 = arg0;` before a `try` whose protected range is quoted, `} catch (java.lang.Throwable local2) { if (local1 != null) { try { local1.close(); } catch (java.lang.Throwable local3) { local2.addSuppressed(local3); } } }`, and no `return` at all — BCI 11, 15 and 35 are named as uncovered blocks | `try (java.io.Reader local1 = arg0) { int local2 = arg0.read(); }` and `return local2;` after the statement: one header, the body's own read inside it, and the compiler's close written by the statement the header states |
| `<init>()V` | `aload_0; invokespecial java/lang/Object.<init>:()V; return` | unchanged by the fix | unchanged by the fix: `super(); return;` — the constructor javac emits, so the class is one a class file can be read from |

The row's handler is the compiler's cleanup, not a clause: the text carries no `catch`, no `close(`
call and no `addSuppressed`, because the `try` the header declares performs them. What the header
declares is the **copy** — the type the frames give it (`java.io.Reader`), the name of the slot the
store fills (`local1`) and the value the store read (`arg0`) — and the copy is written nowhere else.

`tests/p3_try_local.rs` reads the same sample once more with **one byte changed**: the handler's own
null test at BCI 19 (`aload_1; ifnull 35`) becomes `aload_1; ifnonnull 35`, so the handler is no
longer the close the proof reads and `twr` fails on the row. There the copy decides what the row is
*then*: a store of a local's value is no initialisation this rule may refuse the member over, so the
failed attempt leaves the row where it was and the walk reads it as the `catch` its own table names —
never as a `try (…)` header and never as a refusal. A store that holds a `new` or an invocation keeps
the refusal beside it (`tests/p3_guard.rs` pins that side).
