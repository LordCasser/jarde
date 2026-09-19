# P3 3.1 fixture: where a local's declaration is written, and what a name is a name for

`v8/Scope.class` and `v8-debug/Scope.class` are real compiled samples: the sibling `Scope.java`
compiled **twice** by **javac 23.0.1** (Oracle,
`/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javac`) with

```text
javac --release 8 -g:none -d v8 Scope.java         # no LocalVariableTable: every name is an ordinal
javac --release 8 -g     -d v8-debug Scope.java    # the same bodies with the source names in an LVT
```

The compiler prints its usual `源值 8 已过时` / `target value 8 is obsolete` warnings (3 of them) and
writes the classes anyway. The compiler is a generation-only input: it is not checked in and no test
needs it at run time, so both samples are committed as bytes. The same source compiled to the same
bodies is what makes the two readable side by side: the *only* difference is the debug evidence.

| property | `v8/Scope.class` | `v8-debug/Scope.class` |
| --- | --- | --- |
| class | `Scope` | `Scope` |
| class-file version | 52.0 (Java 8) | 52.0 (Java 8) |
| bytes | 532 | 1101 |
| SHA-256 | `5badc5b105de3cb1a49f1dfcfe417b29d283c596aa06cecc233078538458789d` | `a93fdc7ff141daf5860c15106fb39ac860958cc8bb39ff44707ba36ac56a3d53` |
| debug attributes | none (`-g:none`) | `LocalVariableTable` + `LineNumberTable` + `StackMapTable` |
| read by | `tests/p3_scope.rs` (P3 3.1 regression, through `Engine::recover_method`) | the same file |

## What each member is for, and the bytes that make it

```text
public Scope();                    // 2a b7 00 01 b1   (the implicit constructor)

public static int scope(boolean);  // 1a 99 00 08 04 3c a7 00 05 05 3c 1b ac
     0: iload_0                     //   P3-R3: the slot is written in the then arm *and* in the
     1: ifeq          9             //   else arm and read after the join, so a declaration
     4: iconst_1                    //   written at the first write is out of scope twice
     5: istore_1
     6: goto          11
     9: iconst_2
    10: istore_1
    11: iload_1
    12: ireturn

public static int simple();        // 08 3c 1b ac
     0: iconst_5                    //   the control: one write and one read in one straight run,
     1: istore_0                    //   which must keep `int local0 = 5;`
     2: iload_0
     3: ireturn

public static int armOnly(int);    // 03 3c 1a 9e 00 0c 1a 04 60 3d 1c 3c a7 00 04 04 3c 1b ac
     0: iconst_0                    //   both directions at once: slot 1 is written in the prefix
     1: istore_1                    //   and in the arm and read after the join (hoisted), slot 2
     2: iload_0                     //   is written and read inside the then arm alone (declared
     3: ifle          15            //   where it is, with its value)
     6: iload_0
     7: iconst_1
     8: iadd
     9: istore_2
    10: iload_2
    11: istore_1
    12: goto          17
    15: iconst_1
    16: istore_1
    17: iload_1
    18: ireturn

public static int reuse(boolean, int);  // 1a 99 00 0c 1b 04 60 3d 1c 3a a7 00 09 1b 05 60 3d 1c 3a 2c ac
     0: iload_0                     //   slot reuse: the then arm's `c` and the else arm's `d` are
     1: ifeq          13            //   **one** slot (3), their scopes being disjoint — the `-g`
     4: iload_1                     //   sample's table names it `c` over one arm and `d` over the
     5: iconst_1                    //   other
     6: iadd
     7: istore_3
     8: iload_3
     9: istore_2
    10: goto          19
    13: iload_1
    14: iconst_2
    15: iadd
    16: istore_3
    17: iload_3
    18: istore_2
    19: iload_2
    20: ireturn

public static int after(long, int);     // 1c ac
     0: iload_2                     //   category-2: the `long` fills slots 0 and 1, so the `int`
     1: ireturn                     //   parameter is slot **2** — a count that stopped at the
                                    //   descriptor's parameter count would name it `arg1`

public static int reassign(long, int);  // 1c 04 60 3c 1c ac
     0: iload_2                     //   the same layout with a write: a miscount would declare
     1: iconst_1                    //   the parameter slot as a local of the body
     2: iadd
     3: istore_2
     4: iload_2
     5: ireturn

public long receiver(long);             // 2b ad
     0: lload_1                     //   a receiver below a category-2 parameter: `this` is slot 0
     1: lreturn                     //   and the `long` occupies 1 and 2
```

The `-g` sample's `LocalVariableTable` (the facts the presentation reads). Two of its records matter
to the rules that name slots:

```text
scope(boolean):     6   3  slot 1  x  I          armOnly(int):  10  2  slot 2  z  I
                    0  13  slot 0  b  Z                          0 19  slot 0  n  I
                   11   2  slot 1  x  I                          2 17  slot 1  y  I

reuse(boolean,int): 8   2  slot 3  c  I          after(long,int): 0  2  slot 0  a  J
                   10   3  slot 2  a  I                           0  2  slot 2  b  I
                   17   2  slot 3  d  I
                    0  21  slot 0  b  Z
                    0  21  slot 1  seed  I
                   19   2  slot 2  a  I
```

`scope`'s slot 1 carries **two records of one name** (`x`, over `[6,9)` and `[11,13)` — the two arms'
assignments of one source variable), so the slot has one name. `reuse`'s slot 3 carries **two records
of two names** (`c` and `d`), because two variables in disjoint scopes share one storage location:
one slot has one name in the produced text, and neither record's name is the truth about the whole of
it, so the presentation states none and falls back to the ordinal (`local3`). `armOnly`'s slot 2
(`z`) is named and used inside one arm, which is why its declaration stays there.

## The compiler control (the property the fixture exists for)

`tests/p3_scope.rs` asserts the produced text; the same text is also compiled and executed by hand in
the verification record of this slice, because "the body is in scope" is a statement about `javac`
rather than about the text. The shape of the control, over the artifact of `scope` alone:

```java
public final class Scope {
    public static int scope(int arg0) {
        int local1;              // hoisted above the branch: every use of the slot is inside it
        if (arg0 != 0) {
            local1 = 1;
        } else {
            local1 = 2;
        }
        return local1;
    }
}
```

The parameter is spelled `int` because the frames state one slot shape for the four int-sized
primitives: `boolean` vs `int` is not a distinction any statement of this body carries
(`crate::ast::Type`'s own boundary), and `javac` refuses `arg0 != 0` for a `boolean` parameter for
that reason — the *scope* property is what this control is about. Before the fix the same artifact
had the declaration inside the `then` arm (`int local1 = 1;`) and `javac --release 8` refused the
`else` arm's `local1 = 2;` and the join's `return local1;` with `找不到符号` / `cannot find symbol`,
which is what P3-R3 reported.

## Reproducing

```text
cd tests/fixtures/p3-scope
mkdir -p v8 v8-debug
javac --release 8 -g:none -d v8 Scope.java
javac --release 8 -g      -d v8-debug Scope.java
shasum -a 256 v8/Scope.class         # 5badc5b105de3cb1a49f1dfcfe417b29d283c596aa06cecc233078538458789d
shasum -a 256 v8-debug/Scope.class   # a93fdc7ff141daf5860c15106fb39ac860958cc8bb39ff44707ba36ac56a3d53
```
