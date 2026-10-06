# Task 3.2 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`HEAD`, built into `/tmp/jarde-base-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2792 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (738
  candidates).

Self-tests first: this change's `CF` gains the chain's first store (`0 -> 1`), this change's `NEG`
refusals are byte-identical, and the dup-store change's `DS` presentation is byte-identical. Then the
counts.

```
SELF-TEST OK: CF chain 0 -> 1; this change's NEG byte-identical; dup-store DS byte-identical
pass A loose candidate classes: 2792
pass A: moved=3 unrendered=1
archive candidate classes: 738
pass C: moved=5 unrendered=1
moved classes: single-class=3 jar=5 total=8
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are `package-info` entries (a `package-info.class` states no class
name a request can bind), the same on both binaries — they are printed, not silently dropped.

## The eight moved classes, classified

`results/behavior.sh` renders each one, strips the comment lines, compiles with `javac --release 8`
and with real javac 8, runs both under `-Xverify:all` and compares every answer with the committed
class file's own:

| class | class of delta | behaviour leg |
| --- | --- | --- |
| `tests/fixtures/recover-chained-field-assignment/v8/CF.class` and the `v8-javac8` twin | this change's own fixture: the chain, the saved form, the receiver copies | `9/9/9/14/9/f[x][y]` / `1/s[z]/0`, identical on both legs |
| `chained-assign-sideeffect-patrol/fixture/CH.class` | **the chain patrol anchor**: `chain()` recovers to `CH.c = 5; CH.b = 5; CH.a = 5;` | `5/5/5/14/10/20/2/0/1`, identical on both legs |
| `field-string-compound-patrol/fixture/sc.jar!SC.class` | **the `String` accumulate anchor**: `add` recovers to `this.field = this.field + "[" + x + "]";` (the `f` vs `f[x][y]` face closes) | `a-0-1-2/f[x][y]/pq/7` / `nested-run`, identical on both legs |
| `field-compound-soundness-patrol/fixture/bf.jar!BF.class` | **the compound RMW anchor**: `enable`/`disable` recover | `true/false/true/false/3` / `false/2`, identical on both legs |
| `field-compound-soundness-patrol/fixture/bg.jar!BG.class` | **the value-consuming compound anchor**: `ienable2` recovers to `this.flags = this.flags \| 1 << arg1; return this.flags;` | `true/false` / `20`, identical on both legs |
| `assign-chain-soundness-patrol/fixture/ca2.jar!CA2.class` | **the assign-chain patrol anchor**: `chained()` recovers to `CA2.c = 5; CA2.b = 5; CA2.a = 5; return CA2.a + CA2.b + CA2.c;`; `chainedArray` (the array-store chain) stays refused | quoted whole → **the stripped text does not compile** (the safe form) |
| `boolean-loop-earlyret-patrol/fixture/bi.jar!BI.class` | **recovery of three members + a newly reachable pre-existing critical anchor 15** — not a clean recovery; see below | `false/false/false/false` → `false/false/false/true` |

Nothing else in the corpus moved. In particular:

* the **static** `String` compound (`sfield += "[" + x + "]"`, the patrol's `SG`) is byte-identical:
  this change's receiver-copy rule is about the *instance* form, and the static form keeps the
  explicit builder chain it already had;
* the compound-lvalue boundary fixtures (`p3-compound-lvalue-updates`, the six gaps and five identity
  boundaries) are byte-identical, and that suite's own assertions still hold;
* every local-chain fixture (`x = y = 7`, `p3-inner-assignment`'s CF-06 controls) and the dup-store
  and postfix fixtures are byte-identical;
* no class became *more* refused: the sweep's patched side adds no `not recovered` line the baseline
  did not already have.

## `BI`: the one delta that is not a clean recovery

The class's three boolean compounds recover (`this.ok = this.ok & arg1;` and the `|` twin), which
makes the class's stripped text **compile for the first time**. Its `earlyRet` — the patrol's own
recorded critical anchor 15 (`ok &= x > 0` inside a loop, partially quoted with the effect dropped) —
is byte-identical before and after, and it answers `true` where the class answers `false`. This change
does not create that face; it makes it reachable at class granularity, where the baseline's
whole-method quote markers had kept the class from compiling at all.

The full record — the before/after texts, the `ok &= x > 0` reproduction probe frozen in
`results/probe/BE.java` with both legs' hashes, and the guard analysis (`ff3cf21b`'s value-level
soundness guard: the refusal is quoted *inside* a surviving loop, which the guard's own documentation
excludes, and the `boolean` return clause is not what fails) — is
`results/03-finding-bi-anchor15.txt`.

**Handoff:** closing the face means either extending the value-level guard's scope to refusals nested
inside surviving structures, or admitting the cross-block conditional-materialized right-hand value
into the receiver-copy proof — the conditional-value machinery
(`recover-inline-conditional-concat-operands`) and the soundness guard are their owners' work, and the
ruling on this change keeps the slice narrow and records the face for the follow-up.

## The behaviour of the two oracle legs

`cargo test --test p3_execution_comparison --all-features --locked -- --ignored` — the discipline the
previous corpus-moving slices added — was run locally: **3/3 passed**, so no oracle expectation is
stale under this change (the oracle compiles and executes the recovered bodies and compares their
answers).

## Not run: the family-fold posture

The static-generic-field-init sweep's pass B (the fold posture) is not repeated here: this change
touches no fold, no class-level assembly and no type spelling — its whole surface is the method
body's value presentation — and the fold posture renders the same method texts through the same
`jarde-java` entry the two passes above already exercise per member.
