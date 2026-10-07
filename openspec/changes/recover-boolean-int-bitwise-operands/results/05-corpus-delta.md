# 3.1/3.2 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`a11573c6`, a separate worktree at `/tmp/jarde-bwslice-baseline`) and this change's — and diffs the
two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2822 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (739
  candidates).

Self-tests first: the patrol's frozen `BW.class` drops from two refusals to zero with both anchors
written (`andNot` → `return arg0 & !arg1;`, `mix` → `local1 = local1 ^ local5;`), this change's `BWN`
refusals are byte-identical, and the three precedent families' own controls are byte-identical
(`recover-conditional-rhs-field-compound`'s `BI`/`RC`/`RCN`,
`recover-chained-field-assignment`'s `CF`/`NEG`,
`recover-inline-conditional-concat-operands`'s `ICM`/`ICN`). Then the counts.

```
SELF-TEST OK: BW refusals 2 -> 0 with both anchors; BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical
pass A loose candidate classes: 2822
pass A: moved=6 unrendered=1
archive candidate classes: 739
pass C: moved=0 unrendered=1
moved classes: single-class=6 jar=0 total=6
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are the same `package-info` entries every precedent sweep printed
(a `package-info.class` states no class name a request can bind), the same on both binaries — they
are printed, not silently dropped.

## The six moved classes, classified

Every moved class is the change's own fixture or its own anchor, and every one of them moves in the
**same single class of delta**: the `int` local a `boolean` accumulation lowered to becomes the
`boolean` it is, and the materialised `0`/`1` beside a boolean operand becomes that operand's own
negation.

| class | class of delta | behaviour leg |
| --- | --- | --- |
| `openspec/evidence/.../boolean-int-bitwise-patrol/fixture/BW.class` | **the patrol anchor, the acceptance target**: both shapes recover, the class is a whole recovery (0 quoted BCIs) | `true/5/true/2/-2147483648/false/3`, identical on both legs (below) |
| `tests/fixtures/recover-boolean-int-bitwise-operands/patrol-BW.class` | the same frozen bytes, committed as this change's own anchor input | the same |
| `tests/fixtures/recover-boolean-int-bitwise-operands/{v8,v8-javac8}/BW.class` | this change's fixture: the patrol source recompiled on both legs | identical on both legs |
| `tests/fixtures/recover-boolean-int-bitwise-operands/{v8,v8-javac8}/BWR.class` | this change's fixture: `accXor`, `andNotAnd`, `orNot` and `notOr` all recover | identical on both legs |

Nothing else in the corpus moved. In particular:

* `tests/fixtures/recover-boolean-int-bitwise-operands/{v8,v8-javac8}/BWN.class` (the five
  negatives) is byte-identical — the sweep's own self-test;
* `recover-conditional-rhs-field-compound`'s `BI`/`RC`/`RCN` are byte-identical: this change's
  consumption predicate keeps the field-write position the previous slice narrowed to, so its
  anchors neither gain nor lose a statement;
* `recover-chained-field-assignment`'s `CF`/`NEG` and `recover-inline-conditional-concat-operands`'s
  `ICM`/`ICN` are byte-identical;
* the same-type bitwise shapes (`boolean ^ boolean`, `int ^ int`, the over-wide shift, the folded
  constant shift, the Kernighan loop) are byte-identical inside `BW` itself — the change's own tests
  pin those bodies, and the sweep shows the only lines that moved are the two anchors';
* **no class became more refused**: the patched side adds no `not recovered` line and no
  `// @bytecode` quote the baseline did not already have, in any of the 3561 rendered candidates.

## The class-level resolution this change owes

The previous slice's report (`recover-conditional-rhs-field-compound/results/03-implementation.md`)
measured that admitting the bitwise-consumer position for *any* bitwise consumer makes `BW.andNot`
recover, which makes the class's text compile for the first time — and exposes `mix`'s pre-existing
partial quote (the accumulation dropped) as a reachable compilable-wrong face:

```
original:  true/5/true/2/-2147483648/false/3
exposed:   true/5/false/2/-2147483648/false/3     (the third value is `mix`)
```

`mix` is this change's own second row — the patrol's README files both shapes
("两形受影响（布尔累积 xor、`a & !b`）") and the design's decision 2 names the accumulate counter
(`boolean local = false; … local ^= x;` with the exit `return local;`) — so the exposure is resolved
by **recovering `mix` with `andNot`**, not by guarding the class. The class-level probe below is the
acceptance: the stripped text compiles on both compilers and answers exactly what the original class
answers, `true/5/true/…` included.

## The behaviour of the moved classes

`results/04-class-level.md` is the transcript: the frozen `BW`'s own class file prints
`true/5/true/2/-2147483648/false/3`, and the recovered text prints the same after a round trip
through `javac --release 8` and through real javac 8 (Corretto 1.8.0_432), both under
`-Xverify:all`. The same replay is the ignored test
(`cargo test --test recover_boolean_int_bitwise_operands --all-features --locked -- --ignored`),
which also compiles `BWR` on both legs and asserts that `BWN`'s stripped text does **not** compile.

## The oracle leg

`cargo test --test p3_execution_comparison --all-features --locked -- --ignored` — the discipline the
previous corpus-moving slices added — was run locally after the sweep; see `results/06-gates.md` for
its result and for every other gate.
