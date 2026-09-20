# P3 stage A fixture: a type position is spelled as a Java type, arrays included

`v8/ArrayTypes.class` is a real compiled sample: the sibling `ArrayTypes.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 ArrayTypes.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `ArrayTypes` |
| class-file version | 52.0 (Java 8) |
| bytes | 511 |
| SHA-256 | `84cea788f628a076686dcd04c96df91c970ef40ff4e6e740ec14dfb0a0a3ac59` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_array_types.rs` (exact text, the four defect shapes and the refusal boundary) and `tests/p3_execution_comparison.rs` (six members compiled under a declaration derived from the run's own descriptor and executed against the original) |

## What the defect is, and what each member is for

The frame pass keeps a named reference in the class file's own descriptor form
(`jarde_jvm::frame::RefType::Named`) and names every array by its whole descriptor
(`parse_field_type`'s `[` branch returns `[B`, `[[I`, `[[Ljava/lang/String;` …). The presentation's
spelling entry point (`build::spell_reference`) took the `L…;` wrapping off and turned `/` into `.`,
and for an array descriptor it did neither: the descriptor itself became the type of the declaration.
The artifact was structurally whole (`Java`/`Structured`/`contains_statements`, no diagnostic) and
`javac --release 8` refuses it in the member's own declaration:

| member | source shape | pre-fix declaration | `javac --release 8` on the wrapper |
| --- | --- | --- | --- |
| `echoed([B)[B` | `byte[] local = copy(value); return local;` | `[B local1 = copy(arg0);` | `error: illegal start of expression` at the `[B` |
| `named([Ljava/lang/String;)[Ljava/lang/String;` | `String[] local = value; return local;` | `[Ljava.lang.String; local1 = arg0;` | `error: illegal start of expression` (the `[`) and `error: not a statement` (the `.`) |
| `grid([[I)[[I` | `int[][] local = value; return local;` | `[[I local1 = arg0;` | `error: illegal start of expression` |
| `table([[Ljava/lang/String;)[[Ljava/lang/String;` | `String[][] local = value; return local;` | `[[Ljava.lang.String; local1 = arg0;` | `error: illegal start of expression` and `error: not a statement` |

The exact pre-fix texts are the measured output of this same sample with the fix reverted (the
mutation `tests/p3_array_types.rs` records), and the `javac` lines are its refusals of those texts
wrapped in the declaration derived from the same run's facts:

```text
Genechoed.java:7: error: illegal start of expression
    [B local1 = copy(arg0);
    ^
1 error
```

The members are the three defect sites (a primitive array, a reference array, and both
two-dimensional forms), the call-result shape the review measured on a real sample
(`DSTU4145Signer.hash2FieldElement`'s `[B local2 = reverse(arg1)`), and the controls that must not
move:

| member | descriptor | what it pins |
| --- | --- | --- |
| `echoed` | `([B)[B` | the defect at its shortest: a local filled from a call that returns a `byte[]` |
| `named` | `([Ljava/lang/String;)[Ljava/lang/String;` | the element name keeps the object-name rule (`L…;` off, `/` → `.`) |
| `grid` | `([[I)[[I` | the primitive multi-dimensional form: one `[]` per dimension |
| `table` | `([[Ljava/lang/String;)[[Ljava/lang/String;` | the reference multi-dimensional form (the shape `lambda.rs::parse_type` already spelled) |
| `copy` | `([B)[B` | the helper `echoed` calls — a `[B` parameter and `[B` return with **no** type position, so nothing about it may change |
| `text` | `(Ljava/lang/String;)Ljava/lang/String;` | the positive control: an object type's declaration, byte-for-byte what it was before |

```text
public ArrayTypes();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

public static byte[] copy(byte[]);  // 2a b0
     0: aload_0
     1: areturn

public static byte[] echoed(byte[]);  // 2a b8 00 07 4c 2b b0
     0: aload_0
     1: invokestatic  #7                  // Method copy:([B)[B
     4: astore_1
     5: aload_1
     6: areturn

public static java.lang.String[] named(java.lang.String[]);  // 2a 4c 2b b0
     0: aload_0
     1: astore_1
     2: aload_1
     3: areturn

public static int[][] grid(int[][]);  // 2a 4c 2b b0
     0: aload_0
     1: astore_1
     2: aload_1
     3: areturn

public static java.lang.String[][] table(java.lang.String[][]);  // 2a 4c 2b b0
     0: aload_0
     1: astore_1
     2: aload_1
     3: areturn

public static java.lang.String text(java.lang.String);  // 2a 4c 2b b0
     0: aload_0
     1: astore_1
     2: aload_1
     3: areturn
```

## What each member recovered, before and after

Pre-fix texts are the mutation's measured output; post-fix texts are the committed behaviour, and
each is asserted **exactly** (the whole body, not a line of it) in `tests/p3_array_types.rs`.

| member | before | after |
| --- | --- | --- |
| `echoed([B)[B` | `[B local1 = copy(arg0); return local1;` | `byte[] local1 = copy(arg0); return local1;` |
| `named([Ljava/lang/String;)[Ljava/lang/String;` | `[Ljava.lang.String; local1 = arg0; return local1;` | `java.lang.String[] local1 = arg0; return local1;` |
| `grid([[I)[[I` | `[[I local1 = arg0; return local1;` | `int[][] local1 = arg0; return local1;` |
| `table([[Ljava/lang/String;)[[Ljava/lang/String;` | `[[Ljava.lang.String; local1 = arg0; return local1;` | `java.lang.String[][] local1 = arg0; return local1;` |
| `copy([B)[B` | `return arg0;` | unchanged (no type position) |
| `text(Ljava/lang/String;)Ljava/lang/String;` | `java.lang.String local1 = arg0; return local1;` | unchanged |

## What the fix is, and the positions it does not cover

An array descriptor is read by this crate's one descriptor→Java type parser
(`lambda::parse_type`, which already spelled lambda parameters as `int[]`, `java.lang.String[][]`),
so a declaration and a lambda parameter cannot drift; the object-name rule (`L…;` off, `/` → `.`) is
the one both entries use for the element. The same entry point is now fallible: a descriptor with no
Java type to be is refused at the position that would have carried it rather than published.

A type reaches the text from five shapes. Which of them can carry an array descriptor, and what
covers each:

| position | an array descriptor? | covered by |
| --- | --- | --- |
| a local's declaration (`declare`/`declare_at`, and the hoisted declaration) | **yes — the defect** | this sample: four array members, exactly asserted and compiled |
| a `try`-with-resources header (`resource_declaration`) | through the same `value_type`, so the same spelling | the same entry point; no member here reaches it, and no compiler emits a resource whose type is not `AutoCloseable` |
| a lambda's parameters (`lambda::parse_type` → `emit`) | yes, and it was already spelled | the crate's own tests (`int[]`, `java.lang.String[][]`), reused here |
| a static member's owner (`field_read`/`field_write`'s pool owner) | **yes, from hand-built bytes** (measured) | `tests/p3_array_types.rs`: `int[].value` is spelled by the array rule and the descriptor is not published; the *position* is a recorded boundary (below) |
| a construction's class (`new_expr`) and a dynamic site's implementation class | **yes, from hand-built bytes** (measured, the construction) | the same: `new int[]()`; unspellable names refuse in both positions |

Two of those rows are boundaries this change records rather than accepts, and both are the
*position's* limit, not the spelling's:

- A `CONSTANT_Class` may hold an array descriptor (JVMS 4.4.1), and a hand-built `Fieldref` whose
  class is `[I` or an `<init>` whose class is `[I` reaches the owner position and the construction
  position respectively. The layer spells the name with the array rule (`int[].value`, `new int[]()`),
  which is a legal Java *type* in a position no Java *expression* admits: javac refuses both
  (`class expected` at the `.` after `int[]`, `array dimension missing` for `new int[]()`), and it
  refused the pre-fix texts (`[I.value`, `new [I()`) the same way. No legal class file states either
  shape — JVMS 4.4.2 requires a `Fieldref`'s class to be a class or interface type that has the
  field, and an array type declares no `<init>` — so no source and no compiler produces them.
  `tests/p3_array_types.rs` pins the spelling at both positions and states the boundary; a position
  that admits no array type is a question this change does not decide.
- A `checkcast` to an array type is a class-file fact like any other (`[Ljava/lang/String;` in the
  pool), and the layer never spells it from that value: the cast shape is refused as a whole before a
  type is written. A member of this source carrying `String[] local = (String[]) value;` recovers
  `Mixed`/`Fallback` with quoted bytecode and *no* declaration at all (measured on this sample's
  compiler output before the fixture was trimmed, which is why the shape is not a member here).

The descriptors that have no Java spelling at all are refused at whichever position would have
carried them: `[`, `[Lfoo` and `L;` never reach the text. `tests/p3_array_types.rs`'s hand-built
class covers a declaration filled from a `L;` value and a static field of type `[L;`, and an owner
and a construction whose class name is `[`; all four are `Mixed`/`Fallback` with their bytecode
quoted and the fact named at the position's own BCI.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: these are the original class's own answers for the input set the comparison calls the
members with (`null` for every array parameter — `sample_values` has no value for an array type —
plus the one `byte[]` argument `echoed`'s calls state). A trace cannot compare an `Object[]` (the
helper prints a hash), so the driver prints the arrays element by element while the trace compares a
`byte[]` result by its class.

```text
copy(null)=null
echoed({1, 2})=[1, 2]
named(null)=null
grid(null)=null
table(null)=null
text("r")=r
text(null)=null
```

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the
run that introduced this sample:

| member | run | content | wrapper | result |
| --- | --- | --- | --- | --- |
| `copy([B)[B` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `echoed([B)[B` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `named([Ljava/lang/String;)[Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `grid([[I)[[I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `table([[Ljava/lang/String;)[[Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `text(Ljava/lang/String;)Ljava/lang/String;` | Java/Structured | contains_statements | compiles | executed: traces identical |

- the member declarations the comparison derived from the run's own facts: `public static byte[] copy(byte[] arg0)`,
  `public static byte[] echoed(byte[] arg0)`, `public static java.lang.String[] named(java.lang.String[] arg0)`,
  `public static int[][] grid(int[][] arg0)`, `public static java.lang.String[][] table(java.lang.String[][] arg0)`,
  `public static java.lang.String text(java.lang.String arg0)`
- (`<init>()V` is not wrapped: a constructor cannot be re-declared in the scratch class)
- trace: 7 line(s), identical on both sides — `value echoed([B)[B [new byte[] {1, 2}] -> class=[B`
  is the one array input a trace can compare, since `show` states a `byte[]` by its class
- the wrapper extends the sample for `echoed`, whose text names the member `copy` the committed
  class declares

The **pre-fix** state is recorded by the mutation: with `spell_reference` passing an array descriptor
through again, `tests/p3_array_types.rs` fails on the two exact-text assertions and on the
descriptor-leak check with `[B local1 = copy(arg0);`, `[Ljava.lang.String; local1 = arg0;`,
`[[I local1 = arg0;`, and the compiled comparison stops on
`Genechoed.java:7: error: illegal start of expression`. The same mutation also publishes the
unspellable `[L;` as the declaration's type (`[L; local0 = Unspellable.field;`), so the refusal
boundary fails with it. Neither is a skip: the member's text must compile under the declaration
derived from the run's own facts, and a type position must state a Java type.

## Reproducing

```text
cd tests/fixtures/p3-array-types
mkdir -p v8
javac --release 8 -g:none -d v8 ArrayTypes.java
shasum -a 256 v8/ArrayTypes.class   # 84cea788f628a076686dcd04c96df91c970ef40ff4e6e740ec14dfb0a0a3ac59
```

`Baseline.java` is not compiled into this directory: the comparison test compiles it in a temporary
directory with the sample on its classpath, so no driver class is ever committed beside the sample.
