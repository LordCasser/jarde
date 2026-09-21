# P3 stage: a consuming position's required type is decided where the expression is built

`v8/RequiredConversions.class` is a real compiled sample: the sibling `RequiredConversions.java`
compiled by **javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 RequiredConversions.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `RequiredConversions` |
| class-file version | 52.0 (Java 8) |
| bytes | 1031 |
| SHA-256 | `d71eeaa5dc3eb66eafffda9e0c509eae43e3dde566b67df0006348352778d1e4` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_required_conversions.rs` (the exact text of every member, the parts' own conversion, and the refusal of the shapes no conversion states) and `tests/p3_execution_comparison.rs` (every member compiled under a declaration derived from the run's own facts and executed against the original) |

## What the defect is (T5)

A `char`, a `byte` and a `short` share one **slot shape** with an `int`, so a compiler inserts no
instruction when it widens one of them: `append((int) c)` is a `load` plus the `append`, and
`int local = c;` is a `load` plus a store. The conversion therefore lives in the *consumer's*
descriptor — the `append`'s own parameter type, the callee's parameter list, the member's return
type, the local's declaration, the field's descriptor — and a presentation that writes the value's
own text publishes the consumer's **own**, different conversion:

```text
source: new StringBuilder().append((int) c).append("!").toString()
text:   "" + arg0 + "!"            // c == 'A' answers "A!", the class answers "65!"
```

Both texts compile under the member's own declaration, and the pre-fix run reported
`Java`/`Structured`/`contains_statements`/`complete` with no diagnostic: the planes cannot see a
value. `crates/jarde-java/src/build.rs` saved the `append`'s parameter type and adapted `boolean`
alone.

| member | pre-fix text (measured through the recovery entry) | pre-fix value | the class's own value |
| --- | --- | --- | --- |
| `castPart(char)` | `return "" + arg0 + "!";` | `"A!"` | `"65!"` |
| `castPartLast(String, char)` | `return arg0 + arg1;` | `"aA"` | `"a65"` |
| `argued(char)` | `return widen(arg0);` | `65` (agrees) | `65` |
| `arguedByte(byte)` | `return widen(arg0);` | `65` (agrees) | `65` |
| `returned(char)` | `return arg0;` | `65` (agrees) | `65` |
| `returnedShort(short)` | `return arg0;` | `65` (agrees) | `65` |
| `declared(char)` | `int local1 = arg0; return local1;` | `65` (agrees) | `65` |
| `assigned(char)` | `int local1 = 0; local1 = arg0; return local1;` | `65` (agrees) | `65` |
| `written(char)` | `RequiredConversions.field = arg0; return RequiredConversions.field;` | `65` (agrees) | `65` |
| `intPart(int)`, `kept(char)`, `widen(int)` | unchanged (the controls) | — | — |

The two members whose *value* differed are the concat parts; the argument, return and write members
agreed in value but published the conversion as the context's own reading of the value rather than
as text. `"" + arg0` is a string concatenation of a **character** where the bytecode converted the
code unit, and `f(arg0)`/`return arg0;`/`local1 = arg0;` rely on the reader taking the widening on
faith. After the change every position states the conversion its own type requires, and a conversion
no evidence relates (a `boolean` beside another primitive) is refused with its bytecode quoted
(`tests/p3_required_conversions.rs`'s hand-built `p/RequiredBoundary`).

## What each member is for, and what it is presented as now

| member | the position's requirement | text | the control it is |
| --- | --- | --- | --- |
| `castPart(char)` | the `append`'s parameter is `int` | `return "" + (int) arg0 + "!";` | the reported shape: the part states the conversion its own `append` performs |
| `castPartLast(String, char)` | the same, after a `String` part | `return arg0 + (int) arg1;` | the conversion does not depend on the empty string a chain starts from |
| `intPart(int)` | the `append`'s parameter is `int` | `return "" + arg0 + "!";` | the control: an `int` part needs no conversion, byte for byte |
| `widen(int)` | the member returns `int` | `return arg0;` | the control: the value already is the required type |
| `argued(char)` | `widen`'s parameter is `int` | `return widen((int) arg0);` | a call argument meets the callee's own descriptor |
| `arguedByte(byte)` | the same | `return widen((int) arg0);` | the same for a `byte` |
| `returned(char)` | the member returns `int` | `return (int) arg0;` | a `return` meets the member's own descriptor |
| `returnedShort(short)` | the same | `return (int) arg0;` | the same for a `short` |
| `kept(char)` | the member returns `char` | `return arg0;` | the control: the value already is the returned type |
| `declared(char)` | `local1` was declared `int` | `int local1 = (int) arg0; return local1;` | a declaration's value meets the variable's own type |
| `assigned(char)` | the same, after the declaration | `int local1 = 0; local1 = (int) arg0; return local1;` | and so does a later assignment |
| `written(char)` | the field's descriptor is `int` | `RequiredConversions.field = (int) arg0; return RequiredConversions.field;` | a field write meets the field's own descriptor |

The three controls are what keep the conversion from being decoration: a value whose presented type
**is** the required one gains nothing, and the six characters `(int) ` are the whole of what the
change adds to a text.

## The controlled compile-and-execute comparison

`tests/p3_execution_comparison.rs` (`cargo test --test p3_execution_comparison --locked -- --ignored`)
wraps every member in a scratch class that extends this sample, compiles the recovered body under a
declaration derived from the run's own facts, and runs both sides with the same inputs
(`P3_COMPARISON_TRACES=1` prints the lines). Both sides printed, for this sample:

```text
value castPart(C)Ljava/lang/String; ['A'] -> "65!"
value castPart(C)Ljava/lang/String; ['a'] -> "97!"
value castPart(C)Ljava/lang/String; ['0'] -> "48!"
value castPart(C)Ljava/lang/String; ['\u0000'] -> "0!"
value castPartLast(Ljava/lang/String;C)Ljava/lang/String; ["a", 'A'] -> "a65"
value castPartLast(Ljava/lang/String;C)Ljava/lang/String; ["", '0'] -> "48"
value castPartLast(Ljava/lang/String;C)Ljava/lang/String; [null, 'A'] -> "null65"
value intPart(I)Ljava/lang/String; [7] -> "7!"   (and 0, -1)
value widen(I)I [7] -> 7   (and 0, -1)
value argued(C)I ['A'] -> 65   (and '0', '\u0000')
value arguedByte(B)I [(byte) 65] -> 65   (and (byte) -1, (byte) 0)
value returned(C)I ['A'] -> 65   (and '\u0000')
value returnedShort(S)I [(short) 65] -> 65   (and (short) -1)
value kept(C)C ['A'] -> A   (and '\u0000')
value declared(C)I ['A'] -> 65   (and '\u0000')
value assigned(C)I ['A'] -> 65   (and '\u0000')
value written(C)I ['A'] -> 65   (and '\u0000')
```

The pre-fix run of the same comparison failed on the first line — `original "65!"` against
`generated "A!"` — which is the finding's own evidence; the run after the change prints the same
lines on both sides.

## The bytecode

```text
public static java.lang.String castPart(char);   // bb 00 07 59 b7 00 09 1a b6 00 0a 12 0e b6 00 10 b6 00 13 b0
     0: new           #7                  // class java/lang/StringBuilder
     3: dup
     4: invokespecial #9                  // Method java/lang/StringBuilder."<init>":()V
     7: iload_0                            // the `char` parameter, with no conversion instruction
     8: invokevirtual #10                 // Method java/lang/StringBuilder.append:(I)Ljava/lang/StringBuilder;
    11: ldc           #14                 // String !
    13: invokevirtual #16                 // Method append:(Ljava/lang/String;)…
    16: invokevirtual #19                 // Method java/lang/StringBuilder.toString:()Ljava/lang/String;
    19: areturn

