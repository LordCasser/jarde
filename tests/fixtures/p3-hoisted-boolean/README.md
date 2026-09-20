# P3 stage: one local's type is decided once, before its statements are built

`v8/HoistedBoolean.class` is a real compiled sample: the sibling `HoistedBoolean.java` compiled by
**javac 23.0.1** (OpenJDK 23.0.1, `/usr/bin/javac`) with

```text
javac --release 8 -g:none -d v8 HoistedBoolean.java
```

The compiler prints its usual `警告: [options] 源值 8 已过时，将在未来发行版中删除` /
`目标值 8 已过时` / `要隐藏有关已过时选项的警告, 请使用 -Xlint:-options.` warnings (3 of them) and
writes the class anyway, exit code 0. The compiler is a generation-only input: it is not needed at
run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `HoistedBoolean` |
| class-file version | 52.0 (Java 8) |
| bytes | 743 |
| SHA-256 | `b9184230883dd94ad388713196beabd266907b02fa20b00499706092d64d1640` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_hoisted_boolean.rs` (the exact texts, the two orders of one shape, the two refusals, and the plan's own billing) and `tests/p3_execution_comparison.rs` (six members compiled under a declaration derived from the run's own facts and executed against the original, the two refusals recorded as boundaries) |

## What the defect is

A local's type used to be decided **twice**, by two paths that read different evidence:

* the **hoisted** declaration path (`crates/jarde-java/src/build.rs::declarations`) runs before any
  statement of the body exists and read the first write's stored value through the class's own
  descriptors only (`boolean_proof`: a `Z` parameter's load, a call whose callee descriptor returns
  `Z`, a field read whose descriptor is `Z`);
* the **in-place** declaration path (`declare`) runs while the statements are built and also
  recognized "a read of a local this body already declared `boolean`" — because by then the
  declaration it had to see was already written.

The same value therefore got two answers, and which one a variable got depended on whether its
declaration was hoisted — that is, on the order the regions were walked in. `copied` and `swapped`
are the two orders of one shape, and the pre-fix texts are the measurement:

| member | pre-fix recovered text (measured) | `javac --release 8` |
| --- | --- | --- |
| `copied(ZI)I` | `int local3; boolean local2 = arg0; if (arg1 == 0) { local3 = local2; } else { local3 = arg0; } if (local3 != 0) { return 1; } else { return 0; }` | `error: incompatible types: boolean cannot be converted to int` (twice: on `local3 = local2;` and on `local3 = arg0;`) |
| `swapped(ZI)I` | `boolean local3; boolean local2 = arg0; if (arg1 == 0) { local3 = arg0; } else { local3 = local2; } if (local3) { … }` | compiles — the **same shape**, only the arms exchanged |
| `relayed(Z)Z` | `int local3; int local2; boolean local1 = arg0; if (arg0) { local2 = local1; local3 = local2; } else { … }` and the `return` at BCI 18 refused | `error: incompatible types: boolean cannot be converted to int` (twice) and `int cannot be converted to boolean` on the return |
| `conflicted(ZI)I` | `int local2; if (arg1 == 0) { local2 = 1; } else { local2 = arg0; } if (local2 != 0) { … }` | `error: incompatible types: boolean cannot be converted to int` on `local2 = arg0;` |

Every one of those runs reported `Java`/`Structured`/`contains_statements`/`complete` (except
`relayed`, which reported the return refusal) with no diagnostic: the artifact was structurally whole
and its values were typed wrongly. The structural planes cannot see a type, which is why the
acceptance is the text plus a compiler plus the two sides' traces.

The fix this sample pins is one decision per variable, taken in the existing declaration planning,
**before the statements are built**, indexed by the existing `LocalVariable` identity, and consumed by
the hoisted declaration, the in-place declaration, every assignment, every condition and every `Z`
return. `declare()`'s `Ok(None)`, which used to mean both "no declaration is due" and "the
declaration failed and a fallback was already written", is split into `Declared`/`NotDue`/`Refused`,
and after `Refused` neither the assignment nor the declaration is written.

## The members

| member | descriptor | what it pins |
| --- | --- | --- |
| `copied` | `(ZI)I` | the review's sample: a hoisted local (`c`) whose write in one arm copies a local the body declared `boolean` and whose write in the other copies the `Z` parameter |
| `swapped` | `(ZI)I` | the same shape with the arms exchanged — the write the bytecode reaches first is the descriptor-proven one, so a first-write-only reading still answers differently unless the decision is taken for the variable |
| `relayed` | `(Z)Z` | a copy chain (`x` → `y` → `z`): `z`'s evidence arrives only after `y`'s own decision, so the propagation has to reach a fixpoint |
| `literalArmed` | `(Z)I` | the recorded boundary: the only values written are `true`/`false`, which is how both a boolean and an `int` are pushed, so the frames' `int` decides and the text stays an `int` local |
| `fromParameter` | `(ZI)Z` | the descriptor-only positive control (the predecessor change's `pick` shape) |
| `intLocal` | `(I)I` | the pure-`int` control: a `0`/`1` store is not a boolean |
| `unproven` | `(Z)Z` | the same body as `literalArmed` under a `Z` descriptor: the position requires a boolean the evidence does not have — refused |
| `conflicted` | `(ZI)I` | a write that cannot be spelled as the type the variable's own **first** write decided (a literal decides `int`, the other arm stores a `Z` parameter's load) — refused |

## The evidence the decision reads, and what it does not

| evidence | initiates a boolean decision? | where the fact is read |
| --- | --- | --- |
| a `load` of a parameter slot whose descriptor is `Z` | yes | `MethodFacts::parameter_types` (P3-R5), and `boolean_proof` for such a load |
| the result of a call whose callee descriptor returns `Z` | yes | `CallTarget::descriptor()` |
| a field access a `field@1` claim states is `Z` | yes | the claim's own evidence (`field::Evidence::descriptor`) |
| a read of a **local this plan decided `boolean`** | yes, one read and one hop | the propagation (`decide_types`' worklist) |
| a `0`/`1` literal | **no** | the frames state `int` for it; it adapts to `true`/`false` only where the target is already decided boolean |
| anything else — an arithmetic result, a comparison, a merge of unproven parts, a load of a variable no evidence proves | no | the frames' own type when they state one, and otherwise the plan decides no type and the structure is refused |

Two boundaries are deliberate and are part of this sample:

* **No value chain is followed.** The propagation is one read, one hop: the value a write stores has
  to *be* a read of a variable the plan decided boolean. A store's own value, a value two
  definitions away and a value merged out of several pushes state nothing.
* **The first write states the type, and every write is checked against it.** A variable whose first
  write is a literal keeps the frames' `int` however boolean its later writes look (`conflicted` and
  `literalArmed`), and every later write that cannot be spelled as that type terminates the structure
  it belongs to (`conflicted`) — the same rule read in the other direction (`boolean local1; … local1
  = 2;`) was already a refusal before this change.

## The bytecode of every member

```text
public HoistedBoolean();  // 2a b7 00 01 b1
     0: aload_0
     1: invokespecial #1                  // Method java/lang/Object."<init>":()V
     4: return

