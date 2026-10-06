# Task 3.1 — the Phase-A matrix, anchor by anchor

Every line below is a render of the frozen fixture with this change's build
(`target/debug/jarde-cli class-source --policy plain-jar --format text`), stripped of the
presentation's comment lines. The behavior leg is `results/behavior.sh`: the stripped text is
compiled with `javac --release 8` (javac 23.0.1) and with real javac 8 (Corretto 1.8.0_432), run
under `-Xverify:all`, and compared with the fixture's own class file.

## The anchors

| anchor | class / method | recovered text | behavior (both legs) |
| --- | --- | --- | --- |
| local snapshot (store) | `CM2.immUse` | `int local1 = local0++;` | `CM2` runs `5/6/15/16`, identical to the original |
| local snapshot (expression) | `CM2.postfixExpr` | `return local0++ + 10;` | " |
| local snapshot four-shape | `CM.incDec` | `int local1 = local0++;` … `int local3 = local0--;` … `return local0 + local1 + local2 + local3 + local4;` | `CM` runs `2/30/2/29/5`, identical |
| index position, local | `IX.localWrite` / `SR.backWrite` | `local1[local0++] = 10;` / `local1[local0--] = local1[0] + 100;` | `IX` `8/0/0/20/0`, `SR` `102/104001`, identical |
| index position, local read | `IX.localRead` | `return local1[local0--];` | " |
| index position, static field | `IX.fieldWrite` / `IX.fieldRead` | `IX.arr[IX.idx++] = 20;` / `return IX.arr[IX.idx--];` | " |
| **the flagship** | `AD.add` | `this.elems[this.size++] = arg1;` | `AD` runs `x/y` — the `null/null vs x/y` compilable-wrong face is closed |
| the flagship, generic | `GA.add` | `this.elems[this.size++] = arg1;` | isolated probe compiles and runs `x/y` on both legs |
| static ternary arm | `PT.read` (patrol: `AC.read`) | `return PT.pos < PT.src.length ? PT.src[PT.pos++] : null;` | `PT` runs `a/b/null/3`, identical |
| array-store right side | `SR.arrSelf` (patrol: `SD.arrSelf`) | `local1[local0] = local0++;` | `SR` runs `102/104001`, identical (`a[i] = i++` = 102 ✓) |

`AD.class` and `GA.class` render byte-identically in the class-source of both compiler legs;
`GA`'s only leg difference is the pre-existing `(T[])` cast presentation in `typedArray`/`fill`,
outside this slice.

## Behavior, verbatim

```
CM: leg 23 OK  2/30/2/29/5 status=0
CM: leg 8 OK  2/30/2/29/5 status=0
CM2: leg 23 OK  5/6/15/16 status=0
CM2: leg 8 OK  5/6/15/16 status=0
AD: leg 23 OK  x/y status=0
AD: leg 8 OK  x/y status=0
PT: leg 23 OK  a/b/null/3 status=0
PT: leg 8 OK  a/b/null/3 status=0
SR: leg 23 OK  102/104001 status=0
SR: leg 8 OK  102/104001 status=0
IX: leg 23 OK  8/0/0/20/0 status=0
IX: leg 8 OK  8/0/0/20/0 status=0
```

`GA`'s whole class does not compile because its `main` names the nested interface through the pool
spelling (`GA$Cfg`) — a presentation of that member that is unchanged from the parent commit. Its
anchor was verified by isolating the recovered method into a same-named generic class and running
it (`x/y` on both legs):

```sh
void add(T arg1) {
    this.elems[this.size++] = arg1;
    return;
}
```

The self-assignment patrol's trap values are preserved, not regressed: `SR.arrSelf` answers `102`
(the patrol's `2≠102` face) and the `NG` negatives are quoted whole, so their stripped text does
not compile at all.

## The negatives (unchanged)

```
NG.postSelf       the value at BCI 6 is the value local 0 held at BCI 2 …            (i = i++)
NG.postSelfDec    the value at BCI 6 is the value local 0 held at BCI 2 …            (i = i--)
NG.fieldSelf      the old-value update ending at BCI 19 has no complete same-target,  (f = f++)
                  single-consumer, same-handler and evaluation-order proof
NG.compoundSelf   the value at BCI 10 is the value local 1 held at BCI 2 …           (i += i++ + 1)
NG.condShape      local 0 crosses a quoted fallback region …                          (Phase B)
```

`NG`'s stripped class does not compile (its bodies are quoted whole) — the sound form the soundness
invariant asks for; `FS` (the patrol's own field self-assignment and compound-mixed class) renders
**byte-identically** to the parent commit.

## The consumer positions the mechanism reaches beyond the matrix

The proof is value-level, so three shapes outside the matrix recover too, and each was checked
behaviorally rather than assumed:

* `PC.viaArg` — `PC.list.set(PC.idx++, (java.lang.Object) "X");` (the postinc-consumer-positions
  patrol's anchor 9). Recovered `1/n5:6` = the original's `1/n5:6` (the patrol's trap was the wrong
  `n:6`).
* `PC.viaChain` — `PC.sb.append("n").append(local0++);` (the same patrol's anchor 8).
* `IT$IntRange.next` — `return java.lang.Integer.valueOf(this.cur++);` (the handwritten-iterator
  patrol). Recovered `3/4/5` = the original's `3/4/5`.

All three were compilable-wrong faces before this change and are guarded (quoted whole) at the
parent commit; they recover *correctly* here. This is reported as a scope note: the matrix did not
name the invoke-argument position, and the mechanism does not distinguish it.
