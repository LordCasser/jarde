# P3 stage A fixture: an integer binary comparison keeps each operand's own spelling

`v8/IntComparisons.class` is a real compiled sample: the sibling `IntComparisons.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 IntComparisons.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `IntComparisons` |
| class-file version | 52.0 (Java 8) |
| bytes | 647 |
| SHA-256 | `8b535ab811d969249640de82c40d5884b79a712734b883fd868f45ea53583ff6` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_boolean_contexts.rs` (exact text, the recorded boundary and the predecessor change's controls) and `tests/p3_execution_comparison.rs` (nine members compiled under a declaration derived from the run's own facts and executed against the original, constants on the left and on the right, equality and ordering) |

## What the defect is, and what each member is for

The layer proves a value is a boolean from the class file and this body (P3-R5's evidence list), and
one item of that list is the `0`/`1` **literal** — a boolean only where the **position** already
requires one. `condition` read that item before it had decided the test's shape, so the left operand
of an `if_icmp*` comparison was spelled as a boolean although the comparison reads two `int`s and
only its *result* is a boolean. Three shapes came out as text `javac --release 8` refuses when the
body is wrapped in the member's own declaration:

| member | source shape | pre-fix text | `javac --release 8` |
| --- | --- | --- | --- |
| `oneFirst(I)I` | `if (1 == n) { return 3; } return 4;` | `if (true == arg0) {` | `error: incomparable types: boolean and int` |
| `zeroFirst(I)I` | `if (0 < n) { return 3; } return 4;` | `if (false < arg0) {` | `error: bad operand types for binary operator '<'` (`first type: boolean`, `second type: int`) |
| `oneLess(I)I` | `if (1 < n) { return 3; } return 4;` | `if (true < arg0) {` | `error: bad operand types for binary operator '<'` (`first type: boolean`, `second type: int`) |

Each of those runs reported `Produced`/`Java`/`Structured`/`ContainsStatements` with no diagnostic:
the artifact was structurally whole and its comparison operands were spelled wrongly, which is why
the acceptance is the text plus a compiler.

**This is a regression this project introduced.** The reviewer built the pre-fix CLI from
`git archive fa6dc6e` (the commit before the archived `type-boolean-contexts` change) and measured
that these three shapes kept their integer text there; `5a8c36a` turned them into the boolean text
above. It is not a newly found defect, and the fix that closes it is a correction of that change.

The members are the five comparison shapes, the variable-operand control, and the previous change's
boolean shapes so that the same bytes carry both the regression and its controls:

| member | descriptor | what it pins |
| --- | --- | --- |
| `oneFirst` | `(I)I` | the defect: the constant on the **left** of `if_icmpne`, equality |
| `zeroFirst` | `(I)I` | the defect: the constant on the left, ordering (`if_icmpge` spells `0 < arg0`) |
| `oneLess` | `(I)I` | the defect: `1 < arg0`, the same opcode as `zeroFirst` with the other literal |
| `oneLast` | `(I)I` | the same rule with the constant on the **right** (`arg0 == 1`) |
| `zeroLast` | `(I)I` | the constant on the right folded into a zero test by the compiler (`iload_0; ifle`): an unproven operand keeps the integer comparison `arg0 > 0` |
| `nonzero` | `(I)I` | the int control with two variable operands (`arg0 != 0`) — the shape that must not be read as a boolean, and not the only int evidence this sample carries |
| `isZero` | `(I)Z` | the predecessor change's `Z` return of a literal: `return true;`/`return false;` |
| `count` | `(Z)I` | the predecessor change's truth test: a zero test on a proven `boolean` parameter is `if (arg0)`, not `if (arg0 != 0)` |
| `throughLocal` | `(Z)Z` | the predecessor change's local declaration: `boolean local1 = arg0; return local1;` |

```text
public IntComparisons();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

public static int oneFirst(int);  // 04 1a a0 00 05 06 ac 07 ac
     0: iconst_1
     1: iload_0
     2: if_icmpne     7
     5: iconst_3
     6: ireturn
     7: iconst_4
     8: ireturn

public static int zeroFirst(int);  // 03 1a a2 00 05 06 ac 07 ac
     0: iconst_0
     1: iload_0
     2: if_icmpge     7
     5: iconst_3
     6: ireturn
     7: iconst_4
     8: ireturn

public static int oneLess(int);  // 04 1a a2 00 05 06 ac 07 ac
     0: iconst_1
     1: iload_0
     2: if_icmpge     7
     5: iconst_3
     6: ireturn
     7: iconst_4
     8: ireturn

public static int oneLast(int);  // 1a 04 a0 00 05 06 ac 07 ac
     0: iload_0
     1: iconst_1
     2: if_icmpne     7
     5: iconst_3
     6: ireturn
     7: iconst_4
     8: ireturn

public static int zeroLast(int);  // 1a 9e 00 05 06 ac 07 ac
     0: iload_0
     1: ifle          6
     4: iconst_3
     5: ireturn
     6: iconst_4
     7: ireturn

public static int nonzero(int);  // 1a 99 00 05 06 ac 07 ac
     0: iload_0
     1: ifeq          6
     4: iconst_3
     5: ireturn
     6: iconst_4
     7: ireturn

public static boolean isZero(int);  // 1a 9a 00 05 04 ac 03 ac
     0: iload_0
     1: ifne          6
     4: iconst_1
     5: ireturn
     6: iconst_0
     7: ireturn

public static int count(boolean);  // 1a 99 00 05 04 ac 03 ac
     0: iload_0
     1: ifeq          6
     4: iconst_1
     5: ireturn
     6: iconst_0
     7: ireturn

public static boolean throughLocal(boolean);  // 1a 3c 1b ac
     0: iload_0
     1: istore_1
     2: iload_1
     3: ireturn
```

`javac` folded `n > 0` (`zeroLast`) and `n != 0` (`nonzero`) into the compact zero tests `ifle` and
`ifeq`, so those two members reach the condition in its `Test::Zero` form with an operand no
evidence proves boolean — and they must keep the integer comparison. The three defect shapes and
`n == 1` keep their literal on the stack, which is the `Test::Pair` form this fixture is about.

## What each member recovered, before and after

Pre-fix texts are the measured output of the same sample with the fix removed (the mutation the
change records: `boolean_value`, the literal included, read and applied before the `Test` shape is
known); post-fix texts are the committed behaviour.

| member | before | after |
| --- | --- | --- |
| `oneFirst(I)I` | `if (true == arg0)` | `if (1 == arg0)` |
| `zeroFirst(I)I` | `if (false < arg0)` | `if (0 < arg0)` |
| `oneLess(I)I` | `if (true < arg0)` | `if (1 < arg0)` |
| `oneLast(I)I` | `if (arg0 == 1)` | unchanged |
| `zeroLast(I)I` | `if (arg0 > 0)` | unchanged |
| `nonzero(I)I` | `if (arg0 != 0)` | unchanged |
| `isZero(I)Z` | `if (arg0 == 0) { return true; } else { return false; }` | unchanged |
| `count(Z)I` | `if (arg0) { return 1; } else { return 0; }` | unchanged |
| `throughLocal(Z)Z` | `boolean local1 = arg0; return local1;` | unchanged |

## The boundary this fixture records and does not close

A `Test::Pair` comparison that mixes a proven boolean with an `int` literal — `invokestatic flag()Z;
iconst_1; if_icmpne`, the shape a compiler folds away, so it is hand-built in
`tests/p3_boolean_contexts.rs` as the class `Boundary` rather than committed here — is written
`if (flag() == 1) { return 1; } else { return 0; }`. Wrapping that body in the member's own
declaration and compiling it with

```text
javac --release 8 -g:none -J-Duser.language=en -J-Duser.country=US Boundary.java
```

is refused:

```text
Boundary.java:3: error: incomparable types: boolean and int
    public static int probe() { if (flag() == 1) { return 1; } else { return 0; } }
                                           ^
1 error
```

That was already the text before this fix and it stays the text after it: the rule "each operand of a
pair comparison keeps its own evidence" does not close this shape, this change does not claim it
closed, and no general proof is invented for it. If a later change makes it worse (for example by
producing new refused text), that is a regression of *this* rule.

## The baseline driver

`Baseline.java` is the committed original's own driver: it is compiled with this class on the
classpath and prints one line per observation of the input set the executed comparison calls every
member with (`0`, `1`, `2`, `-1` for the comparisons; `isZero(0)`, `isZero(1)`, `isZero(7)`,
`isZero(-1)`; `count(true)`, `count(false)`; `throughLocal(true)`, `throughLocal(false)`). Its 32
lines are asserted exactly by `tests/p3_execution_comparison.rs`, so the positive side of the
comparison is the original's executed behaviour and not a recomputation of the generated text.
