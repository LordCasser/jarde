# P3 stage A fixture: a boolean context is typed by a descriptor, a local's declaration included

`v8/BooleanContexts.class` is a real compiled sample: the sibling `BooleanContexts.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 BooleanContexts.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `BooleanContexts` |
| class-file version | 52.0 (Java 8) |
| bytes | 1114 |
| SHA-256 | `e6a85f9aa40f585642696f0a391c8be9c8fafa2100e0cf0f9ab7ebd6bfb2bf0d` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_boolean_contexts.rs` (exact text, refusals and the hand-built shapes) and `tests/p3_execution_comparison.rs` (sixteen members compiled under a declaration derived from the run's own facts and executed against the original) |

## What the defect is, and what each member is for

The frames state **one** slot shape for the four int-sized primitives (`boolean`, `byte`, `char`,
`short`), so a body read on its own cannot tell a `boolean` from an `int`: only a descriptor says
it. Three positions were presented without that fact, and each text is refused by `javac --release 8`
when it is wrapped in the member's own signature:

| member | source shape | pre-fix text | `javac --release 8` |
| --- | --- | --- | --- |
| `isZero(I)Z` | `if (x == 0) { return true; } return false;` | `return 1;` / `return 0;` | `error: incompatible types: int cannot be converted to boolean` (twice) |
| `flag()Z` | `return true;` | `return 1;` | `error: incompatible types: int cannot be converted to boolean` |
| `parity(I)I` | `if (flag()) { return 1; } return 0;` | `if (flag() != 0)` | `error: incomparable types: boolean and int` |
| `staticFlagCount()I` | `if (staticFlag) { return 1; } return 0;` | `if (BooleanContexts.staticFlag != 0)` | `error: incomparable types: boolean and int` |
| `throughLocal(Z)Z` | `boolean c = b; return c;` | `int local1 = arg0; return local1;` | `error: incompatible types: boolean cannot be converted to int` (the declaration) and `int cannot be converted to boolean` (the return) |
| `localFromCall()Z` | `boolean c = flag(); return c;` | `int local0 = flag(); return local0;` | `error: incompatible types: boolean cannot be converted to int` (the declaration) |
| `pick(IZZ)Z` | `boolean c; if (n != 0) c = a; else c = b; return c;` | `int local3; … local3 = arg1; … return local3;` | `error: incompatible types: boolean cannot be converted to int` (each assignment) |
| `fromLocal(Z)Z` | `boolean c = b; boolean d = c; return d;` | `int local1 = arg0; int local2 = local1; return local2;` | `error: incompatible types: boolean cannot be converted to int` (both declarations) |

Every one of those runs reported `Java`/`Structured`/`contains_statements`/`complete` with no
diagnostic: the artifact was structurally whole and its *values* were typed wrongly. The structural
planes cannot see a type, which is why the acceptance is the text plus a compiler.

The members are the three defect sites, the evidence a boolean context must keep presenting, and the
controls that must not move:

| member | descriptor | what it pins |
| --- | --- | --- |
| `isZero` | `(I)Z` | the `Z` return of a literal: `return true;`/`return false;` |
| `flag` | `()Z` | the same with one return |
| `parity` | `(I)I` | a condition whose operand is a call whose **callee** descriptor returns `Z` |
| `passed` | `(Z)Z` | a condition/return over a `boolean` **parameter** (P3-R5's existing fact) |
| `callFlag` | `()Z` | a `Z` return of a proven boolean value (unchanged by the fix) |
| `fieldFlag` | `()Z` | a `Z` return of a claimed field read whose pool descriptor is `Z` (unchanged) |
| `staticFlagCount` | `()I` | the same fact as a condition (`getstatic`) |
| `localFromCall` | `()Z` | a local declared from a `Z`-returning call's result, read afterwards |
| `pick` | `(IZZ)Z` | a local declared **above** the branch that fills it (the hoisted-declaration path), read through the merge |
| `fromLocal` | `(Z)Z` | a local copied from a local the body already declared `boolean` |
| `assignFromCall` | `(Z)Z` | a store into a variable whose type the **signature** states (`b = flag();` on a `Z` parameter) |
| `throughLocal` | `(Z)Z` | the smallest shape of the third site: `boolean c = b; return c;` |
| `negated` | `(Z)Z` | a value a branch merged out of two constant pushes: refused (as before the change) |
| `count` | `(Z)I` | the control: a `boolean` parameter's condition keeps `if (arg0)` |
| `intLocal` | `(I)I` | the control: `int x = 0; … x = 1;` stays `int` — a literal types no declaration |
| `nonzero` | `(I)I` | the control: a genuine `int` comparison keeps `if (arg0 != 0)` |
| `answer` | `()I` | the control: an `int` return keeps `return 1;` |

Five shapes no compiler emits are built in memory by `tests/p3_boolean_contexts.rs` instead of being
committed here (the repository's second fixture kind): `intReturn(I)Z` (`iload_0; ireturn`, an `int`
value returned from a `Z` method — legal bytecode, and the pre-fix text `return arg0;` is refused by
javac), `intLiteral()Z` (`iconst_2; ireturn`, a literal that is not a boolean), `literalCondition()I`
(`iconst_1; ifeq …`, the only way to reach the literal evidence in the condition position, since a
compiler folds a constant condition away), and `overwrite(Z)Z` / `paramOverwrite(Z)Z`, which store a
value no evidence proves boolean into a variable the run already knows holds a `boolean` (a local its
own first write declared `boolean`, and a `Z` parameter) — the source that would produce them
(`c = 2;` with a `boolean c`) is not Java, so only hand-written bytes carry that shape.

```text
public BooleanContexts();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

public static boolean isZero(int);  // 1a 9a 00 05 04 ac 03 ac
     0: iload_0
     1: ifne          6
     4: iconst_1
     5: ireturn
     6: iconst_0
     7: ireturn

public static boolean flag();  // 04 ac
     0: iconst_1
     1: ireturn

public static int parity(int);  // b8 00 07 99 00 05 04 ac 03 ac
     0: invokestatic  #7                  // Method flag:()Z
     3: ifeq          8
     6: iconst_1
     7: ireturn
     8: iconst_0
     9: ireturn

public static boolean passed(boolean);  // 1a ac
     0: iload_0
     1: ireturn

public static boolean callFlag();  // b8 00 07 ac
     0: invokestatic  #7                  // Method flag:()Z
     3: ireturn

public static boolean fieldFlag();  // b2 00 0d ac
     0: getstatic     #13                 // Field staticFlag:Z
     3: ireturn

public static boolean localFromCall();  // b8 00 07 3b 1a ac
     0: invokestatic  #7                  // Method flag:()Z
     3: istore_0
     4: iload_0
     5: ireturn

public static boolean pick(int, boolean, boolean);  // 1a 99 00 08 1b 3e a7 00 05 1c 3e 1d ac
     0: iload_0
     1: ifeq          9
     4: iload_1
     5: istore_3
     6: goto          11
     9: iload_2
    10: istore_3
    11: iload_3
    12: ireturn

public static boolean fromLocal(boolean);  // 1a 3c 1b 3d 1c ac
     0: iload_0
     1: istore_1
     2: iload_1
     3: istore_2
     4: iload_2
     5: ireturn

public static boolean assignFromCall(boolean);  // b8 00 07 3b 1a ac
     0: invokestatic  #7                  // Method flag:()Z
     3: istore_0
     4: iload_0
     5: ireturn

public static boolean throughLocal(boolean);  // 1a 3c 1b ac
     0: iload_0
     1: istore_1
     2: iload_1
     3: ireturn

public static boolean negated(boolean);  // 1a 9a 00 07 04 a7 00 04 03 ac
     0: iload_0
     1: ifne          8
     4: iconst_1
     5: goto          9
     8: iconst_0
     9: ireturn

public static int staticFlagCount();  // b2 00 0d 99 00 05 04 ac 03 ac
     0: getstatic     #13                 // Field staticFlag:Z
     3: ifeq          8
     6: iconst_1
     7: ireturn
     8: iconst_0
     9: ireturn

public static int intLocal(int);  // 03 3c 1a 99 00 08 04 3c a7 00 05 1a 3c 1b ac
     0: iconst_0
     1: istore_1
     2: iload_0
     3: ifeq          11
     6: iconst_1
     7: istore_1
     8: goto          13
    11: iload_0
    12: istore_1
    13: iload_1
    14: ireturn

public static int count(boolean);  // 1a 99 00 05 04 ac 03 ac
     0: iload_0
     1: ifeq          6
     4: iconst_1
     5: ireturn
     6: iconst_0
     7: ireturn

public static int nonzero(int);  // 1a 99 00 05 04 ac 03 ac
     0: iload_0
     1: ifeq          6
     4: iconst_1
     5: ireturn
     6: iconst_0
     7: ireturn

public static int answer();  // 04 ac
     0: iconst_1
     1: ireturn

static {};  // 04 b3 00 0d b1
     0: iconst_1
     1: putstatic     #13                 // Field staticFlag:Z
     4: return
```

`staticFlag` is a mutable `static` field initialised to `true` by the class initializer, so a
`field@1`-claimed read of a `boolean` member is a real field read (a `static final` constant would
have been inlined by the compiler into `iconst_1` and would not have exercised the claim).

## The evidence a boolean context reads, and what it does not

The layer proves a value is a boolean from what the class file and this body state, never from the
shape of the value:

| evidence | where the fact is read |
| --- | --- |
| a `load` of a parameter slot whose descriptor is `Z` | `MethodFacts::parameter_types` (P3-R5) |
| the result of a call whose callee descriptor returns `Z` | `CallTarget::descriptor()` (the fact `typed_arguments` reads) |
| a field access `field@1` claimed, whose pool descriptor is `Z` | the claim's own evidence (`field::Evidence::descriptor`) |
| a read of a local **this body declared** `boolean` | that declaration's own evidence, recorded where the declaration was written |
| a `0`/`1` literal | **only** in a position that already requires a boolean (a `Z` return, a branch test, a store into a variable already known boolean) |
| anything else — an arithmetic result, a comparison, a merge of unproven parts, a `load` of a slot no write proved boolean | **not** proven: a boolean context refuses it |

The `0`/`1` literal is deliberately **not** a declaration's type. A fresh local states no type at
all, so `int x = 0;` and `boolean c = true;` are the same bytes and the frame says `int` for both;
reading the literal as the declaration's type declares every `int` local filled with `0`/`1` a
boolean. That was measured, not assumed: with the literal added to the declaration decision,
`p3-scope`'s `scope(Z)I` recovers `boolean local1; if (arg0) { local1 = true; } else { …refused… }
return local1;` (`Mixed`/`Fallback`, where it is `Java`/`Structured` today) and this sample's own
`intLocal(I)I` degrades the same way — the execution comparison stops on
`p3-scope/v8 …: scope(Z)I is a body the run writes whole, and this run states Mixed`.

Two further boundaries, each deliberate:

- **No following of values.** The proof never walks a definition chain: a value whose boolean fact
  sits one hop away (a store's own value, a phi of constant pushes, a branch's merge) states nothing.
  `negated(Z)Z` is that boundary at its smallest and stays refused.
- **The hoisted path proves from descriptors only.** A variable whose declaration is written above
  the region that fills it has its type decided before any statement exists, so its first write's
  stored value is read through the descriptor items alone; a hoisted variable copied from another
  local is typed by the frame, and its boolean uses refuse.

## What each member recovered, before and after

Pre-fix texts are the measured output of the same sample with the fix removed (the mutations the
change records: the `Return` branch renders by value without the return descriptor, the condition
recognizes only a `boolean` parameter, and the declaration reads no boolean evidence); post-fix texts
are the committed behaviour.

| member | before | after |
| --- | --- | --- |
| `isZero(I)Z` | `if (arg0 == 0) { return 1; } else { return 0; }` | `if (arg0 == 0) { return true; } else { return false; }` |
| `flag()Z` | `return 1;` | `return true;` |
| `parity(I)I` | `if (flag() != 0) { … }` | `if (flag()) { … }` |
| `passed(Z)Z` | `return arg0;` | unchanged |
| `callFlag()Z` | `return flag();` | unchanged |
| `fieldFlag()Z` | `return BooleanContexts.staticFlag;` | unchanged (the field's descriptor is the proof) |
| `localFromCall()Z` | `int local0 = flag(); return local0;` (`Java`/`Structured`) | `boolean local0 = flag(); return local0;` |
| `pick(IZZ)Z` | `int local3; … local3 = arg1; … return local3;` (`Java`/`Structured`) | `boolean local3; … local3 = arg1; … return local3;` |
| `fromLocal(Z)Z` | `int local1 = arg0; int local2 = local1; return local2;` (`Java`/`Structured`) | `boolean local1 = arg0; boolean local2 = local1; return local2;` |
| `assignFromCall(Z)Z` | `arg0 = flag(); return arg0;` | unchanged (the variable's type is the signature's) |
| `throughLocal(Z)Z` | `int local1 = arg0; return local1;` (`Java`/`Structured`) | `boolean local1 = arg0; return local1;` |
| `negated(Z)Z` | refused: `… is the entry state of stack depth 0 …` | refused, same reason |
| `staticFlagCount()I` | `if (BooleanContexts.staticFlag != 0) { … }` | `if (BooleanContexts.staticFlag) { … }` |
| `intLocal(I)I` | `int local1; local1 = 0; … local1 = 1; … return local1;` | unchanged (the literal control) |
| `count(Z)I` | `if (arg0) { … }` | unchanged |
| `nonzero(I)I` | `if (arg0 != 0) { … }` | unchanged |
| `answer()I` | `return 1;` | unchanged |

The refusals keep the layer's existing contract: the artifact is `Mixed`/`Fallback`, its text quotes
the bytecode it could not present (`// @bytecode 3`), the index stays anchored in the segment table,
and the reason names the position that could not be typed:

```text
{
    boolean local1 = arg0;
    // @bytecode 3
    // the value at BCI 3 is stored into `local1`, which this run already stated holds a `boolean`, and this layer has no evidence that the value is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the variable's own type rejects
    return local1;
}
```

That is the hand-built `overwrite(Z)Z` (`iload_0; istore_1; iconst_2; istore_1; iload_1; ireturn`):
the declaration and the later boolean use are written, and only the unprovable store is refused.
`paramOverwrite(Z)Z` (`iconst_2; istore_0; iload_0; ireturn`) states the same rule for a variable
whose type the signature states.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: these are the original class's own answers for the input set, and they are the values the
generated side must return (`isZero(0)`/`isZero(1)` are the inputs the first finding was measured on;
the default `int` set is 7, 0, -1, so `1` is stated explicitly in `tests/p3_execution_comparison.rs`,
and `pick`/`intLocal` state theirs so each arm runs).

```text
isZero(0)=true
isZero(1)=false
isZero(7)=false
isZero(-1)=false
flag()=true
parity(0)=1
parity(1)=1
passed(true)=true
passed(false)=false
callFlag()=true
fieldFlag()=true
localFromCall()=true
pick(7, true, true)=true
pick(0, false, false)=false
fromLocal(true)=true
fromLocal(false)=false
assignFromCall(true)=true
assignFromCall(false)=true
throughLocal(true)=true
throughLocal(false)=false
staticFlagCount()=1
intLocal(7)=1
intLocal(0)=0
count(true)=1
count(false)=0
nonzero(0)=0
nonzero(1)=1
answer()=1
```

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the
run that introduced this sample:

| member | run | content | wrapper | result |
| --- | --- | --- | --- | --- |
| `isZero(I)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `flag()Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `parity(I)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `passed(Z)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `callFlag()Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `fieldFlag()Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `localFromCall()Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `pick(IZZ)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `fromLocal(Z)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `assignFromCall(Z)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `throughLocal(Z)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `negated(Z)Z` | Mixed/Fallback | contains_statements | javac refuses (`missing return statement`) | boundary: 1 quoted BCI, refused regions [] |
| `staticFlagCount()I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `intLocal(I)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `count(Z)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `nonzero(I)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `answer()I` | Java/Structured | contains_statements | compiles | executed: traces identical |

- the member declarations the comparison derived from the run's own facts: `public static boolean isZero(int arg0)`,
  `public static boolean flag()`, `public static int parity(int arg0)`, `public static boolean passed(boolean arg0)`,
  `public static boolean callFlag()`, `public static boolean fieldFlag()`, `public static boolean localFromCall()`,
  `public static boolean pick(int arg0, boolean arg1, boolean arg2)`, `public static boolean fromLocal(boolean arg0)`,
  `public static boolean assignFromCall(boolean arg0)`, `public static boolean throughLocal(boolean arg0)`,
  `public static int staticFlagCount()`, `public static int intLocal(int arg0)`, `public static int count(boolean arg0)`,
  `public static int nonzero(int arg0)`, `public static int answer()`
- trace: 29 line(s), identical on both sides (the baseline's own calls plus the members called with
  the default input set)
- the wrapper extends the sample for `parity`, `callFlag`, `localFromCall` and `assignFromCall`,
  whose texts name the member `flag` the committed class declares

The **pre-fix** state of the members is recorded by the mutations: with the `Return` branch rendering
by value again, the comparison stops on
`GenisZero.java:8: error: incompatible types: int cannot be converted to boolean` (and the same at
line 10); with the condition recognizing only a `boolean` parameter it stops on
`Genparity.java:7: error: incomparable types: boolean and int`; and with the declaration reading no
boolean evidence it stops on
`p3-boolean-contexts/v8 …: localFromCall()Z is a body the run writes whole, and this run states Mixed`.
The pre-extension text of `throughLocal` compiles beside the sample to
`GenThroughLocal.java:3: error: incompatible types: boolean cannot be converted to int` and
`line 4: error: int cannot be converted to boolean`. None of these is a skip: the member's text must
compile under the declaration derived from the run's own facts.

## Reproducing

```text
cd tests/fixtures/p3-boolean-contexts
mkdir -p v8
javac --release 8 -g:none -d v8 BooleanContexts.java
shasum -a 256 v8/BooleanContexts.class   # e6a85f9aa40f585642696f0a391c8be9c8fafa2100e0cf0f9ab7ebd6bfb2bf0d
```

`Baseline.java` is not compiled into this directory: the comparison test compiles it in a temporary
directory with the sample on its classpath, so no driver class is ever committed beside the sample.
