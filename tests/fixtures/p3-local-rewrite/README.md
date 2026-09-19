# P3 1.3d fixture: a local slot written between a load and its reader

`v8/LocalRewrite.class` is a real compiled sample: the sibling `LocalRewrite.java` compiled by
**javac 23.0.1** (Oracle, `/Library/Java/JavaVirtualMachines/openjdk-23.0.1/Contents/Home/bin/javac`)
with

```text
javac --release 8 -g:none -d v8 LocalRewrite.java
```

The compiler prints its usual `源值 8 已过时` / `target value 8 is obsolete` warnings (3 of them) and
writes the class anyway. The compiler is a generation-only input: it is not checked in and no test
needs it at run time, so the sample is committed as bytes.

| property | value |
| --- | --- |
| class | `LocalRewrite` |
| class-file version | 52.0 (Java 8) |
| bytes | 650 |
| SHA-256 | `f755f062bc9779d941e93bf1ef4889b3ce6efb0dbd670e5ccbd07152126158a6` |
| debug attributes | none (`-g:none`), so every slot is named by its ordinal |
| read by | `tests/p3_local_rewrite.rs` (P3 1.3d regression, through `Engine::recover_method`) |

## What each member is for, and the bytes that make it

```text
public static int post(int);        // 1a 84 00 01 ac
     0: iload_0
     1: iinc          0, 1
     4: ireturn

public static int bump(int);        // 1a 04 60 3c 1b ac
     0: iload_0
     1: iconst_1
     2: iadd
     3: istore_0
     4: iload_0
     5: ireturn

public static int doubleIt(int);    // 1a 05 68 3c 1b ac
     0: iload_0
     1: iconst_2
     2: imul
     3: istore_0
     4: iload_0
     5: ireturn

public static int saved(int);       // 1a 84 00 01 3d 1c ac
     0: iload_0
     1: iinc          0, 1
     4: istore_1
     5: iload_1
     6: ireturn

public static int conditional(int); // 1a 84 00 01 9e 00 05 04 ac 03 ac
     0: iload_0
     1: iinc          0, 1
     4: ifle          9
     7: iconst_1
     8: ireturn
     9: iconst_0
    10: ireturn

public static int loopAcross(int, int);  // 1a 4d 1b 9e 00 0e 1a 04 60 3c 1b 04 64 3d a7 ff f2 1c ac
     0: iload_0
     1: istore_2
     2: iload_1
     3: ifle          17
     6: iload_0
     7: iconst_1
     8: iadd
     9: istore_0
    10: iload_1
    11: iconst_1
    12: isub
    13: istore_1
    14: goto          2
    17: iload_2
    18: ireturn

public static java.lang.String cast();   // b8 00 15 c0 00 13 b0
     0: invokestatic  #15   // make:()Ljava/lang/Object;
     3: checkcast     #19   // class java/lang/String
     6: areturn

static java.lang.Object make();
     0: getstatic     #7    // Field calls:I
     3: iconst_1
     4: iadd
     5: putstatic     #7    // Field calls:I
     8: ldc           #13   // String ok
    10: areturn
```

`post` is P3-R1: the load's value is on the operand stack when the `iinc` writes the slot, so the
slot's *name* at the `ireturn` denotes the incremented value and the honest answer is a quote, not
`return local0;`. `bump` and `doubleIt` are the control: the same write-then-read statement, whose
reload really does read the slot, must still be written with the slot's name. The two are textually
identical programs for *different* bytecode, which is why the regression has to be asserted on the
shapes and not on the text alone.

`saved` and `conditional` are the same disagreement at the two **other** consumption points of this
slice: `saved` stores the loaded value into another local after the increment (so `local1 = local0;`
would carry the incremented value, and the loss has to be quoted), and `conditional` tests the
loaded value in a branch (where the whole region is quoted, naming the branch and the load).
`loopAcross` is the opposite control: the value loaded before the loop is returned after the body
rewrote the slot, and it must **not** be refused — the name is read where the slot still holds it.

`cast` is P3-R2: a cast is a consumer the slice presents, and the invocation that produced the value
it casts (BCI 0) must stay in the answer — with the static counter in `make` saying how many times
the generated code calls it.

## Reproducing

```text
cd tests/fixtures/p3-local-rewrite
mkdir -p v8
javac --release 8 -g:none -d v8 LocalRewrite.java
shasum -a 256 v8/LocalRewrite.class   # f755f062bc9779d941e93bf1ef4889b3ce6efb0dbd670e5ccbd07152126158a6
```
