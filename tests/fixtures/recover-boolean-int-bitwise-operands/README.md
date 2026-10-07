# `recover-boolean-int-bitwise-operands` fixtures

The int-ified boolean operand of a **bitwise** expression — the `0`/`1` a `!` or a condition
materialises, and the `int` counter a `boolean r ^= x` accumulation lowers to — on the patrol's own
anchor and on **both** compiler legs. Each class was compiled from the same source by javac 23.0.1
with `--release 8` and by real javac 8 (Corretto 1.8.0_432); the two legs state the same instruction
shapes (only constant-pool indices differ, checked with `javap -c -p` and pool references blanked),
which is what makes the change's criterion a *bytecode* criterion and not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 BW.java BWR.java BWN.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 BW.java BWR.java BWN.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.

| file | sha256 (source) | sha256 (`v8/`) | sha256 (`v8-javac8/`) |
| --- | --- | --- | --- |
| `BW.java` / `BW.class` | `19174dc6b1501e39ffb9ae908a2740bb8b21e58f9ee157efb824132438dbc598` | `bb96006fd9099cb6388691fb60f5d501dbd60b3fad41b0f733d1b87ddddec691` | `17ac28b67a585d375c523f73ede1afe20c13de82a81c8aa30ad5bd9599af580d` |
| `BWR.java` / `BWR.class` | `9fd44205b89e3bcc71c4576954e6eb71da9a6407d30637ddc152f229046104ff` | `2ae4286cd9f8a72f3d9268616bb1134b0b3b5fe662fec481f4ec99cc5d6e3ba3` | `270e67404020fe0aad7e0347c42eaf810381e435d6901cbeea1e4aa2a53772cc` |
| `BWN.java` / `BWN.class` | `c4d7cca1d889c572b2fff39cae3fa47d71e0fe913a910cc2221ffc7fd70ed064` | `9fe393d50c00dd16e551a5328b49028fc2ced2c7a4cc67a7fc0056eabd4a7189` | `2c47d3ab3bbb3553e6c52ab5ffc8b511ce1bc82d122de3313e31729d545564d2` |

`patrol-BW.class` (the frozen patrol anchor, no source in this directory) is
`30fb5fcbe4e92c012ad7739071bb61bb6991064ac2835ee0113170c780a2a26b`.

## The anchor — the patrol's own `BW`

`BW.java` is a byte-for-byte copy of the patrol's frozen source
(`openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.java`) and
`patrol-BW.class` is that patrol's own frozen class, byte for byte (sha256
`30fb5fcbe4e92c012ad7739071bb61bb6991064ac2835ee0113170c780a2a26b`); `v8/BW.class` and
`v8-javac8/BW.class` are the same source recompiled on the two legs, and their instruction shapes are
the frozen class's own (checked with `javap -c -p` and pool references blanked). The class is the
acceptance anchor: its `main` prints `true/5/true/2/-2147483648/false/3`, and the recovered text must
print the same after a round trip through both compilers.

Two of its methods are the shapes the change reads back:

```text
mix:     static boolean mix(boolean[] f){ boolean r = false; for(boolean x : f) r ^= x; return r; }
         iconst_0; istore_1; …; iload_1; iload 5; ixor; istore_1; …; iload_1; ireturn
andNot:  static boolean andNot(boolean a, boolean b){ return a & !b; }
         iload_0; iload_1; ifne 9; iconst_1; goto 10; 9: iconst_0; 10: iand; 11: ireturn
```

The rest of the class is the **zero-regression** face the patrol recorded as healthy:
`boolean ^ boolean` (`xor`), `int ^ int` (`ixor`), the over-wide shift `x << 33` (`shConst`), the
folded constant shift `1 << 31` (`shFold`) and the Kernighan bit-count loop `x &= x-1` (`bits`).
Every one of them renders byte for byte as it did before the change.

## The positives — `BWR`

The change's own anchor class, the read-back's four admitted shapes:

```text
accXor:    boolean r = false; for (boolean x : f) r ^= x; return r ^ c;   // the counter read by a bitwise operator
andNotAnd: return (a & !b) & c;                                           // the materialisation read by a bitwise operator
orNot:     return a | !b;                                                 // the other eager boolean operator
notOr:     return !a | b;                                                 // the negation on the left
```

`accXor`'s counter is read by two positions the criterion admits (its own update's bitwise operator
and the `Z` return); `andNotAnd`'s inner materialisation is read by a bitwise operator whose own
value the `Z` return then reads.

## The negatives — `BWN`

Five shapes the change must **not** move, pinned verbatim by the tests:

| method | shape | what keeps the refusal |
| --- | --- | --- |
| `plusOne` | `return (b ? 1 : 0) + x;` | the materialised `0`/`1` is consumed by **arithmetic**: the position states an `int` |
| `andNotInt` | `return (a & !b) ? 1 : 0;` | the bitwise value is consumed by a **branch test**, not by a boolean return or a field write |
| `compareRead` | `boolean r = false; for (boolean x : f) r ^= x; return r == other;` | the counter is read by a **comparison** |
| `intSibling` | `return r ^ (b ? 1 : 0);` | the sibling operand is an `int`: no boolean evidence, and the `int` counter keeps its presentation |
| `passed` | `… r ^= x; System.out.println(r);` | the counter is read as a **call argument** |

`BWN`'s stripped text does not compile (two refused bodies carry no `return`), which is the soundness
invariant the refusals exist for: a body a reader cannot compile, never one that compiles and behaves
differently.
