# P3 stage A fixture: a nested expression evaluated after a write (P3-R8)

`v8/NestedEval.class` is a real compiled sample: the sibling `NestedEval.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 NestedEval.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them, the
last one English as `source value 8 is obsolete` / `target value 8 is obsolete`) and writes the class
anyway, exit code 0. The compiler is a generation-only input: it is not needed at run time, so the
sample is committed as bytes.

| property | value |
| --- | --- |
| class | `NestedEval` |
| class-file version | 52.0 (Java 8) |
| bytes | 361 |
| SHA-256 | `141c3dcb3990605466dd54eb1a9bcc1942d0907e4e32d8d0587eb443f0c5c2e9` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_eval_context.rs` (P3-R8 regression, through `Engine::recover_method`) and `tests/p3_execution_comparison.rs` (the refused bodies and their two controls) |

## What each member is for, and the bytes that make it

```text
static int tick(int);               // b2 00 07 04 60 b3 00 07 1a ac
     0: getstatic     #7            // Field calls:I
     3: iconst_1
     4: iadd
     5: putstatic     #7            // Field calls:I
     8: iload_0
     9: ireturn

public static int nestedLocal(int); // 1a 04 60 84 00 01 1a 60 ac
     0: iload_0
     1: iconst_1
     2: iadd
     3: iinc          0, 1
     6: iload_0
     7: iadd
     8: ireturn

public static int nestedPlain(int); // 1a 04 60 1a 05 60 60 ac
     0: iload_0
     1: iconst_1
     2: iadd
     3: iload_0
     4: iconst_2
     5: iadd
     6: iadd
     7: ireturn

public static int nestedCall(int);  // 1a b8 00 0d 84 00 01 1a 60 ac
     0: iload_0
     1: invokestatic  #13           // Method tick:(I)I
     4: iinc          0, 1
     7: iload_0
     8: iadd
     9: ireturn
```

`nestedLocal` is P3-R8: the outer `iadd` at BCI 7 reads the value BCI **0**'s load produced, and the
`iinc` at BCI 3 writes slot 0 before BCI 8 evaluates the outer sum. The check "does the slot's name
still denote the value this load read" is therefore a question about BCI 8 — where the generated text
is evaluated — and asking it at the inner `iadd`'s own BCI 2 passes it. Writing `arg0 + 1 + arg0`
after `arg0 = arg0 + 1;` answers 17 where `nestedLocal(7)` answers 16, so the honest answer is a
refusal that names the consumer (BCI 8) and the read (BCI 0).

`nestedCall` is the same rule at a call argument: `tick(x) + ++x` loads `x` at BCI 0 as the call's
argument, and the call's value is consumed by the `iadd` at BCI 8 after the increment wrote slot 0.
Writing `tick(arg0) + arg0` after the increment calls `tick` on the incremented value: the refusal
names the deferred call (BCI 1) and the read (BCI 0) instead.

`nestedPlain` is the **control**: `(x + 1) + (x + 2)` is the same left-nested `+` shape with no write
between the two loads and the outer sum, so both loads still hold what they read where the text is
evaluated and the body must stay `Java`/`Structured`.

`tick` is the counter the executed controls move; it is presented whole and executed by the
comparison.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: `nestedLocal(7)` and `nestedCall(3)` are the original's own answers — the numbers the
refusals are about — and `tick calls` states that the executed control called `tick` exactly once.

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the
run that introduced this sample:

| member | run | wrapper | result |
| --- | --- | --- | --- |
| `tick(I)I` | Java/Structured | compiles | executed: traces identical |
| `nestedLocal(I)I` | Mixed/Fallback | javac refuses: `GennestedLocal.java:9: error: missing return statement` | boundary: 2 quoted BCI(s), refused regions `[]` |
| `nestedPlain(I)I` | Java/Structured | compiles | executed: traces identical |
| `nestedCall(I)I` | Mixed/Fallback | javac refuses: `GennestedCall.java:9: error: missing return statement` | boundary: 2 quoted BCI(s), refused regions `[]` |

- the member declarations the comparison derived: `static int tick(int arg0)`,
  `public static int nestedLocal(int arg0)`, `public static int nestedPlain(int arg0)`,
  `public static int nestedCall(int arg0)`
- trace: 6 line(s), identical on both sides
- the committed baseline driver printed:

```text
nestedLocal(7)=16
nestedCall(3)=7
tick calls=1
```

## Reproducing

```text
cd tests/fixtures/p3-nested-eval
mkdir -p v8
javac --release 8 -g:none -d v8 NestedEval.java
shasum -a 256 v8/NestedEval.class   # 141c3dcb3990605466dd54eb1a9bcc1942d0907e4e32d8d0587eb443f0c5c2e9
```
