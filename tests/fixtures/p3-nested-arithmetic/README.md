# P3 stage A fixture: the grouping a printed arithmetic tree owes the text

`v8/ModLike.class` is a real compiled sample: the sibling `ModLike.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 ModLike.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `ModLike` |
| class-file version | 52.0 (Java 8) |
| bytes | 568 |
| SHA-256 | `a5827ef4865e7d33aff6ea0c34c46194a69f2436e011f5ad5c848853d2891158` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_eval_context.rs` (the grouping regression, through `Engine::recover_method`) and `tests/p3_execution_comparison.rs` (all eight members, executed against the original) |

## What the defect is, and what each member is for

The layer's expression tree is the proof; the **text is a different artifact**, and Java groups it
by its own precedence and associativity (JLS 15.17, 15.18, 15.20). Printing a binary expression by
concatenating its operands' text therefore published another program wherever an operand was itself
a binary expression of looser — or, on the right, equal — binding: `i * (2 - d * i)` reached the
buffer as `i * 2 - d * i`, which is `(i * 2) - (d * i)`. Every member of this sample is that defect
in one shape, or the control that shows the fix is grouping and not decoration.

```text
public ModLike();                        // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                 // Method java/lang/Object."<init>":()V
     4: return

public static int inverse32(int);        // 1a 3c 1b 05 1a 1b 68 64 68 3c 1b 05 1a 1b 68 64 68 3c 1b 05 1a 1b 68 64 68 3c 1b 05 1a 1b 68 64 68 3c 1b ac
     0: iload_0
     1: istore_1
     2: iload_1
     3: iconst_2
     4: iload_0
     5: iload_1
     6: imul
     7: isub
     8: imul
     9: istore_1
    10: iload_1
    11: iconst_2
    12: iload_0
    13: iload_1
    14: imul
    15: isub
    16: imul
    17: istore_1
    18: iload_1
    19: iconst_2
    20: iload_0
    21: iload_1
    22: imul
    23: isub
    24: imul
    25: istore_1
    26: iload_1
    27: iconst_2
    28: iload_0
    29: iload_1
    30: imul
    31: isub
    32: imul
    33: istore_1
    34: iload_1
    35: ireturn

public static int scaledDifference(int, int);  // 1a 05 1b 1a 68 64 68 ac
     0: iload_0
     1: iconst_2
     2: iload_1
     3: iload_0
     4: imul
     5: isub
     6: imul
     7: ireturn

public static int nestedDifference(int, int, int);  // 1a 1b 1c 64 64 ac
     0: iload_0
     1: iload_1
     2: iload_2
     3: isub
     4: isub
     5: ireturn

public static int nestedQuotient(int, int, int);  // 1a 1b 1c 68 6c ac
     0: iload_0
     1: iload_1
     2: iload_2
     3: imul
     4: idiv
     5: ireturn

public static int differenceOfSum(int, int);  // 1a 1b 04 60 05 68 64 ac
     0: iload_0
     1: iload_1
     2: iconst_1
     3: iadd
     4: iconst_2
     5: imul
     6: isub
     7: ireturn

public static int productOfSum(int, int, int);  // 1a 1b 60 1c 68 ac
     0: iload_0
     1: iload_1
     2: iadd
     3: iload_2
     4: imul
     5: ireturn

public static int sumOfProducts(int, int, int);  // 1a 1b 1c 68 60 ac
     0: iload_0
     1: iload_1
     2: iload_2
     3: imul
     4: iadd
     5: ireturn

public static int leftNestedSum(int, int, int);  // 1a 1b 60 1c 60 ac
     0: iload_0
     1: iload_1
     2: iadd
     3: iload_2
     4: iadd
     5: ireturn
```

`inverse32` is the shape the benchmark campaign measured on
`org.bouncycastle.math.raw.Mod.inverse32(I)I`: `i = i * (2 - d * i)` four times, each block
`iload_1; iconst_2; iload_0; iload_1; imul; isub; imul; istore_1`. The tree is
`local1 * (2 - arg0 * local1)`; the text that concatenated the operands was
`local1 = local1 * 2 - arg0 * local1;`, which Java parses as `(local1 * 2) - (arg0 * local1)`. The
report of that run was `quality=Structured`, `content=contains_statements`, `execution=complete`
and carried no diagnostic — the structural planes are not value evidence — and the compiled text
answered `inverse32(-1) = -81` where the class answers `-1`.