public static int copied(boolean, int);  // 1a 3d 1b 9a 00 08 1c 3e a7 00 05 1a 3e 1d 99 00 05 04 ac 03 ac
     0: iload_0
     1: istore_2
     2: iload_1
     3: ifne          11
     6: iload_2
     7: istore_3
     8: goto          13
    11: iload_0
    12: istore_3
    13: iload_3
    14: ifeq          19
    17: iconst_1
    18: ireturn
    19: iconst_0
    20: ireturn

public static int swapped(boolean, int);  // 1a 3d 1b 9a 00 08 1a 3e a7 00 05 1c 3e 1d 99 00 05 04 ac 03 ac
     0: iload_0
     1: istore_2
     2: iload_1
     3: ifne          11
     6: iload_0
     7: istore_3
     8: goto          13
    11: iload_2
    12: istore_3
    13: iload_3
    14: ifeq          19
    17: iconst_1
    18: ireturn
    19: iconst_0
    20: ireturn

public static boolean relayed(boolean);  // 1a 3c 1a 99 00 0a 1b 3d 1c 3e a7 00 07 1b 3d 1c 3e 1d ac
     0: iload_0
     1: istore_1
     2: iload_0
     3: ifeq          13
     6: iload_1
     7: istore_2
     8: iload_2
     9: istore_3
    10: goto          17
    13: iload_1
    14: istore_2
    15: iload_2
    16: istore_3
    17: iload_3
    18: ireturn

public static int literalArmed(boolean);  // 1a 99 00 08 04 3c a7 00 05 03 3c 1b 99 00 05 04 ac 03 ac
     0: iload_0
     1: ifeq          9
     4: iconst_1
     5: istore_1
     6: goto          11
     9: iconst_0
    10: istore_1
    11: iload_1
    12: ifeq          17
    15: iconst_1
    16: ireturn
    17: iconst_0
    18: ireturn

public static boolean fromParameter(boolean, int);  // 1b 9a 00 08 1a 3d a7 00 05 1a 3d 1c ac
     0: iload_1
     1: ifne          9
     4: iload_0
     5: istore_2
     6: goto          11
     9: iload_0
    10: istore_2
    11: iload_2
    12: ireturn

