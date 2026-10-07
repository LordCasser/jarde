# 3.1/3.2 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`9faae256`, a separate worktree at `/tmp/jarde-spn-baseline`, target dir
`/tmp/jarde-spn-baseline-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2838 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (739
  candidates).

Self-tests first: the patrol's frozen `B5.class` drops from one refused body to none with all three
constructions written, this change's `SPC` (the statement-position counterexample) and `SPN` (the
four negatives) are byte-identical, and the precedent families' own controls are byte-identical
(the bitwise change's `BW`/`BWN`, the conditional-rhs field-compound change's `BI`/`RC`/`RCN`, the
chained-field-assignment change's `CF`/`NEG`, the inline-conditional-concat change's
`ICM`/`ICN`). Then the counts:

```
SELF-TEST OK: B5 refusals 3 -> 0 with all three statements; SPC/SPN byte-identical; BW/BWN byte-identical; BI/RC/RCN/CF/NEG/ICM/ICN byte-identical
pass A loose candidate classes: 2838
pass A: moved=13 unrendered=1
archive candidate classes: 739
pass C: moved=1 unrendered=1
moved classes: single-class=13 jar=1 total=14
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are the same `package-info` entries every precedent sweep printed
(a `package-info.class` states no class name a request can bind), the same on both binaries — they
are printed, not silently dropped.

**No class became more refused**: over every rendered pair in both passes, the patched side adds no
`jarde_refused_body` marker and no `// @bytecode` quote the baseline did not already have
(`results/03-corpus-sweep.out` is the transcript; the count is 0).

## The 14 moved classes, classified

`results/04-class-level.sh` renders every moved class with both binaries, strips the comment lines,
compiles **both** texts with `javac --release 8` against the fixture's own sibling classes, and —
when the patched text compiles — runs the original class and the recovered one under `-Xverify:all`
and diffs their output. `results/04-class-level.out` is the transcript.

| class | class of delta | text before → after | behaviour leg |
| --- | --- | --- | --- |
| `statement-new-patrol/fixture/B5.class` (+ `fam.jar!B5.class`) | **the patrol anchor**: `main`'s three statement-position constructions are written | refused body → compiles | **identical** (9 lines, `orig.out`) |
| `statement-new-patrol/fixture/B6.class` | the discriminator: `argless`/`withArg` recover, `chained` stays refused | refused body → still incomplete (the registered boundary) | — |
| `tests/fixtures/recover-statement-position-news/{v8,v8-javac8}/SP.class` | this change's positives: `argless`, `withArg`, `fromArg`, `nestedArgument`, `mixed` | refused body → compiles | **identical** (12 lines) |
| `…/{v8,v8-javac8}/SB.class` | the patrol's `B6` shapes without the boundary | refused body → compiles | **identical** (1 line, `9`) |
| `ctor-throw-init-patrol/fixture/CE.class` | the two statement constructions inside the `try` (`new CE(5); new CE();`) | compiles → compiles, with the constructions written | **identical** (5 lines, `caught:neg:-1` included) |
| `discarded-allocation-patrol/fixture/CD.class` | `main`'s `new CD$C();` | refused body → compiles | **identical** (8 lines) |
| `discarded-allocation-patrol/fixture/DN.class` | `discarded()`'s `new DN$N();` | refused body → still incomplete | — (below) |
| `recover-javac8-allocation-qualifier-null-check/d3/D3.class` | `main`'s `new D3$In(new D3());` | refused body → still incomplete | — (below) |
| `p3-nested-parent-projection/{v8,v8-javac8}/Multiseg.class` | `main`'s `new Multiseg(new MO());` | refused body → still incomplete | — (below) |
| `proved-java-structure/…/unspellable-owner-alloc/UnspellableOwnerAlloc$1.class` | the child's `render()`: `new DeepCarrier.Mid.Leaf();` | refused body → compiles | **identical** (3 lines, `leaf-allocated` included) |