The four shapes the delta names are the four members after it, each with one operator combination:

| member | tree | text that lost the grouping |
| --- | --- | --- |
| `scaledDifference(II)I` | `arg0 * (2 - arg1 * arg0)` | `arg0 * 2 - arg1 * arg0` |
| `nestedDifference(III)I` | `arg0 - (arg1 - arg2)` | `arg0 - arg1 - arg2` |
| `nestedQuotient(III)I` | `arg0 / (arg1 * arg2)` | `arg0 / arg1 * arg2` |
| `differenceOfSum(II)I` | `arg0 - (arg1 + 1) * 2` | `arg0 - arg1 + 1 * 2` |

`productOfSum(III)I` is the same defect on the **left** operand: `(a + b) * c` was published as
`arg0 + arg1 * arg2`, which Java reads as `arg0 + (arg1 * arg2)`.

`sumOfProducts(III)I` (`a + b * c`) and `leftNestedSum(III)I` (`(a + b) + c`) are the **controls**
in the other direction: Java's default precedence states the first tree, left associativity states
the second, and neither text may gain parentheses. Together with the two shapes above they state the
rule the printer applies — an operand keeps its own group unless it binds tighter than its parent,
and an equal-precedence operand on the right never does — rather than one fixed string.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: these are the original class's own answers for the input set, and they are the values the
generated side must return. The divergence case comes first: the class answers `-1` for
`inverse32(-1)` and the text that lost its group answered `-81`.

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the
run that introduced this sample:

| member | run | content | wrapper | result |
| --- | --- | --- | --- | --- |
| `inverse32(I)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `scaledDifference(II)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `nestedDifference(III)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `nestedQuotient(III)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `differenceOfSum(II)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `productOfSum(III)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `sumOfProducts(III)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `leftNestedSum(III)I` | Java/Structured | contains_statements | compiles | executed: traces identical |

- the member declarations the comparison derived: `public static int inverse32(int arg0)`,
  `public static int scaledDifference(int arg0, int arg1)`,
  `public static int nestedDifference(int arg0, int arg1, int arg2)`,
  `public static int nestedQuotient(int arg0, int arg1, int arg2)`,
  `public static int differenceOfSum(int arg0, int arg1)`,
  `public static int productOfSum(int arg0, int arg1, int arg2)`,
  `public static int sumOfProducts(int arg0, int arg1, int arg2)`,
  `public static int leftNestedSum(int arg0, int arg1, int arg2)`
- trace: 24 line(s), identical on both sides — every member is called with the comparison's own
  input set (`[7]`, `[0]`, `[-1]` per arity, so `inverse32(-1)` is the third row) and the two sides
  return the same value or throw the same exception for every row
- the committed baseline driver printed:

```text
inverse32(-1)=-1
inverse32(7)=-1227133513
inverse32(0)=0
scaledDifference(3, 5)=-39
nestedDifference(10, 4, 7)=13
nestedQuotient(20, 3, 4)=1
differenceOfSum(10, 3)=2
productOfSum(2, 3, 4)=20
sumOfProducts(2, 3, 4)=14
leftNestedSum(2, 3, 4)=9
```

The executable state of the **defect** is recorded by the comparison's own mutation run: with the
printer's grouping removed again, the same run fails on the first row
(`value inverse32(I)I [7]` → original `-1227133513`, generated `4375`), and the text it compiles
answers `-81` for `inverse32(-1)` where the class answers `-1`.

## Reproducing

```text
cd tests/fixtures/p3-nested-arithmetic
mkdir -p v8
javac --release 8 -g:none -d v8 ModLike.java
shasum -a 256 v8/ModLike.class   # a5827ef4865e7d33aff6ea0c34c46194a69f2436e011f5ad5c848853d2891158
```

`Baseline.java` is not compiled into this directory: the comparison test compiles it in a temporary
directory with the sample on its classpath, so no driver class is ever committed beside the sample.
