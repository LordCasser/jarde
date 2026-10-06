# 3.2 — the corpus delta, classified

Reproduce with:

```sh
sh openspec/changes/recover-array-initializer-value-positions/results/03-corpus-sweep.sh   # the sweep
sh openspec/changes/recover-array-initializer-value-positions/results/04-corpus-delta-replay.sh
cargo test --test p3_execution_comparison --all-features --locked -- --ignored               # the oracle leg
```

The sweep renders every class committed under `openspec/evidence` and `tests/fixtures` (2807 loose
`.class` files in pass A, every `.class` entry of every committed jar in pass C) with the baseline
(parent commit `bd6ba671`) and the patched binary, and diffs the two texts. Full transcript:
[05-corpus-sweep.out](05-corpus-sweep.out).

```
SELF-TEST OK: MD/MD2/MD3/AV anchors 0 -> recovered; AVN byte-identical; CF/NEG/RC/RCN/ICM/ICN byte-identical
pass A: moved=17 unrendered=1
pass C: moved=5 unrendered=1
moved classes: single-class=17 jar=5 total=22
```

## The moved set, in full

| # | class | what moved | classification |
| --- | --- | --- | --- |
| 1 | `tests/fixtures/recover-array-initializer-value-positions/v8/{MD,MD2,MD3,AV}.class` | the change's own anchors | **the change's own fixtures** — replayed end to end (both legs, both compilers, `-Xverify:all`) |
| 2 | the same four on the `v8-javac8` leg | idem | idem |
| 3 | `tests/fixtures/recover-array-initializer-value-positions/AVN.class` | `single` — the hand-built builder self-test | **the change's own fixture**; its two negatives did not move |
| 4 | `openspec/evidence/…/array-initializer-value-patrol/fixture/md.jar!MD.class` | `partSet` — the patrol's own frozen anchor | **the change's own anchor** (the same class the v8 leg recompiles) |
| 5 | `tests/fixtures/recover-loop-else-if-early-returns/{v8,v8-javac8}/{BS,CB,CB2,LB}.class` | their `main` (eight classes) | **the same admission, in another fixture family** — the dance as a *non-final* argument |
| 6 | `…/assign-chain-soundness-patrol/fixture/ca2.jar!CA2.class` | `main` | idem |
| 7 | `…/postinc-condition-patrol/fixture/cp7.jar!CP7.class` | `main` | idem |
| 8 | `…/triple-nested-labels-patrol/fixture/nl.jar!NL.class` | `main` (and a nested local initializer in it) | idem |
| 9 | `…/binary-search-twopointer-patrol/fixture/bs.jar!BS.class` | `main` | idem |

Rows 5–9 are **twelve** classes, and every one of them moves for the same reason: their driver
passes an initialization dance as an argument that is **not the last one**
(`bsearch(new int[]{1, 3, 5, 7}, 5)`, `assignInBranch(new int[]{9, 4}, true)`,
`find(new int[]{5, 7, 9}, 7)`, `findMid(new int[][]{…}, 2)`). The patrol's own `argPos` fixture
(`sum(new int[]{5})`) has the dance as the *final* argument, which is why the position read as
"already presented": the instruction after the last element store is the invocation only when
nothing follows. The reader the change now looks for is the value's own single use, so a following
argument's run no longer hides the position — the same admission, in the argument position.

## Every moved class verified, not just counted

* Rows 1–4: the dual-leg replay
  ([dual-leg-replay.sh](dual-leg-replay.sh)) — each fixture's stripped whole class compiles with
  `javac --release 8` **and** with real javac 8 (Corretto 1.8.0_432), runs under `-Xverify:all`, and
  prints exactly what its own class prints: `MD 21/9/9/10`, `MD2 3/3/4/5`, `MD3 2/9/3`,
  `AV 7/9/3/3/1/8/2/3/6/3/9/8/10/17/8/9/3/true/2/1/2`.
* Rows 5–9: [04-corpus-delta-replay.sh](04-corpus-delta-replay.sh) compiles each moved `main` as a
  subclass unit (`public class XR extends X { <the recovered main body> }`, compiled with the
  fixture's own class on the class path, so every unqualified static call resolves through
  inheritance), runs it and the fixture's own class under `-Xverify:all`, on both compilers, and
  compares: `BS 2/-3/[1, 2, 3]/102334155`, `CB 1/4/-1/50`, `CB2 -3/1`, `LB 1/1/0/0/1/-1`,
  `CA2 15/14/dflt/q/3/9`, `CP7 0/1/true`, `NL 1,2,|3,5,6,|7,/2` — identical for all twelve, on both
  compilers ([04-corpus-delta-replay.out](04-corpus-delta-replay.out)).
* The change's own suite's replay (`the_recovered_text_compiles_and_runs_identically_on_both_legs`)
  covers rows 1–3.
* The oracle leg ([oracle-leg.out](oracle-leg.out)):

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.29s
```

  No oracle expectation is stale under this change, so none was updated.

## The unmoved set

Everything else is byte-identical, including the precedent families the sweep's own self-test pins
(`CF`/`NEG` of the chained-field-assignment change, `RC`/`RCN` of the conditional-RHS change,
`ICM`/`ICN` of the inline-conditional-concat change) and the patrol fixtures of every other family.
The zero-regression surface this change is accepted against — the four positions, the bare immediate
consumption and the sawtooth family — is pinned byte for byte by the integration test, not merely
absent from the moved list.

## The two unrendered candidates

`tests/fixtures/proved-java-structure/package-info-basic/v8/p/package-info.class` and
`openspec/evidence/java-syntax-2026-10-05/package-info-patrol/fixture/pi.jar!com/example/package-info.class`
render under no name either binary states (`operation_target_not_found`, the same answer from the
baseline and the patched binary): a `package-info` class declares no class a request can name. A
harness fact, not a change effect — recorded here so the count is not read as one.
