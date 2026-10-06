# `recover-chained-field-assignment` fixtures

The copy family's third shape, on **both** compiler legs. Each class was compiled from the same
source by javac 23.0.1 with `--release 8` and by real javac 8 (Corretto 1.8.0_432); both legs emit
the identical instruction sequences (only constant-pool indices differ), which is what makes the
change's criterion a *bytecode* criterion and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 CF.java NEG.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 CF.java NEG.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.

## Frozen hashes

```text
a0559d64402243f303b0203716ca27b49d444b036cb40eeec32dbad7d4c6026a  v8/CF.class
03ef02b7862c5993beac942223c572b4ea228e5724019940727b580d57220cbc  v8/NEG.class
e9f42bbbfec646904fae3a137809d409ede79bb7664a60fd5da6c4e997a3995e  v8-javac8/CF.class
061d3aee471cb8a5275cb1ac011cbe59db9f86b10a79fc4ff28a6ddee6f978d9  v8-javac8/NEG.class
4e0de647ac04d05a757a8bc14d3ab8e90c8d4eb1942833e86564cf60ba295ec1  CF.java
7711af550bedcbcab97769281c3e496d9d9dd1c980352937969abe11207bd11e  NEG.java
```

## What each member is for, and the bytes that make it

`CF` — the shapes this change presents:

```text
static void chain();      // CF.sa = CF.sb = CF.sc = 5
     0: iconst_5
     1: dup              <- the first copy: one store takes it, the next copy reads it
     2: putstatic     sc
     5: dup              <- the second copy: both stores take one each
     6: putstatic     sb
     9: putstatic     sa
```
→ the value is evaluated **once**, and the text is one assignment per store in bytecode order —
which is the source's own right-to-left order: `CF.sc = 5; CF.sb = 5; CF.sa = 5;`.

```text
static void pair();       // CF.sa = CF.sb = 7 — the two-store chain
static void call();       // CF.sa = CF.sb = CF.sc = f() — the source may not be written once
                          // per store: the lead saves it (`int saved0 = f(); …`)
```

```text
void enable(int bit);     // flags |= 1 << bit
     0: aload_0
     1: dup              <- one copy is the read's receiver, the other the write's
     2: getfield      flags
     5: iconst_1
     6: iload_1
     7: ishl
     8: ior
     9: putfield      flags
```
→ `this.flags = this.flags | 1 << bit;` — the operator the update rule does not present keeps the
receiver copy's own presentation.

```text
void disable(int bit);    // flags &= ~(1 << bit) — the same shape with `iand`/`ixor`
CF add(String x);         // field += "[" + x + "]" — the receiver copy is a `dup_x1` under the
                          // builder, and the concatenation folds back to `+`
```
→ `this.field = this.field + "[" + x + "]";`.

```text
static void single();     // the control: one field assignment, no copy at all
static int chainLocal();  // the control: the *local* chain's own text (`x = y = 7`)
static void sAdd(String); // the control: the static String compound, whose presentation the
                          // static form already had (an explicit builder chain, no receiver copy)
```

`NEG` — the refusals:

```text
static int mixed();         // int x; x = sa = 7  — the second copy feeds a *local* store
static int expr();          // sa = (sb = 5) + 1   — the second copy is consumed by the `iadd`
static int arrayConsumer(); // arr[0] = sa = 3    — the second copy feeds an array store
static void callReceiver(); // holder().ia |= 1   — the receiver copy's source is a call, so the
                            // text would evaluate it twice
```

Every one of them keeps the refusal it had: the copy is quoted whole, and the stripped text does not
compile — which is the safe form the soundness invariant asks for.

## The behaviour both legs answer

```text
CF   : 9/9/9/14/9/f[x][y]
       1/s[z]/0
NEG  : 7/5/3/3
       1
```
