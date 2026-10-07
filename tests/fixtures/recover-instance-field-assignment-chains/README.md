# `recover-instance-field-assignment-chains` fixtures

The copy family's **chain** shape as javac writes it for **instance** fields, on **both** compiler
legs. Each class was compiled from the same source by javac 23.0.1 with `--release 8` and by real
javac 8 (Corretto 1.8.0_432); both legs emit the identical instruction sequences (only
constant-pool indices differ), which is what makes the change's criterion a *bytecode* criterion
and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 CP.java MX.java NEG.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 CP.java MX.java NEG.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.

## Frozen hashes

```text
55d1f83e23d908737679e6d4a2775e06cb064758b7dfcc91ba4f9a35da8d1761  CP.java
07864db0c608a4343b211dd25ae9e6d83b0fd6960b436ae88bd5c5866604337d  MX.java
bd51c0e69f37970aa500e961783d9b864db644918b61e36ff36288414565bd8b  NEG.java
e975010eda77fd915663e0d7ba0f36c3e082c089289ac0197d0297605ca1ff00  v8/CP.class
1fb10a9c814c02793d9d0e1e8cc8e745ac8d9cd3264c7177dc36b92c19de9868  v8/MX.class
2805fa91afd121222e9c5cdfd779d195e094c2ca8527e1327dac9d5355ceace8  v8/NEG.class
f86d61401f009c7cd927221481930486327b25807d2f142d9c304281a3356d77  v8-javac8/CP.class
9738495451d32548207bfc30ef7c3b76e0c92b0a50db785062d8e69e9ec69419  v8-javac8/MX.class
78689e89327e624ace928cba4fbe460805e6c5eb7afc349afe1b21f077f7f45e  v8-javac8/NEG.class
```

The same bytes are recorded in `tests/fixtures/corpus-fingerprint.json` (blake3), which the P5
fingerprint test verifies.

## What each member is for, and the bytes that make it

`CP` — the shape this change presents:

```text
void inst();   // this.a = this.b = this.c = 5
     0: aload_0
     1: aload_0
     2: aload_0
     3: iconst_5
     4: dup_x1        <- inserts a copy of 5 UNDER the top receiver (slot 2)
     5: putfield c    <- receiver = the this the copy moved (slot 3), value = the copy on top
     8: dup_x1        <- inserts a copy of the surviving 5 under the next receiver
     9: putfield b
    12: putfield a    <- takes the surviving copy and the receiver that never left slot 0
```

→ the value is evaluated **once**, and the text is one assignment per store in bytecode order —
which is the source's own right-to-left order: `this.c = 5; this.b = 5; this.a = 5;`. Every store
is called on the same `this`: the three `aload_0`s read the one value slot 0's entry state holds.

```text
void pair();   // this.a = this.b = 7 — the two-store chain (one `dup_x1`, two stores)
```

`MX` — the **mixed chain**, a recorded boundary: a dance that carries a static store beside the
instance ones is not this change's shape, and its refusals are pinned verbatim.

```text
void mixStaticRight();  // this.a = this.b = MX.s = 5  — dup; putstatic s; dup_x1; putfield b; putfield a
void mixStaticLeft();   // MX.s = this.a = this.b = 6  — dup_x1; putfield b; dup_x1; putfield a; putstatic s
void mixStaticMid();    // this.a = MX.s = this.b = 7  — dup_x1; putfield b; dup; putstatic s; putfield a
```

`NEG` — the refusals:

```text
void cross();        // o1.a = o2.b = 5   — the two receivers are two different values
int  expr();         // this.a = (this.b = 5) + 1 — the copy's value feeds the `iadd`
void saved();        // this.a = this.b = this.c = f() — the source may not be written once per store
void holderChain();  // h().a = h().b = 5 — the receivers are call results, not `this`
```

Every one of them keeps the refusal it had (this change admits none of them); `MX` and `NEG`'s
stripped texts do not compile, which is the safe form the soundness invariant asks for.

## The behaviour both legs answer

```text
CP   : 5/5/5
       7/7/5
```

`tests/recover_instance_field_assignment_chains.rs`'s ignored replay strips `CP`'s comment lines,
compiles the result with `javac --release 8` and with real javac 8, runs both under `-Xverify:all`
and compares the answer with the committed class file's own — identical on both legs.