public static java.lang.String castPartLast(java.lang.String, char);  // bb 00 07 59 b7 00 09 2a b6 00 10 1b b6 00 0a b6 00 13 b0
     0: new / 3: dup / 4: invokespecial <init>()V
     7: aload_0                            // the `String` part
     8: invokevirtual #16                 // append:(Ljava/lang/String;)…
    11: iload_1                            // the `char` parameter
    12: invokevirtual #10                 // append:(I)…
    15: invokevirtual #19                 // toString:()Ljava/lang/String;
    18: areturn

public static int widen(int);              // 1a ac
     0: iload_0
     1: ireturn

public static int argued(char);            // 1a b8 00 17 ac
     0: iload_0                            // the `char` parameter
     1: invokestatic  #23                 // Method widen:(I)I  — the JVM widens with no instruction
     4: ireturn

public static int arguedByte(byte);        // 1a b8 00 17 ac
     0: iload_0                            // the same body for a `byte` parameter

public static int returned(char);          // 1a ac
     0: iload_0
     1: ireturn                            // the member's own descriptor says `I`

public static int returnedShort(short);    // 1a ac
public static char kept(char);             // 1a ac  — the control: the two types are the same

public static int declared(char);          // 1a 3c 1b ac
     0: iload_0
     1: istore_1                           // the frame states `int` for the `char` value
     2: iload_1
     3: ireturn

public static int assigned(char);          // 03 3c 1a 3c 1b ac
     0: iconst_0 / 1: istore_1             // `int local = 0;`
     2: iload_0 / 3: istore_1              // `local = c;`
     4: iload_1 / 5: ireturn

public static int written(char);           // 1a b3 00 1d b2 00 1d ac
     0: iload_0
     1: putstatic     #29                 // Field field:I
     4: getstatic     #29                 // Field field:I
     7: ireturn
```

Every member is a straight line with no branch and no handler, so the body needs no
`StackMapTable`; the frames a version-52 verifier derives are the ones the layer reads, and the
`char`/`byte`/`short` positions are stated by the descriptors above and nowhere else.
