# `recover-dup-store-conditional` fixtures

The dup-store dance's own targets, on **both** compiler legs. Each class was compiled from the same
source by javac 23.0.1 with `--release 8` and by real javac 8 (Corretto 1.8.0_432); both legs emit
the identical instruction sequences (only constant-pool indices differ), which is what makes the
change's criterion a *bytecode* criterion and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 DS.java REF.java NEG.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 DS.java REF.java NEG.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name; the
patrol anchors (`OP2`, `AC`) were compiled with javac's default and are rendered from the patrols'
own jars, not from here.

## What each member is for, and the bytes that make it

`DS` — the int form:

```text
static boolean condAssign(int x);   // 1a 04 60 59 3c 9e 00 07 04 ac 03 ac
     0: iload_0
     1: iconst_1
     2: iadd
     3: dup              <- the copy: one to the store, one to the test
     4: istore_0
     5: ifle          12
     8: iconst_1
     9: goto          13
    12: iconst_0
    13: ireturn
```
→ the target has **no reader**: the store is eliminated, `return x + 1 > 0;`.

```text
static int deadLocal(int x);        // a *local* target with no reader: same elimination
static boolean deadChain(int x);    // the dance inside a short-circuit chain, target dead
```
→ `if (x + 1 > 0) { … }` and `boolean b = x + 1 > 0 && x > 10;`.

```text
static int split(int x);            // 1a 04 60 59 3c 9e 00 05 1a ac 02 ac
     0: iload_0
     1: iconst_1
     2: iadd
     3: dup
     4: istore_0
     5: ifle          10
     8: iload_0          <- the target *is* read later
     9: ireturn
    10: iconst_m1
    11: ireturn
```
→ the split form: `x = x + 1; if (x > 0) { return x; } else { return -1; }`.

```text
static int splitLocal(int x);       // the same shape with a *local* target
```
→ the control: the copy family's assignment rule already presents a local target with a later reader
as the in-place assignment expression (`if ((y = x + 1) > 0) { … }`), and this change leaves that
text byte-identical.

`REF` — the reference form, the classic Java IO idiom:

```text
static int deadLine();              // b8 .. 59 4d c6 00 09 84 00 01 a7 ff f6 1a ac
     0: iconst_0
     1: istore_0
     2: invokestatic  read:()Ljava/lang/String;
     5: dup              <- the copy: one to the store, one to the null test
     6: astore_1
     7: ifnull        16
    10: iinc          0, 1
    13: goto           2
    16: iload_0
    17: ireturn
```
→ `while (read() != null) { n = n + 1; }` — the same dance, a different type and branch opcode.

`NEG` — the two refusals this change keeps:

```text
static int liveLine();              // the same loop with a *read* target: `n += line.length();`
static boolean shortChain(int x);   // `boolean b = (x = x + 1) > 0 && x > 10; return b;`
```
→ both quoted whole. The first is this change's own boundary (the split cannot be written in front of
a re-evaluated condition); the second is refused by the short-circuit chain's own proof, exactly as
it was before this change — a parameter target in a chain step is not a position this slice proves,
because the chain proof refuses those shapes first.

## SHA-256

| file | SHA-256 |
| --- | --- |
| `DS.java` | `e29debc5946489f21a7d1303dbdfa0f6db4258dbac7057c22dda5dceada45e7f` |
| `REF.java` | `c3c52119d1629207ff096d9d3b66e7078662e3819436435d580bd1e96175e0d0` |
| `NEG.java` | `6f36c90b0944a42aabbd828ed07b53a4640e0f0674982834d2cfd31adf3a7c46` |
| `v8/DS.class` | `7f418881f11a7b05a119e26196d852c3c59b6b96e7de66a9846fe83b74af8d5d` |
| `v8/REF.class` | `fcbf5e95ccf1ae5f1f78ccfaff4217b1ce591c93aa0cd6e0c320cd58663c6557` |
| `v8/NEG.class` | `d0c346143898663b9fdf7da4101da197dc786829826487f9251424ce0f282496` |
| `v8-javac8/DS.class` | `dec1f24de0a02a36da7339802ce5e70ee6e5c96f46aa972c8674500159a51ccb` |
| `v8-javac8/REF.class` | `939137d1810cfb0c511b307ac391797adcc34e47e37846064743224c3079e6b7` |
| `v8-javac8/NEG.class` | `49e2bac112c0c56ca32b7b062f5941984c2713584c4598d41dbde42ec97bdd80` |

## Reproducing

```text
cd tests/fixtures/recover-dup-store-conditional
javac --release 8 -g -nowarn -d v8 DS.java REF.java NEG.java
/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 DS.java REF.java NEG.java
```

## The multi-reader negative

The copy family's frozen multi-reader control is CF-06's `ExtraCopy.class`
(`tests/fixtures/p3-inner-assignment/ExtraCopy.class`, SHA-256
`8bf9c555834e310c920073cb59405ea10a5ca831453e57616617ae0ba54fbf9e`: BCI 13's `iconst_5` patched to
`dup`, so the first copy feeds another copy rather than the test).
`tests/recover_dup_store_conditional.rs` reads it from the fixture that owns it and pins its
refusal; it is not copied here.
