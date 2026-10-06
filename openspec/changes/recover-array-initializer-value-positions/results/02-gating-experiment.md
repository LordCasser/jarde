# 1.1/1.2 — the gating experiment, and the baseline re-verified

Reproduce with:

```sh
sh openspec/changes/recover-array-initializer-value-positions/results/02-gating-experiment.sh
```

Two binaries: the **baseline** built from this change's parent commit (`bd6ba671`, a separate
worktree at `/tmp/jarde-array-baseline-wt`) and the **patched** one. Both render this change's own
fixture inputs — `MD`/`MD2`/`MD3` (the patrol's six-position discriminator, recompiled) and `AV`
(the change's own neighbours) on the javac 23 `--release 8` leg, plus the hand-built `AVN` — and the
script reports which **methods** moved, with the expectation stated before the counts are believed.

```
MD moved: partSet
MD2 moved: retPos
MD3 moved: bareIdx2
AV moved: elemStore immIdx immLen immIdxVar immIdxExpr immIdxSum twoIdx twoStores nestedIdx condIdx immIdxInCall order
GATING OK: the two positions flip (MD.partSet, MD2.retPos, MD3.bareIdx2), the four positions and
the bare consumption do not move, AVN moves only its builder self-test, and no patrol class quotes bytecode.
```

## What flipped — the two refusing positions, and only their position class

| class | member | before (baseline) | after (patched) |
| --- | --- | --- | --- |
| `MD` | `partSet` | whole body quoted: `the copy at BCI 7 has no proved local assignment` ×2 (BCIs 11, 12) | `int[][] saved0 = MD.partial;` / `saved0[0] = new int[]{7};` / `MD.partial[1] = new int[2];` / `return MD.partial[0][0] + MD.partial[1].length;` |
| `MD3` | `bareIdx2` | whole body quoted: `the copy at BCI 3 has no proved local assignment` ×2 (BCIs 7, 10) | `return new int[]{9}[0];` |
| `MD2` | `retPos` | the same refusal (BCIs 6, 9) | `return new int[]{4}[0];` |

`MD2.retPos` is the patrol fixture's own comment calling that member 返回位; its bytecode is the
immediate-subscript position, and it refuses at the parent commit exactly as `MD3.bareIdx2` does —
so the change flips **two** members of the patrol's own fixture set beyond the two the design names,
both of them the same position (the immediate subscript), not a fifth position.

`AV` moves exactly the twelve members that are the same two position classes one step further:

* `elemStore` (element store into a pre-existing array), `twoStores` (two of them in one body);
* `immIdx` (constant subscript), `immLen` (length receiver), `immIdxVar` (variable subscript),
  `immIdxExpr` (computed subscript), `immIdxSum` (the reader inside a larger expression),
  `immIdxInCall` (a call beside a subscript), `condIdx` (the reader as a condition's operand),
  `nestedIdx` (a child initializer read immediately), `twoIdx` (two dances, one reader each),
  `order` (an effectful element beside an effectful index).

## What did not move — the zero-regression surface

The methods whose text must not move are **absent from every moved list**:

| position | members | status |
| --- | --- | --- |
| local store | `MD2.localPos`, `AV.localPos` | unmoved |
| field write | `MD2.fieldPos`, `AV.fieldPos` | unmoved |
| call argument | `MD2.argPos`, `AV.argPos` | unmoved |
| outer initializer's element | `MD.jagged`, `MD.mkJagged`, `AV.outerPos`, `AV.jagged` | unmoved |
| bare immediate consumption | `MD3.bareIdx`, `MD3.bareRet`, `AV.bareIdx`, `AV.bareRet` | unmoved |
| sawtooth family | `MD.jagged`, `MD.mkJagged`, `AV.jagged` | unmoved |
| the rest of the patrol classes | `MD.sumJag`, `MD.regSet`, `MD.main`, `MD2.sum`, `MD2.main`, `MD3.main`, … | unmoved |

The integration test pins each of them **byte for byte**
(`the_presenting_positions_keep_their_text_byte_for_byte`), so the claim is not "the diff did not
show them" but "the text is the frozen parent commit's text".

## The negatives

`AVN` moves exactly one member — `single`, the builder's own self-test (the hand-built bytes really
are the anchor's shape, and the patched binary writes `return new int[]{9}[0];` for it). The two
negatives do not move at all:

```
    public static int twoReaders() {
        // jarde: not recovered: the recovery run for `twoReaders()I` stopped (jre_ir_table_missing); the analysis of that member did not complete (ir_frame_inconsistent)
    }

    public static int discarded() {
        // @bytecode 1 … // @bytecode 3 1 … // @bytecode 7 1
        // the copy at BCI 3 has no proved local assignment
        // @bytecode 8 1 … // @bytecode 9 … // @bytecode 12 11 1
        // the copy at BCI 8 has no proved local assignment
    }
```

`discarded` is the shape the *gate* refuses: its `dup` is not a consumption position this layer
writes an array expression at, so the initializer stays unproved and every instruction of the dance
stays named by a quote (BCIs 1, 3, 7, 8, 9, 12) — the accounting observation the test asserts.
`twoReaders` is refused one layer below the recovery rules (the member's analysis does not complete:
`ir_frame_inconsistent`), which is where a stack value with two *readers* is answered; the patched
binary answers it exactly as the baseline did.

## The baseline re-verified (task 1.2)

The patrol's recorded renders (`openspec/evidence/java-syntax-2026-10-05/array-initializer-value-patrol/results/jarde-MD*.txt`)
were produced before the copy family's last two slices merged; re-rendering the patrol's own inputs
at `bd6ba671` reproduces the same **refusals** but a different *shape* of text for `MD.partSet` — the
parent commit quotes the whole body (`the copy at BCI 7 has no proved local assignment` once, with
the value-level guard's escalated BCI list) where the patrol's own file had shown the partial
presentation with `saved0`. Both refuse the same two positions, and this change's own baseline is
the parent commit's render, not the patrol's older file; the fixture texts the tests pin are the
parent commit's, re-measured here.
