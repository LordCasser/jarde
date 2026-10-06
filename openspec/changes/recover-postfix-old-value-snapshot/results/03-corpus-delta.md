# Task 3.2 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`207760d2`, built into `/tmp/posv-base-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture;
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture.

Self-tests first: `AD` gains its `size++` line (`0 -> 1`), the `NG` negatives are byte-identical,
and the unrelated `bs.jar!BS.class` is byte-identical. Then the counts.

```
SELF-TEST OK: AD size++ 0 -> 1; NG negatives byte-identical; bs.jar!BS.class byte-identical
pass A: moved=20 unrendered=1
pass C: moved=9 unrendered=1
moved classes: single-class=20 jar=9 total=29
```

The two `unrendered` candidates are `package-info` entries (a `package-info.class` states no class
name a request can bind), the same on both binaries — they are printed, not silently dropped.

## The 29 moved classes, classified

| class | class of delta |
| --- | --- |
| `CM.class` (jar) | **the local snapshot position** — `incDec`'s `int local1 = local0++;` / `int local3 = local0--;` |
| `AD.class`, `GA.class` (jars) | **the flagship**: `this.elems[this.size++] = arg1;` (the dependency-chain family closes) |
| `AC.class` (jar) | **the static ternary arm**: `AC.pos < AC.src.length ? AC.src[AC.pos++] : null` |
| `SD.class`, `SA.class` (jars) | **the array-store right side** (`local1[local0] = local0++;`) and the cross-variable local snapshot |
| `CH.class` (loose) | **the static-field index position** (`CH.arr[CH.idx++] = 10;`) |
| `LocalRewrite.class` (loose) | the `p3-local-rewrite` fixture whose two guard assertions this change updates (the store and the branch write the postfix expression) |
| `recover-postfix-old-value-snapshot/v8/{CM,CM2,AD,GA,PT,SR,IX}.class` and the seven `v8-javac8` twins (14) | this change's own fixtures |
| `pc.jar!PC.class` | **the invoke-argument position** (outside the matrix): `PC.list.set(PC.idx++, …)` and `PC.sb.append("n").append(local0++)` |
| `it.jar!IT.class`, `it.jar!IT$IntRange.class` | the same position: `return java.lang.Integer.valueOf(this.cur++);` |
| `StringIterableForeachRunner$ProbeIterable$1.class` (four copies: `original`/`jadx` × `root-replay`/direct) | the **instance field postfix as an array index**: `return …access$200(this.this$0)[this.index++];` — the same shape as the verified `IX.fieldRead`/`IX.fieldWrite` anchors, inside the enhanced-for patrol's anonymous iterator |

Every moved class is one of the two families the mechanism states: a **local** snapshot or a
**field** snapshot, at a consumer position. No moved class changed for any other reason — in
particular the three healthy shapes (`CM.compound*`, `CM2.crossStmt`, `CM2.prefix`) and every
refusal text of the untouched shapes are byte-identical. No class in the corpus became *more*
refused: the sweep's patched side adds no `not recovered` line the baseline did not already have,
and my accounting check's whole-method quote never fires in the corpus.

## The three deltas outside the matrix

`PC`, `IT$IntRange` and the `ProbeIterable$1` anonymous iterator consume the old value as an
**invoke argument**. The matrix did not name that position, and the proof is value-level, so it does
not distinguish it. Each was checked behaviorally rather than assumed:

```
PC:  recovered 1/n5:6  == original 1/n5:6     (the patrol's trap was the wrong `n:6`)
IT:  recovered 3/4/5   == original 3/4/5
```

Both were *compilable-wrong* faces before this change (they are quoted whole by the soundness guard
at the parent commit), so recovering them correctly removes two traps; the report states them as a
scope note rather than as matrix anchors.

## Not run: the family-fold posture

The static-generic-field-init sweep's pass B (the fold posture) is not repeated here: this change
touches no fold, no class-level assembly and no type spelling — its whole surface is the method
body's value presentation — and the fold posture renders the same method texts through the same
`jarde-java` entry the two passes above already exercise per member. The three healthy shapes and
every refusal text are byte-identical in the passes that did run.
