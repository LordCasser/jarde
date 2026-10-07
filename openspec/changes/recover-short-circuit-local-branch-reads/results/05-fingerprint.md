# Task 3.1 — the census and the fingerprint

## The fixture census (`corpus-fingerprint.py` → `fingerprint.txt`)

`corpus-fingerprint.py` renders every `.class` under `tests/fixtures/` through the shipped entry
point and pairs each input's digest with its render's digest (`fingerprint.txt`). Diffed against
the previous slice's manifest (`recover-lock-guard-loop-finally/results/fingerprint.txt`):

```text
820a821,826
> … NotZeroBranch.class
> … NumericBranch.class
> … v8/BranchReadNegatives.class
> … v8/BranchReads.class
> … v8-javac8/BranchReadNegatives.class
> … v8-javac8/BranchReads.class
```

**Six added lines, zero changed, zero removed** — the six new class files, and every pre-existing
fixture's render digest is byte-identical to the previous slice's manifest. The two legs' renders
agree with each other for both classes (`BranchReads` `2deaa2e9…`, `BranchReadNegatives`
`59c05a1f…` on both), which is the expected leg-independence of the presentation.

## The reader crate's own fixture census

`crates/jarde-reader/src/classfile.rs`'s
`repository_class_fixtures_validate_without_false_target_rejections` counts the committed corpus
(classes, bodies, handler records, branch targets, subroutines) and fails when it moves. It moved
by this change's fixtures, so the tuple and a paragraph describing them were updated in the same
commit:

```text
left:  (879, 3829, 334, 2445, 8)
right: (873, 3801, 332, 2359, 8)   <- before this change
```

The delta: **6 classes, 28 bodies** (five bodies per class, `BranchReadNegatives` four), **2 handler
records** (`crossCatch`'s `NullPointerException` catch, one per leg) and **86 branch targets**
(43 per leg: `BranchReads`' and the controls' 16 each — `ternaryRead` 5, `ifStatement` 4,
`midChain` 7 — and `BranchReadNegatives`' 11 — `loopCondition` 6, `crossCatch` 5), no subroutine.
The per-member counts were measured by instrumenting the counting loop, not inferred.

## The corpus fingerprint manifest

`tests/fixtures/corpus-fingerprint.json` was regenerated with the manifest's own command
(`cargo test --locked --all-features --test p5_corpus_fingerprint -- --ignored
regenerate_corpus_fingerprint`). The diff is **eight added entries** (the two sources and the six
class files) and no other change: no digest moved, no classification changed. The manifest's
verification tests pass in the default suite (part of the workspace gate below).

