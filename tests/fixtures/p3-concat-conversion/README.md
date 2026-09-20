# P3 stage: a presented concatenation keeps each `append`'s own conversion

`v8/ConcatConversion.class` is a real compiled sample: the sibling `ConcatConversion.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 ConcatConversion.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `ConcatConversion` |
| class-file version | 52.0 (Java 8) |
| bytes | 1461 |
| SHA-256 | `0e999005218c79f45502aa3f3e986ee0c97f20029fb9bc5b297fb3797803b932` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_concat_conversion.rs` (the exact text of every chain member, the parts' own anchors, and the two controls) and `tests/p3_execution_comparison.rs` (every member compiled under a declaration derived from the run's own facts and executed against the original) |

## What the defect is (T4)

A verified `StringBuilder`/`StringBuffer` chain is presented as one `+` expression (`concat@1`), and
`+` reproduces what `append` writes **only in a string context**: `String.valueOf(int)` is
`append(int)`, while `arg0 + arg1` on two `int`s is a numeric addition. The pre-fix construction
folded `+` from the first operand, so a chain that starts with parts needing conversion added them as
numbers and converted the *sum*:

```text
source: new StringBuilder().append(a).append(b).append("!").toString()
text:   arg0 + arg1 + "!"            (1, 2) -> "3!", the class -> "12!"
```

