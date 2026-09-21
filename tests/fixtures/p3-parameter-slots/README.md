# P3 fixture: one parameter's slot is the descriptor's own statement

`v8/SlotTypes.class` is a real compiled sample: the sibling `SlotTypes.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 SlotTypes.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时…` warnings (3 of them) and writes the
class anyway, exit code 0. The compiler is a generation-only input: it is not needed at run time, so
the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `SlotTypes` |
| class-file version | 52.0 (Java 8) |
| bytes | 605 |
| SHA-256 | `ba2a1380f4530f578a507a3587bce97b426979a28267a1d7db7294e190249c12` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_parameter_slots.rs` (exact declarations and bodies, the pre-fix text, and the compiled-and-executed comparison of the presentation's own text) and `tests/p3_execution_comparison.rs` (`SlotTypes` is the `p3-parameter-slots` row: nine bodies compiled under a declaration derived from the run's own facts and executed against the original) |

## What the defect is

JVMS 2.6.1 gives a `long` or a `double` **two local slots** and every other type one, and JVMS 4.3.2
says what an array is: a reference. So `[J`, `[[J`, `[D` and `[[D` fill **one** slot — exactly like
`[I` — and only an unadorned `J`/`D` fills two.

The class-source signature reader took the array's placement from its **element**: `descriptor_type`
returned the base type's width for the whole component, so every parameter *after* an array of a
`long`/`double` was placed one slot too far while the member's body — whose `arg<slot>` names come
from the real slot numbering — read the slot the descriptor really states:

```text
public static int afterArray(long[] arg0, int arg2) {   // the signature's `arg2`
    return arg1;                                        // the body's `arg1`
}
```

`javac --release 8` refuses that text (`cannot find symbol: variable arg1`), and the sample's own
bytecode states the truth: `afterArray(long[], int)` is `iload_1; ireturn`.

## The members, and the slot arithmetic

Every member reads the parameter its descriptor places **last**, so a misplaced slot is a name the
body does not use. `-g:none` means the names are the ordinal ones (`arg<slot>`).

| member | descriptor | slots (JVMS 2.6.1) | post-fix declaration | pre-fix declaration | body |
| --- | --- | --- | --- | --- | --- |
| `afterArray` | `([JI)I` | `[J`→0, `I`→1 | `public static int afterArray(long[] arg0, int arg1)` | `…(long[] arg0, int arg2)` | `return arg1;` |
| `afterNested` | `([[JI)I` | `[[J`→0, `I`→1 | `public static int afterNested(long[][] arg0, int arg1)` | `…(long[][] arg0, int arg2)` | `return arg1;` |
| `afterGrid` | `([[DJ)J` | `[[D`→0, `J`→1 (2 slots) | `public static long afterGrid(double[][] arg0, long arg1)` | `…(double[][] arg0, long arg2)` | `return arg1;` |
| `betweenWide` | `(J[JI)I` | `J`→0 (2), `[J`→2, `I`→3 | `public static int betweenWide(long arg0, long[] arg2, int arg3)` | `…(long arg0, long[] arg2, int arg4)` | `return arg3;` |
| `arraysOnly` | `([J[DJ)J` | `[J`→0, `[D`→1, `J`→2 (2) | `public static long arraysOnly(long[] arg0, double[] arg1, long arg2)` | `…(long[] arg0, double[] arg2, long arg4)` | `return arg2;` |
| `instanceAfterArray` | `([JI)I` | *this*→0, `[J`→1, `I`→2 | `public int instanceAfterArray(long[] arg1, int arg2)` | `…(long[] arg1, int arg3)` | `return arg2;` |
| `instanceWide` | `(J[JI)I` | *this*→0, `J`→1 (2), `[J`→3, `I`→4 | `public int instanceWide(long arg1, long[] arg3, int arg4)` | `…(long arg1, long[] arg3, int arg5)` | `return arg4;` |

The two controls:

| member | descriptor | what it pins |
| --- | --- | --- |
| `echoArray` | `([J)[J` | an array of a wide primitive in **parameter 0** and in the return type: nothing follows it, so its own spelling (`long[]`, not `long`) is the whole claim — and the pre-fix text is already right |
| `control` | `([II)I` | an array whose element is one slot wide: the parameter after it was placed correctly before this change and still is |
| `<init>` | `()V` | the instance receiver is slot 0 and is written `this` where the body reads it (`this.offset = 3;`), never as a parameter name — the P3 receiver rule, which this change must not move |

```text
public SlotTypes();                     // 2a b7 00 01 2a 06 b5 00 07 b1
public static int afterArray(long[], int);        // 84 01 ac
public static int afterNested(long[][], int);     // 84 01 ac
public static long afterGrid(double[][], long);   // 85 01 ad
public static int betweenWide(long, long[], int); // 84 03 ac
public static long arraysOnly(long[], double[], long); // 86 02 ad
public static long[] echoArray(long[]);           // 2a b0
public static int control(int[], int);            // 84 01 ac
public int instanceAfterArray(long[], int);       // 84 02 ac
public int instanceWide(long, long[], int);       // 84 04 ac
```

Each body is one `iload_1`/`iload_2`/`iload_3`/`iload 4`/`lload_1`/`lload_2`/`aload_0` plus its
return, so the slot the bytecode names is the slot the sample states.

## What the fix is

The descriptor is read once, by the reader's own facts
(`jarde_reader::classfile::descriptor_facts`), and the **one** place a slot rule lives is
`DescriptorComponent::slots`: a `long`/`double` component fills two slots, every other component —
an array of anything included — fills one. The positions a member's parameters occupy are derived
from those facts in the JVM layer (`jarde_jvm::method_ir::parameter_positions`), with slot 0 the
receiver of a member that is not `static`, and the class-source presentation only *writes* what
those two state (types through the recovery layer's own spelling, names through the slot the body
already uses).

So the signature's `arg1` after a `long[]` and the body's `arg1` are the same reading of one
descriptor, and the frame pass, the name numbering, the parameter-type fact and the declaration
cannot disagree about where a parameter sits.

## The counterexample: the mutation, and what it fails on

Putting the array's element width back, in the one place the rule now lives
(`DescriptorComponent::slots`: `if self.dimensions > 0` → the base's own width), makes every check
that covers this sample go red:

* `tests/p3_parameter_slots.rs` fails on the exact text: the declaration places the `int` at `arg2`
  where the descriptor puts it at slot 1 — and the member's own run stops, because the frame pass
  reads the same rule and the local it reads is no longer the one the bytecode loads:

```text
---- the_declaration_and_the_body_name_one_slot_one_way stdout ----
thread '…' panicked at tests/p3_parameter_slots.rs:320:9:
`afterArray` must declare `public static int afterArray(long[] arg0, int arg1)`:
    public static int afterArray(long[] arg0, int arg2) {
        // jarde: not recovered: the recovery run for `afterArray([JI)I` stopped (jre_ir_table_missing); the analysis of that member did not complete (ir_frame_inconsistent)
    }
```

* `tests/p3_execution_comparison.rs` fails first, before any wrapper is compiled, on the run's own
  parameter-type fact — the slot the descriptor states is the one the fact must place:

```text
thread 'the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies' panicked at
tests/p3_execution_comparison.rs:2591:13:
assertion `left == right` failed: p3-parameter-slots/v8 (javac 23.0.1, --release 8 -g:none):
`afterArray([JI)I` - the run's parameter-type fact for slot 1 must spell what the descriptor states
```

* the JDK-driven comparison (`-- --ignored`) then has no trace to compare: the members whose runs
  stopped carry no body, so the presentation does not compile (`error: missing return statement`,
  seven times).

The **pre-fix text itself** (the defect as it stood, with the frame pass and the name numbering
unmoved) is reproduced by mutating only `class_source::method_descriptor` back to the element
reading; `tests/p3_parameter_slots.rs -- --ignored` then fails on the presentation's own compile,
with the body recovered exactly as before:

```text
SlotTypes.java:19: error: cannot find symbol
        return arg1;
               ^
  symbol:   variable arg1
  location: class SlotTypes
SlotTypes.java:33: error: cannot find symbol
        return arg1;
               ^
  symbol:   variable arg1
  location: class SlotTypes
SlotTypes.java:40: error: cannot find symbol
        return arg3;
…
SlotTypes.java:47: error: incompatible types: double[] cannot be converted to long
        return arg2;
…
SlotTypes.java:68: error: cannot find symbol
        return arg2;
…
SlotTypes.java:75: error: cannot find symbol
        return arg4;
…
7 errors
```

Every failure is one parameter placed one slot too far — `arg2` where the body reads `arg1`, `arg5`
where it reads `arg4` — and `arraysOnly`'s `arg2` even binds to the wrong *type*, because the
declaration named the `double[]` parameter with it.

## Reproducing

```text
cd tests/fixtures/p3-parameter-slots
mkdir -p v8
javac --release 8 -g:none -d v8 SlotTypes.java
shasum -a 256 v8/SlotTypes.class   # ba2a1380f4530f578a507a3587bce97b426979a28267a1d7db7294e190249c12
cargo test --test p3_parameter_slots --locked
cargo test --test p3_parameter_slots --locked -- --ignored   # needs a JDK on PATH
```

`SlotTypes.java` is the fixture's source of record and is compiled into `v8/` only; no driver class
is committed beside it (the tests build their own drivers).
