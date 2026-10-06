# Task 3.1 — the corpus render differential, every delta classified

`03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary (`HEAD`
= `d1f0aa39`, built into `/tmp/icco/base-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2788 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (738
  candidates).

Self-tests first: this change's `ICC` gains its source-form chain (`0 -> 1`), this change's `ICN`
refusals are byte-identical, and the postfix patrol's `NG` negatives are byte-identical. Then the
counts (full transcript in `03-corpus-sweep.out`):

```
SELF-TEST OK: ICC chain 0 -> 1; this change's ICN byte-identical; postfix NG byte-identical
pass A: moved=8 unrendered=1
pass C: moved=2 unrendered=1
moved classes: single-class=8 jar=2 total=10
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are `package-info` entries (a `package-info.class` states no class
name a request can bind), the same on both binaries — they are printed, not silently dropped.

## The ten moved classes, classified

| class | class of delta |
| --- | --- |
| `tests/fixtures/recover-inline-conditional-concat-operands/{v8,v8-javac8}/{ICC,ICB,ICQ}.class` | this change's own anchors: the chain is written in source form with the comparison inlined (six classes, three per leg) |
| `openspec/evidence/…/multiconsumer-local-soundness-patrol/fixture/ni.jar!NI.class` | the frozen patrol anchor `NI`: the whole method presents (`3/10/5/true`), the frozen refusal gone |
| `openspec/evidence/…/serialization-callbacks-patrol/fixture/sz.jar!SZ.class` | **the same mechanism on an archived patrol anchor**: `main`'s chain at BCI 124 (`""+r.name+"/"+r.cache+"/"+(r != s)`) is now owned, so the chain's own values leave the deferred-binding plan and its ~16 refusal lines disappear — the method itself stays refused for its own reasons (the `OutputStream`/`InputStream` argument conversions at BCI 23/51 and the undeclared `local3`/`local4`), and its text is unchanged |
| `tests/fixtures/proved-java-structure/anonymous-cross-class-use/Main.class` | **the mechanism's own generalization**: `"sameClass=" + (local1.getClass() == local2.getClass())` is the same cut shape, so `main` recovers whole; the recovered text compiles with `javac --release 8` against the fixture's own classes and prints `sameClass=true`, exactly the frozen class's answer under `-Xverify:all` |
| `tests/fixtures/proved-java-structure/anonymous-double-site/AnonymousDoubleSite.class` | the same generalization on the frozen byte-patched negative: `main` (`"sameClass=" + (first.getClass() == second.getClass())`) is presented instead of quoted. Its text names `AnonymousDoubleSite$1` twice — the two allocation sites the fixture's `freeze.py` patched to one anonymous class — and javac refuses that binary name from source (it resolves as a nested type), so the text stays **uncompilable**, which is the safe form: a body a reader cannot compile, never one that compiles and behaves differently. The fixture's own mechanism (the anonymous-allocation scan) is untouched, and its own suite passes |

Nothing else in the corpus moved. In particular:

* the no-branch control `ICM` and the negatives `ICN` are **byte-identical** on both legs (they are
  this change's own fixtures, and the sweep's self-test pins `ICN`);
* `NI2` and `NMA` (the frozen no-branch and reduced-comparison probes) are **byte-identical**;
* the whole `jre_concat_split` surface of the corpus is untouched except where the certificate
  proved the cut: no class gained a `not recovered` line, and no class lost one that this table does
  not name.

## The fingerprint and the census

* `cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint`
  → **+85 lines, 0 removed**: this change's seventeen new files (five sources and twelve class
  files), five lines each. No existing entry moved.
* `crates/jarde-reader/src/classfile.rs`'s fixture-population census is a **re-measure**, not a
  relaxation: `(789, 3311, 286, 2061, 8)` → `(801, 3363, 288, 2095, 8)` — +12 classes, +52 bodies,
  +2 handler records (`ICN.guarded`'s `RuntimeException` catch, one per leg) and +34 branch targets
  (the cut's `if_acmpne`/`goto` pair in `ICC`, `ICB`, `ICQ` and `ICN.guarded`, `armCall`'s and
  `armStore`'s `ifeq`/`goto` pair, `nested`'s two comparison pairs and the catch's own `goto` — 17
  per leg). The comment above the tuple gains this change's paragraph naming the classes, the two
  legs and the counts, exactly as the preceding changes' paragraphs do.

## The oracle ignored leg (corpus-moving discipline)

`cargo test --test p3_execution_comparison --all-features --locked -- --ignored` → **3 passed, 0
failed** (exit 0). The sweep moves no expectation this leg holds: the two archived anchors it
replays are not among the ten moved classes, and the change's own fixtures are not in its set. Run
locally before the gate, as the discipline requires — CI runs this leg.

## What is not repeated here

The static-generic-field-init sweep's pass B (the fold posture) is not repeated: this change touches
no fold, no class-level assembly and no type spelling — its whole surface is the method body's value
presentation and one certificate over the SSA/CFG — and the fold posture renders the same method
texts through the same `jarde-java` entry the two passes above already exercise per member.