### The three classes whose text still does not compile

None of the three is this change's own doing, and each is printed with the reason javac states, on
both sides:

* **`DN`** — the class's text was *already* not a compilable unit on the baseline: its other,
  already-recovered methods write the self-nested pool spelling (`DN$N local0 = new DN$N();`,
  `new DN$N().hi()`, `takes(new DN$N())`), and javac reports `cannot find symbol: class DN$N` for
  each of them. This is the `nested-name-spelling-patrol` boundary: this text folds no declaration
  for the member, so the member is its own physical unit under the pool's name. The delta adds one
  more statement in the same spelling; it adds no new *kind* of failure.
* **`D3`** — the site's class (`D3$In`) is a **member class of the declaring class**, and the member
  projection (`new D3().new In()`) is not proved in this request, so the expression is written in
  the channel's own spelling: `new D3$In(new D3())`. That spelling is **pre-existing**, not this
  slice's: a hand-built control (`D3y.run` = `new D3$In; dup; new D3; dup; D3.<init>; D3$In.<init>;
  Take.take(Object)`, the same construction consumed by a call instead of a `pop`) renders
  `Take.take((java.lang.Object) new D3$In(new D3()));` on the **baseline** binary and byte-identically
  on the patched one. The statement position makes the same expression reachable for the discarded
  shape; it does not invent the spelling. javac rejects it (`cannot reference non-static variable
  this from a static context`), so nothing compiles silently wrong. The member form stays out:
  `tests/recover_javac8_allocation_qualifier_null_check.rs` still asserts `!text.contains("new In()")`.
* **`Multiseg`** — the class **declaration** is refused on both sides (`需要包含MO.Mid的封闭实例`:
  the multi-segment `Signature` path `LMO<…>.Mid;` has no one-segment direct-parent candidate), so
  the class text never was compilable; the delta is `main`'s own statement `new Multiseg(new MO());`
  being written inside it.

### The two stale expectations this change updates (assertion updates, never deletions)

* `tests/class_source.rs::anonymous_superclass_refuses_unproved_local_declaration_sites` — its
  "不可拼写 owner 的分配" block asserted the child method was incomplete
  (`anonymous_child_methods_incomplete`) because the child's allocation was quoted. The statement
  position now writes the child's own statement, the child method is complete, and the anonymous
  fold happens with the child's method inside it — the shape the fixture's source states. The
  updated block asserts the fold, the child's statement, and the *absence* of the old diagnostic.
* `tests/recover_javac8_allocation_qualifier_null_check.rs::a_construction_whose_only_reader_is_the_discard_keeps_the_member_form_out`
  (renamed from `a_construction_with_no_rendering_reader_refuses_on_both_legs`) — the reader is now
  the statement position's discard, so the class is no longer quoted whole. The boundary that slice
  froze is kept verbatim: the **member** fold (`new In()`) still does not appear, because the tail
  dance still proves no member relation.

## The behaviour of the moved classes

`results/04-class-level.out` is the transcript: eight of the moved classes' texts compile and answer
**exactly what their original classes answer** under `-Xverify:all`, including the constructor side
effects the patrol filed as swallowed (`B5`'s three `println`s, `CE`'s `caught:neg:-1`,
`UnspellableOwnerAlloc`'s `leaf-allocated`). The ignored test
(`cargo test --test recover_statement_position_news --all-features --locked -- --ignored`) replays
the same anchors on both compilers — `javac --release 8` and real javac 8 (Corretto 1.8.0_432) — and
asserts that the classes which keep a refused body (`B6`, `SPN`, the hand-built `SPC`) do **not**
compile.

## The oracle leg

`cargo test --test p3_execution_comparison --all-features --locked -- --ignored` — the discipline
the corpus-moving slices added — was run locally after the sweep: **3 passed, 0 failed**, with no
stale expectation to update (nothing in its sample set moved). See `results/06-gates.md`.
