# P3 stage A fixture: the grouping a printed subexpression owes the **position** it is written in

`v8/ReceiverGrouping.class` is a real compiled sample: the sibling `ReceiverGrouping.java` compiled by **javac 23.0.1**
(OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 ReceiverGrouping.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `ReceiverGrouping` |
| class-file version | 52.0 (Java 8) |
| bytes | 1038 |
| SHA-256 | `8e030b5338a357a0c2afd77b7c7b30bf7dd5a8a2c6d2a17f641621cc30cc8c00` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_eval_context.rs` (exact text, through `Engine::recover_method`) and `tests/p3_execution_comparison.rs` (all eight members, executed against the original) |

## What the defect is, and what each member is for

The expression tree is the layer's proof; the **text is a different artifact**, and Java groups it
by its own rules (JLS 15). The archived arithmetic fix parenthesised a binary expression's
**operands**; but grouping is a property of the **position** a subexpression is written in, not only
of its parent operator, and the receiver position had none: `crates/jarde-java/src/emit.rs`'s call
branch printed the receiver's text and then the `.`, so

```text
source: (a + b).substring(1)          tree: Call { receiver: Add(a, b), name: substring }
text:   arg0 + arg1.substring(1)      reads: arg0 + (arg1.substring(1))
```

The report of that run was `Java`/`Structured`/`contains_statements`/`complete` with no diagnostic —
the structural planes are not value evidence — and with `("a", "bc")` the class answers `"bc"` while
that text answers `"ac"`. Every member of this sample is that defect in one shape, or the control
that shows the fix is grouping and not decoration.

```text
public ReceiverGrouping();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                 // Method java/lang/Object."<init>":()V
     4: return

public static java.lang.String call(java.lang.String, java.lang.String);  // bb 00 07 59 b7 00 09 2a b6 00 0a 2b b6 00 0a b6 00 0e 04 b6 00 12 b0
     0: new           #7                 // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                 // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: aload_1
    12: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    15: invokevirtual #14                // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    18: iconst_1
    19: invokevirtual #18                // Method java/lang/String.substring:(I)Ljava/lang/String;
    22: areturn

public static int length(java.lang.String, java.lang.String);  // bb 00 07 59 b7 00 09 2a b6 00 0a 2b b6 00 0a b6 00 0e b6 00 18 ac
     0: new           #7                 // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                 // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: aload_1
    12: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    15: invokevirtual #14                // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    18: invokevirtual #24                // Method java/lang/String.length:()I
    21: ireturn

public static java.lang.String nested(java.lang.String, java.lang.String, java.lang.String);  // bb 00 07 59 b7 00 09 2a b6 00 0a 2b b6 00 0a 2c b6 00 0a b6 00 0e 04 b6 00 12 b0
     0: new           #7                 // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                 // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: aload_1
    12: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    15: aload_2
    16: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    19: invokevirtual #14                // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    22: iconst_1
    23: invokevirtual #18                // Method java/lang/String.substring:(I)Ljava/lang/String;
    26: areturn

public static java.lang.String plain(java.lang.String);  // 2a b6 00 1c b0
     0: aload_0
     1: invokevirtual #28                // Method java/lang/String.trim:()Ljava/lang/String;
     4: areturn

public static int chained(java.lang.String);  // 2a b6 00 1c b6 00 18 ac
     0: aload_0
     1: invokevirtual #28                // Method java/lang/String.trim:()Ljava/lang/String;
     4: invokevirtual #24                // Method java/lang/String.length:()I
     7: ireturn

public static java.lang.String same(java.lang.String, java.lang.String, java.lang.String);  // bb 00 07 59 b7 00 09 2a b6 00 0a 2b b6 00 0a 2c b6 00 0a b6 00 0e b0
     0: new           #7                 // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                 // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: aload_1
    12: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    15: aload_2
    16: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    19: invokevirtual #14                // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    22: areturn

public static java.lang.String argument(java.lang.String, java.lang.String);  // bb 00 07 59 b7 00 09 2a b6 00 0a 2b b6 00 0a b6 00 0e b8 00 1f b0
     0: new           #7                 // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                 // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: aload_1
    12: invokevirtual #10                // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    15: invokevirtual #14                // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    18: invokestatic  #31                // Method wrap:(Ljava/lang/String;)Ljava/lang/String;
    21: areturn

public static java.lang.String wrap(java.lang.String);  // 2a b0
     0: aload_0
     1: areturn
```

`call` is the review's reported shape; the receiver of `substring` is the concatenation the chain
at BCI 0–15 builds. `length` is the review's second shape: the same receiver, consumed by
`String.length()`, whose ungrouped text `arg0 + arg1.length()` is not even a program under the
member's `int` return (`String` cannot convert to `int`). `nested` is a three-operand chain in
receiver position, so the parentheses have to hold the whole chain (`(a + b) + c`), not just its
first pair. The remaining four members are the controls: the positions that already state their
trees must gain no parentheses.

| member | tree | text that lost the grouping |
| --- | --- | --- |
| `call(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | `(arg0 + arg1).substring(1)` | `arg0 + arg1.substring(1)` — `arg0 + (arg1.substring(1))` |
| `length(Ljava/lang/String;Ljava/lang/String;)I` | `(arg0 + arg1).length()` | `arg0 + arg1.length()` — and it does not compile |
| `nested(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | `(arg0 + arg1 + arg2).substring(1)` | `arg0 + arg1 + arg2.substring(1)` |
| `plain(Ljava/lang/String;)Ljava/lang/String;` | `arg0.trim()` | control: a name receiver is a primary |
| `chained(Ljava/lang/String;)I` | `arg0.trim().length()` | control: a call receiver is a primary |
| `same(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | `arg0 + arg1 + arg2` | control: left associativity already states the left-nested chain |
| `argument(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | `wrap(arg0 + arg1)` | control: a call argument is delimited by `,` and `)` |
| `wrap(Ljava/lang/String;)Ljava/lang/String;` | `arg0` | the helper `argument` names |

Both defect shapes are executed, not only compiled: the comparison compiles the recovered text
under a declaration it derives from the run's own facts and runs it beside the original.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: these are the original class's own answers for the input set, and they are the values the
generated side must return. The divergence case comes first: the class answers `"bc"` for
`call("a", "bc")` and the text that dropped the receiver's group answered `"ac"`.

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the
run that introduced this sample:

| member | run | content | wrapper | result |
| --- | --- | --- | --- | --- |
| `call(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `length(Ljava/lang/String;Ljava/lang/String;)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `nested(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `plain(Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `chained(Ljava/lang/String;)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `same(Ljava/lang/String;Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `argument(Ljava/lang/String;Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `wrap(Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |

- the member declarations the comparison derived: `public static java.lang.String call(java.lang.String arg0, java.lang.String arg1)`,
  `public static int length(java.lang.String arg0, java.lang.String arg1)`,
  `public static java.lang.String nested(java.lang.String arg0, java.lang.String arg1, java.lang.String arg2)`,
  `public static java.lang.String plain(java.lang.String arg0)`,
  `public static int chained(java.lang.String arg0)`,
  `public static java.lang.String same(java.lang.String arg0, java.lang.String arg1, java.lang.String arg2)`,
  `public static java.lang.String argument(java.lang.String arg0, java.lang.String arg1)`,
  `public static java.lang.String wrap(java.lang.String arg0)`
- trace: 15 line(s), identical on both sides. The calls are the sample's own, stated in
  `tests/p3_execution_comparison.rs` (`inputs`): `call("a", "bc")`, `call("r", "r")`,
  `call(null, "bc")`, `length("a", "bc")`, `length("", "xy")`,
  `nested("a", "bc", "def")`, `nested("", "", "x")`, `plain("  a ")`, `chained("  a ")`,
  `same("a", "b", "c")`, `argument("a", "bc")` — the default `String` values would have been
  `"r"`/`null`, and `call("r", "r")` is exactly the input where the two texts agree, which is why
  the finding's own pair is stated rather than assumed
- the committed baseline driver printed:

```text
call("a", "bc")=bc
call("r", "r")=r
call(null, "bc")=ullbc
length("a", "bc")=3
nested("a", "bc", "def")=bcdef
plain("  a ")=[a]
chained("  a ")=1
same("a", "b", "c")=abc
argument("a", "bc")=abc
```

The executable state of the **defect** is recorded by the mutation run: with the receiver grouping
removed again, the recovered text is `return arg0 + arg1.substring(1);`, and compiling it beside the
original answers `value call(...) ["a", "bc"] -> "bc" | generated -> "ac"` (and
`[null, "bc"] -> "ullbc" | generated -> "nullc"`, while `["r", "r"]` agrees). The comparison itself
goes red earlier, on `length`: its ungrouped text is refused by javac under the member's `int`
return.

## Reproducing

```text
cd tests/fixtures/p3-receiver-grouping
mkdir -p v8
javac --release 8 -g:none -d v8 ReceiverGrouping.java
shasum -a 256 v8/ReceiverGrouping.class   # 8e030b5338a357a0c2afd77b7c7b30bf7dd5a8a2c6d2a17f641621cc30cc8c00
```

`Baseline.java` is not compiled into this directory: the comparison test compiles it in a temporary
directory with the sample on its classpath, so no driver class is ever committed beside the sample.
