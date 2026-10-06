# Task 3.2 — the corpus render differential, every delta classified

`results/03-corpus-sweep.sh` renders the whole committed corpus twice — the parent commit's binary
(`HEAD`, built into `/tmp/dupstore/base-target`) and this change's — and diffs the two texts:

* **pass A**: every loose `.class` under `openspec/evidence` and `tests/fixtures`, in the
  single-class posture (2776 candidates);
* **pass C**: every `.class` entry of every committed `.jar`, in the plain-jar posture.

Self-tests first: this change's `DS` gains its eliminated form (`0 -> 1`), this change's `NEG`
refusals are byte-identical, and the postfix patrol's `NG` negatives are byte-identical. Then the
counts.

```
SELF-TEST OK: DS eliminated 0 -> 1; this change's NEG byte-identical; postfix NG byte-identical
pass A: moved=4 unrendered=1
pass C: moved=2 unrendered=1
moved classes: single-class=4 jar=2 total=6
unrendered candidates: A=1 C=1
```

The two `unrendered` candidates are `package-info` entries (a `package-info.class` states no class
name a request can bind), the same on both binaries — they are printed, not silently dropped.

## The six moved classes, classified

| class | class of delta |
| --- | --- |
| `tests/fixtures/recover-dup-store-conditional/v8/DS.class` and the `v8-javac8` twin | this change's own fixture: the eliminated forms and the split form |
| `…/v8/REF.class` and the `v8-javac8` twin | this change's own fixture: the reference form (`while (read() != null)`) |
| `ac.jar!AC.class` | **the reference-form patrol anchor**: `ioLoop` recovers to `while (read() != null) { local0 = local0 + 1; }` |
| `op2.jar!OP2.class` | **the int-form patrol anchor**: `condAssign` recovers to `return arg0 + 1 > 0;` |

Nothing else in the corpus moved. In particular:

* the copy family's assignment rule's own fixture (`tests/fixtures/p3-inner-assignment/…`,
  `InnerAssignCases.class` and the two patched controls) is **byte-identical** — the local live
  target keeps its in-place assignment expression;
* every loop-test fixture (`p3-loop-test-values`, the hand-built `REUSED_LENGTH`/`CARRIED_LENGTH`
  cases are Rust-side, and the `p3_java_recovery` loop fixtures) is **byte-identical**;
* the postfix change's own fixtures and the postfix patrol's `NG` negatives are **byte-identical**;
* no class in the corpus became *more* refused: the sweep's patched side adds no `not recovered`
  line the baseline did not already have, and the accounting check's whole-method quote never fires
  in the corpus.

## The deltas outside the two anchor classes

There are none. The mechanism's whole surface is the dance's own two shapes (a parameter target's
eliminated and split forms, and any target's eliminated form), and the only corpus classes that hold
them are this change's fixtures and the two patrol anchors.

## Not run: the family-fold posture

The static-generic-field-init sweep's pass B (the fold posture) is not repeated here: this change
touches no fold, no class-level assembly and no type spelling — its whole surface is the method
body's value presentation — and the fold posture renders the same method texts through the same
`jarde-java` entry the two passes above already exercise per member.
