# 3.1 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`HEAD`, a separate worktree at `/tmp/jarde-baseline-wt`) and this change's — and diffs the two
texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2798 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (739
  candidates).

Self-tests first: this change's `RC` gains its four anchor statements (`0 -> 2/1/1`), this change's
`RCN` refusals are byte-identical, and the two precedent families' own controls are byte-identical
(`recover-chained-field-assignment`'s `CF`/`NEG`, `recover-inline-conditional-concat-operands`'s
`ICM`/`ICN`). Then the counts.

```
SELF-TEST OK: RC anchors 0 -> 2/1/1; RCN byte-identical; CF/NEG/ICM/ICN byte-identical
pass A loose candidate classes: 2798
pass A: moved=4 unrendered=1
archive candidate classes: 739
pass C: moved=2 unrendered=1
moved classes: single-class=4 jar=2 total=6
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are the same `package-info` entries the precedent's sweep printed
(a `package-info.class` states no class name a request can bind), the same on both binaries — they
are printed, not silently dropped.

## The six moved classes, classified

| class | class of delta | behaviour leg |
| --- | --- | --- |
| `openspec/evidence/.../boolean-loop-earlyret-patrol/fixture/bi.jar!BI.class` | **the patrol anchor, the acceptance target**: `earlyRet`'s six quoted lines become `this.ok = this.ok & local5 > 0;`, and the class is a whole recovery (0 quoted BCIs) | `false/false/false/false`, identical on both legs (below) |
| `tests/fixtures/recover-conditional-rhs-field-compound/bi.jar!BI.class` | the same frozen bytes, committed as this change's own anchor input | the same |
| `tests/fixtures/recover-conditional-rhs-field-compound/{v8,v8-javac8}/BI.class` | this change's fixture: the patrol source recompiled on both legs (`this.ok = this.ok & x > 0;`) | identical on both legs |
| `tests/fixtures/recover-conditional-rhs-field-compound/{v8,v8-javac8}/RC.class` | this change's fixture: `earlyRet` (the loop anchor), `plain`, `orEq` and `mask` all recover | identical on both legs |

Nothing else in the corpus moved. In particular:

* `recover-chained-field-assignment`'s `CF`/`NEG` and `recover-inline-conditional-concat-operands`'s
  `ICM`/`ICN` are byte-identical (the sweep's own self-tests);
* the three anchors the brief names — the CH chain, the SC `String` accumulation and the BF compound
  — are byte-identical, and `recover_chained_field_assignment`'s suite is green;
* the `boolean-int-bitwise-patrol`'s `BW` is byte-identical: see below, it is why this change's
  boolean position is narrowed to the field compound;
* no class became *more* refused: the patched side adds no `not recovered` line the baseline did not
  already have.

## `BW` — the broader rule measured, and narrowed

The first implementation admitted the boolean position for **any** bitwise consumer with a
descriptor-proven boolean sibling. The sweep then moved
`openspec/evidence/java-syntax-2026-10-05/boolean-int-bitwise-patrol/fixture/BW.class`: its `andNot`
(`a & !b`, the same `ifne; iconst_1; goto; iconst_0; iand` materialisation) recovered as
`return arg0 & !arg1;`, which is what the patrol's own jadx reference prints. That made the class's
text compile for the first time — and exposed the class's pre-existing `mix` partial quote (the
boolean `^=` accumulation, quoted inside its surviving loop, its effect dropped) as a reachable
compilable-wrong face:

```
original:  true/5/true/2/-2147483648/false/3
recovered: true/5/false/2/-2147483648/false/3     (the third value is `mix`)
```

On the baseline the class's stripped text does not compile at all (`andNot`'s quote has no
`return`), so that face was unreachable. `andNot` is `recover-boolean-int-bitwise-operands`' own
row — the patrol filed the boolean–int operand restoration as its own change, with `andNot` and
`mix` as its two rows — so this change's admission is narrowed to the field compound: the bitwise
operation must be the value a claimed **field write** takes. `BW` is byte-identical to the baseline
again (measured by the sweep above), and the face stays where its owner filed it.

## The behaviour of the moved classes

Every moved class is one of this change's anchors, and the ignored replay
(`tests/recover_conditional_rhs_field_compound.rs`) compiles each one's stripped presentation with
`javac --release 8` and with real javac 8 (Corretto 1.8.0_432), runs both under `-Xverify:all` and
compares every answer with the fixture's own class files — including the frozen `bi.jar`'s own
`BI.class`, read back out of the archive. `results/04-replay-frozen-bi.txt` is the frozen anchor's
own transcript:

```
original: false/false/false/false
javac 23 --release 8: false/false/false/false
corretto 1.8.0_432: false/false/false/false
replay: both legs identical to the original
```

## The oracle leg

`cargo test --test p3_execution_comparison --all-features --locked -- --ignored` — the discipline the
previous corpus-moving slices added — was run locally after the sweep; see `results/06-gates.md` for
its result and for every other gate.
