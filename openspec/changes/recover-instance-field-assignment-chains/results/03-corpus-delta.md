# Task 3.1 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`HEAD`, built into `/tmp/jarde-base-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2920 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture (739
  candidates).

Self-tests first: this change's `CP` gains the instance chain's first store (`0 -> 1`), this
change's `NEG` refusals are byte-identical, and the chained-field change's `CF` (the static chain
and the receiver copies) is byte-identical. Then the counts.

```
SELF-TEST OK: CP instance chain 0 -> 1; this change's NEG byte-identical; chained-field CF byte-identical
pass A loose candidate classes: 2920
pass A: moved=2 unrendered=1
archive candidate classes: 739
pass C: moved=0 unrendered=1
moved classes: single-class=2 jar=0 total=2
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are `package-info` entries (a `package-info.class` states no class
name a request can bind), the same on both binaries — they are printed, not silently dropped.

## The two moved classes, classified

Both are **this change's own positive**, one per compiler leg: `CP.class` in `v8/` and in
`v8-javac8/`. Each moves from the six-line copy-family cascade to the chain's three assignments
(the diff is in `results/01-gating.md`); the two legs' texts are identical, as their bytecode is.
Nothing else in the corpus moves:

| family | anchor | state |
| --- | --- | --- |
| the static chain | `CH.class` (patrol), `CA2.class` (patrol), `CF.chain`/`CF.pair`/`CF.call` | byte-identical |
| the compound receiver copies | `BF.class`, `BG.class` (patrols), `CF.enable`/`CF.disable` | byte-identical |
| the concatenation receiver copy | `SC.class` (patrol), `CF.add`/`CF.sAdd` | byte-identical |
| every other committed class | 2920 loose + 739 jar entries | byte-identical |

The named anchors are rendered before/after by `results/01-gating.sh`; the corpus-wide statement is
the sweep above.

## The oracle leg (mandatory corpus-moving discipline)

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
running 3 tests
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.27s
```

No oracle expectation is stale: the replay's sample list names the P3 corpus, none of which moves
(only this change's own new fixture does, and it is not one of the oracle's samples). Nothing was
updated, deleted or weakened.

## The behaviour leg

The change's own replay (`tests/recover_instance_field_assignment_chains.rs`, ignored) compiles
`CP`'s stripped text with `javac --release 8` and with real javac 8, runs both under `-Xverify:all`
and compares every answer with the committed class file's own:

```
CP   : 5/5/5
       7/7/5
```

Both legs, identical. `MX` and `NEG`'s stripped texts do not compile (their refused bodies carry
the `jarde_refused_body();` marker), which is the safe form the soundness invariant asks for.
