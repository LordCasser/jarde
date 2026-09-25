# P3 fixture: what a position does with a widening, and what the `pop` a compiler writes means

`v8/Meet.class` is a real compiled sample: the sibling `Meet.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Meet.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时…` / `目标值 8 已过时` /
`要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them), the
`注: Meet.java使用了未经检查或不安全的操作.` note for the raw `List` in `unchecked`, and writes the
class anyway, exit code 0. The compiler is a generation-only input: it is not needed at run time, so
the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `Meet` |
| class-file version | 52.0 (Java 8) |
| bytes | 940 |
| SHA-256 | `0e3b9f78d79e49e345ccb163a01eeaa0d21f9ed7ca0b340c24d7901cd4b1c04c` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal (`arg0`, `local1`) |
| read by | `tests/p3_meeting.rs` (every member's text, and the two members that are still refused) |

## What each member is for

The sample is one member per spelling question of the 2c.29/2c.30/2c.31 batch. A `char`, a `byte` and
a `short` share one slot shape with an `int`, so the frames cannot say which of the four a position
holds and a compiler writes **no instruction** when it widens one of them; where such a conversion
exists, the table below states what the text does with it.

| member | bytecode | text before the batch | text after it |
| --- | --- | --- | --- |
| `at(Ljava/lang/String;I)I` | `aload_0; iload_1; invokevirtual java/lang/String.charAt:(I)C; ireturn` | `return (int) arg0.charAt(arg1);` | `return arg0.charAt(arg1);` — the `return` widens the `char` itself (P3 2c.29) |
| `viaStore(C)I` | `iload_0; istore_1; iload_1; ireturn` | `int local1 = (int) arg0;` | `int local1 = arg0;` — the declaration's type widens it (P3 2c.29) |
| `fieldArg(Ljava/lang/String;)I` | `aload_0; iconst_0; invokevirtual charAt:(I)C; invokestatic pass:(I)I; ireturn` | `return pass((int) arg0.charAt(0));` | `return pass(arg0.charAt(0));` — the callee's `int` parameter widens it (P3 2c.29) |
| `pass(I)I` | `iload_0; ireturn` | `return arg0;` | unchanged — the callee `fieldArg` reaches, and the control that must not gain a conversion |
| `viaStoreLong(I)J` | `iload_0; i2l; lstore_1; lload_1; lreturn` | refused: BCI 1 is `Other` and BCI 2 comes from it | unchanged — the widening here has a **real** instruction, so it is P3 2c.8's, which has not landed |
| `trunc(I)B` | `iload_0; i2b; istore_1; iload_1; ireturn` | refused: BCI 1 is `Other`, BCI 3 is an `int` in a `byte` position | unchanged — the narrowing `i2b` is no conversion this layer writes |
| `grade(I)C` | `lookupswitch {80→39, 90→36, 95→36, default→42}` and `bipush 65/66/67; ireturn` | `return 65;` / `return 66;` / `return 67;` | `return 'A';` / `return 'B';` / `return 'C';` — a `char` return spells the code unit (P3 2c.30) |
| `stat()I` | `bipush 7; ireturn` | `return 7;` | unchanged — the control: an `int` position keeps the number |
| `viaRef(LMeet;)I` | `aload_0; pop; invokestatic Meet.stat:()I; ireturn` | `pop` quoted at BCI 1, body `return stat();` | `return arg0.stat();` — the popped evaluation is the call's qualifier (P3 2c.31a) |
| `unchecked(Ljava/util/List;)V` | `aload_0; ldc "x"; invokeinterface java/util/List.add:(Ljava/lang/Object;)Z; pop; return` | `arg0.add("x");` plus the `pop` quoted at BCI 8 | `arg0.add("x");` and no quote — the call's statement is the discard (P3 2c.31b) |
| `pop2Control()V` | `invokestatic java/lang/System.nanoTime:()J; pop2; return` | `java.lang.System.nanoTime();` plus the `pop2` quoted at BCI 3 | unchanged — no shape of the rule claims a `pop2` |
| `<init>()V` | `aload_0; invokespecial java/lang/Object.<init>:()V; return` | `super();` | unchanged: the constructor javac emits, so the class is one a class file can be read from |

## The rules the members pin

* **P3 2c.29 — a widening a position performs is not written.** `at`, `viaStore` and `fieldArg` are
  one conversion each: `char` → `int`, with no instruction in the body. Java performs it at the
  assignment, the invocation argument and the `return`, so the value's own text is the same program
  and `(int) arg0` is a cast the source does not have. The two controls keep the other direction
  honest: `trunc` (a `byte` return whose value the frames present as `int`) and `viaStoreLong` (a
  `long` local whose value an `i2l` really produced) are refused exactly as they were.
* **P3 2c.30 — a `char` position spells the code unit.** `grade` returns `'A'`/`'B'`/`'C'` from the
  `bipush 65/66/67` javac writes for a `char` return: the constant is in `char`'s range, the
  descriptor says the position is a `char`, and the character that code unit stands for is what the
  text writes (spelled by the emitter's own escape table, the one a `char` `switch` key is written
  with). `stat` is the control: an `int` member returning `7` keeps the number, and only an
  out-of-range or non-constant value keeps the refusal.
* **P3 2c.31 — the two `pop`s a compiler writes are the text beside them.** javac evaluates the
  qualifier of a static call made on an instance reference and throws it away (`viaRef`: `aload_0;
  pop`), and it discards a call result nobody reads with a `pop` (`unchecked`). The first `pop` is
  the qualifier the call's text has to carry — writing the bare `stat()` would drop the evaluation
  the source performed — and the second is the call's own statement. `pop2Control` is the control:
  a two-slot result is discarded with a `pop2`, no shape of this rule claims one, and it keeps the
  quote it always had.
