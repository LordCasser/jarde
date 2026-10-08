# Task 3.1 — the corpus census, the committed fingerprint, and the oracle leg

## The census (baseline vs admission, every class under `tests/fixtures/**`)

[corpus-fingerprint.py](corpus-fingerprint.py) (the nested-lock slice's script, unchanged; the CLI
is `JARDE_CLI`) renders every `.class` under `tests/fixtures/**` once and pairs each input's digest
with its render's digest:

```
968 class file(s) fingerprinted into …/fingerprint-baseline.txt     (binary = HEAD, 6ca9cfdf)
968 class file(s) fingerprinted into …/fingerprint-current.txt      (binary = this slice)
```

Line-by-line comparison (same input, different render):

| path | base | current |
| --- | --- | --- |
| `tests/fixtures/recover-nested-lock-finally-bodies/v8/MLProbe.class` | `f91090af…` | `290cb382…` |
| `…/v8-javac8/MLProbe.class` | `f91090af…` | `290cb382…` |
| `tests/fixtures/recover-branching-guard-body/v8/BG.class` | `0f24c9ae…` | `469506af…` |
| `…/v8-javac8/BG.class` | `0f24c9ae…` | `469506af…` |
| `tests/fixtures/recover-branching-guard-body/v8/BGOrder.class` | `e01c5dc0…` | `f2896348…` |
| `…/v8-javac8/BGOrder.class` | `e01c5dc0…` | `f2896348…` |
| `tests/fixtures/recover-branching-guard-body/v8/BGNegatives.class` | `ac8188bb…` | `eeac7b4f…` |
| `…/v8-javac8/BGNegatives.class` | `ac8188bb…` | `eeac7b4f…` |
| `tests/fixtures/recover-branching-guard-body/v8/BGProbe.class` | `2de2640a…` | `3870c5d6…` |
| `…/v8-javac8/BGProbe.class` | `2de2640a…` | `3870c5d6…` |

**10 of 968 renders move, and every one is classified**: the branching anchor on both legs (the
slice's own claim) and this slice's own fixture (8 renders — four presentations and the
`BGNegatives` refusal whose `switchBody` diagnostic moves). The other 958 classes render
byte-identically, the five guard families' 37 anchor renders among them (`02-gating.md`). The two
`fingerprint-*.txt` files are stored beside this document.

No class was added or removed by the slice *as a delta between the two runs*: both runs enumerate
the same 968 files (the fixture's bytes are committed before the current run), so the new fixture
appears as its 8 moved renders rather than as new paths.

## The committed fingerprint (`tests/fixtures/corpus-fingerprint.json`)

`cargo test --test p5_corpus_fingerprint --locked` named the fixture's 18 unregistered files (6
sources + 12 class files) before the change; after
`-- --ignored regenerate_corpus_fingerprint`:

```
 tests/fixtures/corpus-fingerprint.json | 90 ++++++++++++++++++++++++++++++++++
 1 file changed, 90 insertions(+)
```

**Purely additive**: 0 removed lines, 18 new entries (5 lines each), every existing entry's blake3
and length untouched; `files` goes 1750 → 1768. `cargo test --test p5_corpus_fingerprint --locked`
→ `5 passed; 0 failed; 1 ignored`.

## The reader-side population assertion (corpus-moving discipline)

`crates/jarde-reader/src/classfile.rs::repository_class_fixtures_validate_without_false_target_rejections`
failed by design after the fixture landed:
`fixture population changed: re-measure these counts`,
`left = (968, 4161, 456, 2660, 8)` vs `right = (956, 4101, 430, 2576, 8)`. Per the standing
practice the **assertion is updated** (never deleted) to the measured values, with a registration
comment naming the fixture's own population (measured per class, both legs identical): 12 classes,
60 bodies, 30 per leg — `BG` 3, `BGOrder` 6, `Order` 10, `BGNegatives` 5, `BGProbe` 4, the driver 2
— 26 handler records (13 per leg: 2/3/4/3 plus the driver's own catch) and 84 branch targets (42 per
leg: 5/8/10/9 plus the driver's 9 and the lock's own `goto`), no subroutine. The test is green after
the update.

## The oracle leg

```
$ cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.15s
ORACLE-EXIT=0
```

The oracle's own corpus is a fixed set of samples (not the fixture tree), so nothing in it was
stale; it is run as the slice's standing evidence that the recovered bodies still compile and
execute as the originals do.

## The guard-family ignored legs (both JDKs)

```
$ cargo test --test recover_branching_guard_body --locked -- --ignored
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 2.06s
$ cargo test --test recover_nested_lock_finally_bodies --locked -- --ignored
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.50s
$ cargo test --test recover_io_resource_finally --locked -- --ignored
test result: ok. 2 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 1.45s
$ cargo test --test recover_loop_test_copy_store --locked -- --ignored
test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 4 filtered out; finished in 0.64s
$ cargo test --test recover_lock_guard_loop_finally --locked -- --ignored
test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 5 filtered out; finished in 0.00s
```
