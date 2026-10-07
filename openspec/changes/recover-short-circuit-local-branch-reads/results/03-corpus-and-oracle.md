# Task 3.1 — the corpus differential and the oracle leg

## The whole-corpus differential

`03-corpus-sweep.sh` renders every class the repository commits under `openspec/evidence` and
`tests/fixtures` with the parent commit's binary and with this slice's, in both the single-class
and the plain-jar posture, and diffs the two texts. Self-tests run first: the patrol anchor must
move, this change's own negatives and a healthy control must not. The transcript is
`03-corpus-sweep.out`.

```text
SELF-TEST OK: OP2 anchor 0 -> 1; this change's negatives byte-identical; healthy control byte-identical
pass A loose candidate classes: 2866
pass A: moved=4 unrendered=1
archive candidate classes: 739
pass C: moved=1 unrendered=1
moved classes: single-class=4 jar=1 total=5
unrendered candidates: A=1 C=1
```

**Every moved class, classified:**

| moved | class | why |
| --- | --- | --- |
| single-class | `tests/fixtures/recover-short-circuit-local-branch-reads/v8/BranchReads.class` | this change's own new fixture: the ternary, the `if` statement and the mid-chain read recover |
| single-class | `.../v8-javac8/BranchReads.class` | the same, real-javac-8 leg |
| single-class | `.../controls/NumericBranch.class` | the class-level render moves in its **unpatched** members (`ternaryRead`, `midChain`); the patched `ifStatement` member is byte-identical — see `01-refusal-chain.md` |
| single-class | `.../controls/NotZeroBranch.class` | the `ifne` arm: the same class's `ifStatement` presents `if (!b)` |
| jar | `openspec/evidence/java-syntax-2026-10-05/operator-remainder-patrol/fixture/op2.jar!OP2.class` | the anchor: `condAssignOld` recovers |

The two `unrendered` candidates are the pre-existing `package-info` shapes that render under no
name they state (one loose file, one jar entry). They are **not** skipped silently: each is
compared byte for byte on the two binaries and both are identical, and the script fails on a
one-sided non-render (`ONE-SIDED NON-RENDER (investigate)`) — none occurred.

No class outside the change's own fixtures and the patrol anchor moved. In particular: every
`scv-*` fixture and control, the `putstatic`-Z/`ireturn`-Z and `append(Z)` anchors, the whole
`mixed-short-circuit-*` family, the `p3-boolean-*`/`p3-loop-boolean-*`/`proved-java-structure`
boolean fixtures, and the local-scope and handler controls.

## The oracle leg

```text
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
running 3 tests
test the_corpus_is_read_the_same_way_by_every_legal_flag_set ... ok
test the_bulk_entrys_bodies_are_the_same_text_and_the_same_behaviour ... ok
test the_p3_findings_are_replayed_by_compiling_and_executing_the_bodies ... ok
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 43.64s
```

**No stale oracle expectation**: the slice's corpus delta does not include any class the oracle
pins, so no assertion needed updating (and none was deleted or relaxed).

## The replay (`03-roundtrip.sh`)

```text
== OP2 ==
original  (frozen jar):  12/-4/2147483644/16/NaN/Infinity/-Infinity/true/true/1
recovered (javac 23/8):  12/-4/2147483644/16/NaN/Infinity/-Infinity/true/true/1
verdict: identical (both legs compiled; condAssignOld(0) included)

== BranchReads (v8, javac 23.0.1 --release 8) ==
original:  1 -1 1 -1 true false
recovered: 1 -1 1 -1 true false
verdict: identical

== BranchReads (v8-javac8, Corretto 1.8.0_432) ==
original:  1 -1 1 -1 true false
recovered: 1 -1 1 -1 true false
verdict: identical

== BranchReadNegatives (the boundaries) ==
verdict: does not compile, as the safe direction requires
```

`OP2`'s own `main` is a registered residual of another family (the concatenation chain's saved
producers, refused before and after this change), so the replay replaces its one
`jarde_refused_body();` marker with the **frozen source's own** `main` body — read out of
`OP2.java` and asserted to be that source's, not a transcription — before compiling. Everything
else in the class is the recovered text, and the line compared includes `condAssignOld(0)`.
