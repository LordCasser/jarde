# `recover-array-initializer-value-positions` fixtures

The initialization dance's value at the consumption positions the six-position discriminator found
refusing — `partial[0] = new int[]{7}` (an element store into an array the body already had) and
`return new int[]{9}[0]` (a fresh array's immediate subscript) — on the patrol's own anchors and on
**both** compiler legs. Each class was compiled from the same source by javac 23.0.1 with
`--release 8` and by real javac 8 (Corretto 1.8.0_432); the two legs state the same shape (only
constant-pool indices differ), which is what makes the change's criterion a *bytecode* criterion and
not a compiler's habit.

| leg | directory | command |
| --- | --- | --- |
| javac 23.0.1 | `v8/` | `javac --release 8 -g -nowarn -d v8 MD.java MD2.java MD3.java AV.java` |
| Corretto 1.8.0_432 | `v8-javac8/` | `/Library/Java/JavaVirtualMachines/corretto-1.8.0_432/Contents/Home/bin/javac -g -nowarn -d v8-javac8 MD.java MD2.java MD3.java AV.java` |

Debug information is kept (`-g`) so every local the presentation names is the source's own name.

## The bytes, pinned

| file | sha256 |
| --- | --- |
| `AV.java` | `fa789607135618b11cfbd0fa785a8037009a58040c1260cc7101f5cfb8c7ec2d` |
| `MD.java` (the patrol's own) | `3cf80864b405fda84d0a5c2969ef9096992dbb89646a635e898168920fb569c3` |
| `MD2.java` (the patrol's own) | `0f2fbcd11056ade84d53f53d11aea269e8febadbb8d8856ede79260282f06e18` |
| `MD3.java` (the patrol's own) | `4499276db130a9c0aeec0f82c41144e383b7b234fd8fe62f86ecce09d6d1932f` |
| `AVN.class` (hand-built) | `e183cf5f745c207db8ad22bd0c4551c9214e96b2d082730ac0c3cd39cff039de` |
| `v8/{AV,MD,MD2,MD3}.class` | `6f04a622…`, `9d553081…`, `f811f7fc…`, `6cd8dfdd…` |
| `v8-javac8/{AV,MD,MD2,MD3}.class` | `772b876c…`, `9cad2228…`, `810126c2…`, `be5ed98a…` |

(The leg digests are the first eight hex digits of the full sha256; the sources, `AVN.class` and the
patrol's own three are stated whole because they are the inputs every claim here is about. The
integration test embeds the class files with `include_bytes!`, so a leg file that moves fails the
build's own corpus fingerprint before any of these assertions run.)

## The anchors — the patrol's own fixtures

`MD.java`, `MD2.java`, `MD3.java` are the `array-initializer-value-patrol` fixtures
(`openspec/evidence/java-syntax-2026-10-05/array-initializer-value-patrol/fixture/`), byte for byte.
They carry the six-position discriminator the change is accepted against:

| position | member | the change |
| --- | --- | --- |
| local store | `MD2.localPos` | presented before, and after |
| field write | `MD2.fieldPos` | presented before, and after |
| call argument | `MD2.argPos` | presented before, and after |
| outer initializer's element | `MD.jagged`, `MD.mkJagged` | presented before, and after |
| **element store RHS** | `MD.partSet` | refused before (`the copy at BCI 7 has no proved local assignment` ×2), recovered after |
| **immediate subscript** | `MD2.retPos`, `MD3.bareIdx2` | refused before, recovered after |

`MD2.retPos` is the patrol fixture's own comment calling that member 返回位; the bytecode is the
immediate-subscript position (`…; iastore; iconst_0; iaload; ireturn`), and it refuses at the parent
commit exactly as `MD3.bareIdx2` does. The four positions that *were* presented are pinned byte for
byte by the integration test, and so are the bare immediate consumption (`MD3.bareIdx` — a
`newarray` with no dance at all — and `MD3.bareRet`) and the sawtooth family (`MD.jagged`,
`MD.mkJagged`).

Each class's own `main` prints its answers, which the recovered text must print too:

| class | answer |
| --- | --- |
| `MD` | `21/9/9/10` |
| `MD2` | `3/3/4/5` |
| `MD3` | `2/9/3` |

## `AV` — the change's own class

`AV` states the same consumption positions one step further, with one call per `main` statement
(a single twenty-argument concatenation chain is past the concatenation rule's bounded dependency
closure for reasons this fixture is not about):

```text
elemStore:     p[0] = new int[]{7}; return p[0][0];       the element-store anchor
immIdx:        return new int[]{9}[0];                    the immediate-subscript anchor
localPos/fieldPos/argPos/outerPos                          the four positions, unchanged
bareIdx/bareRet/jagged/foreach                             bare consumption and the sawtooth family
immLen:        new int[]{4, 5, 6}.length                  the length receiver
immIdxVar:     new int[]{9}[i]                            a variable subscript
immIdxExpr:    new int[]{9, 8}[i + 1]                     a computed subscript
immIdxSum:     new int[]{9}[0] + 1                        the reader inside a larger expression
twoIdx:        new int[]{9}[0] + new int[]{8}[0]          two dances, one reader each
twoStores:     p[0] = …; p[1] = …;                        two element stores in one body
nestedIdx:     new int[][]{new int[]{9}}[0][0]            a child initializer, read immediately
condIdx:       new int[]{9}[0] > 0                        the reader as a condition's operand
immIdxInCall:  sum(new int[]{9}) + new int[]{1}[0]        a call beside a subscript
order:         new int[]{mark(1)}[mark(0)]                an effectful element beside an effectful index
```

`order` is the one that pins the *evaluation order*: the bytecode builds the array (running `mark(1)`)
before it produces the index (`mark(0)`), and the text states the same order. Its answer, and the
`calls` counter `main` prints last, are what the replay compares.

## `AVN` — the hand-built negatives

`javac` emits no dance value with a second consumer, so those shapes are assembled byte by byte by
[`build_avn.py`](build_avn.py) (`python3 build_avn.py`, which writes `AVN.class` beside it). One
class, three static `()I` members over the same dance
(`iconst_1; newarray int; dup; iconst_0; bipush 9; iastore`):

| member | what follows the dance | why it is here |
| --- | --- | --- |
| `single` | `iconst_0; iaload; ireturn` | the builder's self-test: the bytes really are the anchor's shape (the presentation writes `return new int[]{9}[0];`) |
| `twoReaders` | `dup; iconst_0; iaload; arraylength; iadd; ireturn` | the dance's value has two readers; refused, byte for byte as before |
| `discarded` | `dup; pop; iconst_0; iaload; ireturn` | a second consumer that reads nothing; refused by the consumer admission, byte for byte as before |

`discarded` is the negative the *gate* refuses (its `dup` is not a consumption position), and its
quote names every instruction of the dance (BCIs 1, 3, 7, 8, 9, 12), which is the accounting
observation the test asserts: a dance the text neither renders nor quotes would be an effect it
silently drops. `twoReaders` is refused one layer below the recovery rules (the member's analysis
does not complete: `ir_frame_inconsistent`), which is where a stack value with two *readers* is
answered.

## The tests

`tests/recover_array_initializer_value_positions.rs` pins the two anchors, the unmoved positions
byte for byte, the two negatives' refusals verbatim, and replays every fixture (both legs, both
compilers, `-Xverify:all`) against the fixture's own class files.