public static int intLocal(int);  // 03 3c 1a 9a 00 08 04 3c a7 00 05 1a 3c 1b ac
     0: iconst_0
     1: istore_1
     2: iload_0
     3: ifne          11
     6: iconst_1
     7: istore_1
     8: goto          13
    11: iload_0
    12: istore_1
    13: iload_1
    14: ireturn

public static boolean unproven(boolean);  // 1a 99 00 08 04 3c a7 00 05 03 3c 1b ac
     0: iload_0
     1: ifeq          9
     4: iconst_1
     5: istore_1
     6: goto          11
     9: iconst_0
    10: istore_1
    11: iload_1
    12: ireturn

public static int conflicted(boolean, int);  // 1b 9a 00 08 04 3d a7 00 05 1a 3d 1c 99 00 05 04 ac 03 ac
     0: iload_1
     1: ifne          9
     4: iconst_1
     5: istore_2
     6: goto          11
     9: iload_0
    10: istore_2
    11: iload_2
    12: ifeq          17
    15: iconst_1
    16: ireturn
    17: iconst_0
    18: ireturn
```

`copied` and `swapped` are byte-for-byte the same instruction sequence with BCIs 6–7 and 11–12
exchanged: `copied` stores `iload_2` (`a`) in the arm the source wrote first, `swapped` stores
`iload_0` (`b`) there. That one exchange is what used to decide the variable's type.

## What each member recovered, before and after

| member | before | after |
| --- | --- | --- |
| `copied(ZI)I` | `int local3; boolean local2 = arg0; if (arg1 == 0) { local3 = local2; } else { local3 = arg0; } if (local3 != 0) { return 1; } else { return 0; }` | `boolean local3; boolean local2 = arg0; if (arg1 == 0) { local3 = local2; } else { local3 = arg0; } if (local3) { return 1; } else { return 0; }` |
| `swapped(ZI)I` | `boolean local3; … if (arg1 == 0) { local3 = arg0; } else { local3 = local2; } if (local3) { … }` (compiles) | unchanged — the two orders now agree |
| `relayed(Z)Z` | `int local3; int local2; boolean local1 = arg0; …` with the return at BCI 18 refused (`Mixed`/`Fallback`) | `boolean local3; boolean local2; boolean local1 = arg0; … return local3;` (`Java`/`Structured`) |
| `literalArmed(Z)I` | `int local1; if (arg0) { local1 = 1; } else { local1 = 0; } if (local1 != 0) { … }` | unchanged (the recorded boundary) |
| `fromParameter(ZI)Z` | `boolean local2; if (arg1 == 0) { local2 = arg0; } else { local2 = arg0; } return local2;` | unchanged |
| `intLocal(I)I` | `int local1; local1 = 0; if (arg0 == 0) { local1 = 1; } else { local1 = arg0; } return local1;` | unchanged |
| `unproven(Z)Z` | `int local1; if (arg0) { local1 = 1; } else { local1 = 0; }` + the return at BCI 12 refused | unchanged |
| `conflicted(ZI)I` | `int local2; if (arg1 == 0) { local2 = 1; } else { local2 = arg0; } if (local2 != 0) { … }` (`Java`/`Structured`, javac refuses it) | `int local2; if (arg1 == 0) { local2 = 1; } else { …refused… } if (local2 != 0) { … }` |

### The two refusals

Both keep the layer's existing contract: the artifact is `Mixed`/`Fallback`, its text quotes the
bytecode it could not present, the index stays anchored in the segment table, and the reason names
the member, the BCI and the fact that stopped it.

`conflicted(ZI)I` — the store at BCI 10 is quoted, and the assignment it would have carried is not
written (before this change the same refusal was followed by `local2 = arg0;`):

```text
{
    int local2;
    if (arg1 == 0) {
        local2 = 1;
    } else {
        // @bytecode 10
        // the value at BCI 10 is stored into `local2`, which this run decided holds `int`, and this layer presents the value as a boolean (a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this run decided `boolean`): the `boolean` spelling this layer would write is text the variable's own type rejects
    }
    if (local2 != 0) {
        return 1;
    } else {
        return 0;
    }
}
```

`unproven(Z)Z` — the return at BCI 12 is quoted:

```text
{
    int local1;
    if (arg0) {
        local1 = 1;
    } else {
        local1 = 0;
    }
    // @bytecode 12
    // the value at BCI 12 is returned from a method whose own descriptor returns `Z`, and this layer has no evidence that the value is a boolean (a `0`/`1` literal, a `boolean` parameter's load, the result of a call whose callee descriptor returns `Z`, a claimed field read whose descriptor is `Z`, or a local this body declared `boolean`): the `int` spelling this layer would write is text the member's own signature rejects
}
```

The same rule for a declaration that cannot be typed at all (the frame states no type, or states a
descriptor this layer cannot spell) is pinned by `tests/p3_array_types.rs`'s
`a_descriptor_that_states_no_java_type_is_refused_at_its_bci`: `elementless`'s store is refused, its
bytecode is quoted, and no assignment follows it — the shape `declare()`'s split result exists for.

## Baseline driver

`Baseline.java` is compiled beside the sample by the ignored comparison test and run as its own
program: these are the original class's own answers for the input set the comparison states, and they
are the values the generated side must return (`copied`/`swapped` are called with both values of `b`
and with `n` on either side of their branch, `relayed` with both values of `b`).

```text
copied(true, 0)=1
copied(false, 0)=0
copied(true, 7)=1
copied(false, -1)=0
swapped(true, 0)=1
swapped(false, 0)=0
swapped(true, 7)=1
swapped(false, -1)=0
relayed(true)=true
relayed(false)=false
literalArmed(true)=1
literalArmed(false)=0
fromParameter(true, 0)=true
fromParameter(false, 7)=false
fromParameter(true, -1)=true
intLocal(7)=7
intLocal(0)=1
intLocal(-1)=-1
```

## What the comparison answered

`cargo test --test p3_execution_comparison --locked -- --ignored --nocapture` on javac 23.0.1, the
run that introduced this sample:

| member | run | content | wrapper | result |
| --- | --- | --- | --- | --- |
| `copied(ZI)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `swapped(ZI)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `relayed(Z)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `literalArmed(Z)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `fromParameter(ZI)Z` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `intLocal(I)I` | Java/Structured | contains_statements | compiles | executed: traces identical |
| `unproven(Z)Z` | Mixed/Fallback | contains_statements | javac refuses (`missing return statement`) | boundary: 1 quoted BCI, refused regions [] |
| `conflicted(ZI)I` | Mixed/Fallback | contains_statements | javac refuses (`variable local2 might not have been initialized`) | boundary: 1 quoted BCI, refused regions [] |

- the member declarations the comparison derived from the run's own facts: `public static int
  copied(boolean arg0, int arg1)`, `public static int swapped(boolean arg0, int arg1)`, `public
  static boolean relayed(boolean arg0)`, `public static int literalArmed(boolean arg0)`, `public
  static boolean fromParameter(boolean arg0, int arg1)`, `public static int intLocal(int arg0)`,
  `public static boolean unproven(boolean arg0)`, `public static int conflicted(boolean arg0, int
  arg1)`
- trace: 18 line(s), identical on both sides
- the wrapper of a refused member does not compile, and it is not meant to: the text holds a quoted
  region, and the two messages above are the compiler's own statement of what the quote left out

The **pre-fix** state of the members is recorded by the change's two mutations, both measured on this
sample before it was committed:

* with the hoisted path proving from the descriptors only (`boolean_proof`, as it did before the plan
  carried the decision), `tests/p3_hoisted_boolean.rs` fails three of its eight cases on the pre-fix
  text (`int local3; boolean local2 = arg0; … if (local3)`) and the comparison stops on
  ``p3-hoisted-boolean/v8 (javac 23.0.1, --release 8 -g:none): the text of `copied(ZI)I` must compile
  under the declaration this file derives from the run's facts; javac refused it: … Gencopied.java:10:
  error: incompatible types: boolean cannot be converted to int`` (and the same at line 12, plus
  `int cannot be converted to boolean` at line 14 for the condition);
* with the per-write consistency check reading only the first write (the conflicting store published
  as it is), `tests/p3_hoisted_boolean.rs` fails
  `a_write_that_cannot_be_spelled_as_the_decided_type_terminates_its_structure`
  (`` `conflicted`: … left: Java right: Mixed``, over the text `int local2; … local2 = arg0; …`) and
  the comparison stops on
  ``p3-hoisted-boolean/v8 …: `conflicted(ZI)I` keeps quoted bytecode, and this run states Java``.

Neither mutation is a `skip`: the member's text has to compile under the declaration derived from the
run's own facts, and a refusal has to be the refusal the sample records. Both mutations were restored
byte-identically (`shasum -a 256 crates/jarde-java/src/build.rs`), and the pre-fix texts themselves are
the table above; the pre-fix `javac` refusals were measured by wrapping those texts in the member's own
declaration (`boolean cannot be converted to int` twice for `copied`, once for `conflicted`; two of
them plus `int cannot be converted to boolean` for `relayed`; none for `swapped`).

## Reproducing

```text
cd tests/fixtures/p3-hoisted-boolean
mkdir -p v8
javac --release 8 -g:none -d v8 HoistedBoolean.java
shasum -a 256 v8/HoistedBoolean.class   # b9184230883dd94ad388713196beabd266907b02fa20b00499706092d64d1640
```

`Baseline.java` is not compiled into this directory: the comparison test compiles it in a temporary
directory with the sample on its classpath, so no driver class is ever committed beside the sample.