Both texts compile (`javac --release 8` accepts both under the member's own declaration), and both
runs reported `Java`/`Structured`/`contains_statements`/`complete` with no diagnostic: the structural
planes cannot see a value. The same fold also lost the group of a part whose value is **itself** an
addition, so `append(a + b).append("!")` and `append(a).append(b).append("!")` — two different
programs — came out as the **same text**:

| member | pre-fix text (measured through `jarde-cli recover`) | `(1, 2)` pre-fix | the class's own value |
| --- | --- | --- | --- |
| `twoIntsThenString(II)` | `return arg0 + arg1 + "!";` | `"3!"` | `"12!"` |
| `onePartIsASum(II)` | `return arg0 + arg1 + "!";` (the same text) | `"3!"` | `"3!"` |
| `marked()` | `return markA() + markB() + "!";` | `"3!"` | `"12!"` |
| `failing(I)` | `return markA() + div(100, arg0) + "!";` | a numeric sum of the two calls | `"150!"` for `2` |
| `booleanLiteral()` | `return 1 + "!";` | `"1!"` | `"true!"` |
| `booleanParameter(Z)` | `return arg0 + "!";` | already the same value | — |
| `nullPart()` | `return null + "!";` | already the same value | — |
| `objectPart(Object)` | `return arg0 + "!";` | already the same value | — |

The pre-fix texts and values were recorded by compiling both sides of the counterexample and running
them (see "The pre-fix counterexample" below); the fix is the change from that table's middle column
to the "What each member is for" table's texts.

## What each member is for, and what it is presented as now

| member | the chain's parts | text | the control it is |
| --- | --- | --- | --- |
| `twoIntsThenString(int, int)` | `int`, `int`, `String` | `return "" + arg0 + arg1 + "!";` | the reported shape: two parts that need conversion before the first `String` part |
| `onePartIsASum(int, int)` | `int` (a sum), `String` | `return "" + (arg0 + arg1) + "!";` | **one** part that is itself an addition: the sum is evaluated first and converted once |
| `numericLast(String, int)` | `String`, `int` | `return arg0 + arg1;` | the control: the part needing conversion comes last, so the text is unchanged (byte for byte) |
| `allStrings(String, String)` | `String`, `String` | `return arg0 + arg1;` | the control: nothing needs conversion, and no empty string is added |
| `booleanLiteral()` | `boolean` (a literal), `String` | `return "" + true + "!";` | the `boolean` descriptor: `append(true)`'s `iconst_1` is `true`, never the numeric `1` |
| `booleanParameter(boolean)` | `boolean`, `String` | `return "" + arg0 + "!";` | the same conversion for a value the descriptor proves `boolean` |
| `nullPart()` | `Object` (`null`), `String` | `return "" + null + "!";` | `append((Object) null)` is `String.valueOf(Object)`: `"null"` |
| `objectPart(Object)` | `Object`, `String` | `return "" + arg0 + "!";` | the same conversion for a value that may also be a `String` |
| `marked()` | two observable `int` calls, `String` | `return "" + markA() + markB() + "!";` | the evaluation order and count: two parts, each evaluated once, in the bytecode's order |
| `failing(int)` | an observable `int`, a throwing `int`, `String` | `return "" + markA() + div(100, arg0) + "!";` | the order of the evaluations *before* the one that throws |
| `markA()` | — | `ConcatConversion.calls = ConcatConversion.calls + 1; return 1;` | the first observable part |
| `markB()` | — | `ConcatConversion.calls = ConcatConversion.calls + 1; return 2;` | the second observable part |
| `div(int, int)` | — | `return arg0 / arg1;` | the part that throws when the divisor is zero |

The two controls are what keep the fix from being decoration: a chain that already starts in a string
context (a `String` first part, in either shape) gains **nothing**. `"x" + 5 + f()` (the
`crates/jarde-java/tests/p3_patterns.rs` sample) and the receiver-grouping fixture's all-`String`
chains are the same property in the in-memory fixtures.

The parts' anchors are the chain's own: each part carries its value's producers as direct anchors and
its `append` as a presented one, the `toString` the chain ends in anchors the expression, and every
other BCI the chain owns is a derived anchor of it. The empty string a chain starts from when its
first part is not a `String` has **no** instruction behind it — no `append` read it — so it is not a
part: it is written by the chain's own node, and it carries that node's anchors, none of which names
an instruction the chain did not execute. `tests/p3_concat_conversion.rs` asserts exactly that.

## The bytecode

```text
public ConcatConversion();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

public static java.lang.String twoIntsThenString(int, int);  // bb 00 07 59 b7 00 09 1a b6 00 0a 1b b6 00 0a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: iload_0
     8: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    11: iload_1
    12: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    15: ldc           #14                 // String !
    17: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    20: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    23: areturn

public static java.lang.String onePartIsASum(int, int);  // bb 00 07 59 b7 00 09 1a 1b 60 b6 00 0a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: iload_0
     8: iload_1
     9: iadd
    10: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    13: ldc           #14                 // String !
    15: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    18: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    21: areturn

public static java.lang.String numericLast(java.lang.String, int);  // bb 00 07 59 b7 00 09 2a b6 00 10 1b b6 00 0a b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: iload_1
    12: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    15: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    18: areturn

public static java.lang.String allStrings(java.lang.String, java.lang.String);  // bb 00 07 59 b7 00 09 2a b6 00 10 2b b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    11: aload_1
    12: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    15: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    18: areturn

public static java.lang.String booleanLiteral();  // bb 00 07 59 b7 00 09 04 b6 00 17 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: iconst_1
     8: invokevirtual #23                 // Method java/lang/StringBuilder.append:(Z)Ljava/lang/StringBuilder;
    11: ldc           #14                 // String !
    13: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn

public static java.lang.String booleanParameter(boolean);  // bb 00 07 59 b7 00 09 1a b6 00 17 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: iload_0
     8: invokevirtual #23                 // Method java/lang/StringBuilder.append:(Z)Ljava/lang/StringBuilder;
    11: ldc           #14                 // String !
    13: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn

public static java.lang.String nullPart();  // bb 00 07 59 b7 00 09 01 b6 00 1a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: aconst_null
     8: invokevirtual #26                 // Method java/lang/StringBuilder.append:(Ljava/lang/Object;)Ljava/lang/StringBuilder;
    11: ldc           #14                 // String !
    13: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn

public static java.lang.String objectPart(java.lang.Object);  // bb 00 07 59 b7 00 09 2a b6 00 1a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: aload_0
     8: invokevirtual #26                 // Method java/lang/StringBuilder.append:(Ljava/lang/Object;)Ljava/lang/StringBuilder;
    11: ldc           #14                 // String !
    13: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn

public static java.lang.String marked();  // bb 00 07 59 b7 00 09 b8 00 1d b6 00 0a b8 00 23 b6 00 0a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: invokestatic  #29                 // Method markA:()I
    10: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    13: invokestatic  #35                 // Method markB:()I
    16: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    19: ldc           #14                 // String !
    21: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    24: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    27: areturn

public static java.lang.String failing(int);  // bb 00 07 59 b7 00 09 b8 00 1d b6 00 0a 10 64 1a b8 00 26 b6 00 0a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: invokestatic  #29                 // Method markA:()I
    10: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    13: bipush        100
    15: iload_0
    16: invokestatic  #38                 // Method div:(II)I
    19: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    22: ldc           #14                 // String !
    24: invokevirtual #16                 // Method java/lang/StringBuilder.append:(Ljava/lang/String;)Ljava/lang/StringBuilder;
    27: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    30: areturn

public static int markA();  // b2 00 2a 04 60 b3 00 2a 04 ac
     0: getstatic     #42                 // Field calls:I
     3: iconst_1
     4: iadd
     5: putstatic     #42                 // Field calls:I
     8: iconst_1
     9: ireturn

public static int markB();  // b2 00 2a 04 60 b3 00 2a 05 ac
     0: getstatic     #42                 // Field calls:I
     3: iconst_1
     4: iadd
     5: putstatic     #42                 // Field calls:I
     8: iconst_2
     9: ireturn

public static int div(int, int);  // 1a 1b 6c ac
     0: iload_0
     1: iload_1
     2: idiv
     3: ireturn
```

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: these are the original class's own answers for the input set, and they are the values the
generated side must return.

```text
twoIntsThenString(1, 2)=12!
twoIntsThenString(7, 0)=70!
twoIntsThenString(0, -1)=0-1!
onePartIsASum(1, 2)=3!
onePartIsASum(7, 0)=7!
numericLast("a", 1)=a1
numericLast("", 0)=0
numericLast(null, 2)=null2
allStrings("a", "b")=ab
allStrings("", "x")=x
allStrings(null, "x")=nullx
booleanLiteral()=true!
booleanParameter(true)=true!
booleanParameter(false)=false!
nullPart()=null!
objectPart("o")=o!
objectPart(null)=null!
marked()=12! calls=0->2
failing(2)=150! calls=2->3
failing(0)=java.lang.ArithmeticException: / by zero calls=3->4
markA()=1 markB()=2 calls=6
div(7, 7)=1 div(-1, -1)=1
```

`marked()` answers `"12!"` only when its two parts are evaluated once each in the bytecode's order,
and its counter moves exactly twice; `failing(0)` shows `calls=3->4`, so the observable part *before*
the throwing one ran and no later part did.

## Reproducing

```text
cd tests/fixtures/p3-concat-conversion
mkdir -p v8
javac --release 8 -g:none -d v8 ConcatConversion.java
shasum -a 256 v8/ConcatConversion.class   # 0e999005218c79f45502aa3f3e986ee0c97f20029fb9bc5b297fb3797803b932
```

`Baseline.java` is not compiled into this directory: the comparison test compiles it in a temporary
directory with the sample on its classpath, so no driver class is ever committed beside the sample.
