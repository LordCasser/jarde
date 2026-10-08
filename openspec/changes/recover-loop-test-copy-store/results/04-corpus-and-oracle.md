# Task 3.1 — the corpus delta and the oracle leg

## The fingerprint

`cargo test --test p5_corpus_fingerprint --locked` before the regeneration reported exactly nine
files "in the corpus but not in the manifest" — this slice's own fixture inputs — and nothing else:
no recorded file changed and none vanished. Regenerated with

```
cargo test --test p5_corpus_fingerprint --locked -- --ignored regenerate_corpus_fingerprint
```

and the diff is **nine pure additions** (`git diff --stat tests/fixtures/corpus-fingerprint.json`
= 45 insertions, 0 deletions):

| file | bytes | blake3 |
| --- | --- | --- |
| `tests/fixtures/recover-loop-test-copy-store/Probe.java` | 969 | `d1af14bb…2620bf` |
| `tests/fixtures/recover-loop-test-copy-store/ProbeControls.java` | 891 | `758ac756…f90251e` |
| `tests/fixtures/recover-loop-test-copy-store/data.txt` | 12 | `e55980c7…d108e881a9` |
| `tests/fixtures/recover-loop-test-copy-store/v8/Probe.class` | 1261 | `d0c061a5…d6d071406` |
| `tests/fixtures/recover-loop-test-copy-store/v8/ProbeControls.class` | 1074 | `6ff4c844…8dde21d33e` |
| `tests/fixtures/recover-loop-test-copy-store/v8/MultiCopy.class` | 1261 | `2ac17b1d…54fd6cd8545fe5` |
| `tests/fixtures/recover-loop-test-copy-store/v8-javac8/Probe.class` | 1267 | `91554704…cbfbcb1bb50453e` |
| `tests/fixtures/recover-loop-test-copy-store/v8-javac8/ProbeControls.class` | 1077 | `df0f2cc4…beefd631b36651d4` |
| `tests/fixtures/recover-loop-test-copy-store/v8-javac8/MultiCopy.class` | 1267 | `87cc2ef9…1805f2168daef2ba` |

(The fixture's `README.md` and `patch_controls.py` are outside the walk: the manifest excludes
`.md` and `.py` as records rather than inputs.)

## The fixture census

`crates/jarde-reader/src/classfile.rs`'s fixture-population test re-measures every `.class` under
`tests/fixtures`. Measured before the update:

```
fixture population changed: re-measure these counts
  left: (956, 4101, 430, 2576, 8)
 right: (950, 4077, 418, 2540, 8)
```

i.e. **(classes, bodies, handler records, branch targets, subroutines)** moved by
**+6, +24, +12, +36, +0**. The delta is this slice's six classes (three per leg), four bodies each
(`<init>`, the loop member, the guard control or refusal, `main`), two exception rows per guard body
(`guardPlain`'s and `guardIfFirst`'s protected range and binding store, per leg) and six branch
instructions per class (the loop tests' `if_icmpeq`/`goto` pairs and the `if`s' pairs, per leg) —
no subroutine. The pinned tuple is updated to the measured `(956, 4101, 430, 2576, 8)` with the
precedent's provenance comment; `cargo test -p jarde-reader --lib` is green (178 passed).

## The oracle leg

```
cargo test --test p3_execution_comparison --all-features --locked -- --ignored
test result: ok. 3 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 49.91s
```

No stale oracle expectation: the P3 compile-and-execute comparison's own texts and behaviors are
unaffected by this slice (its fixtures are the committed corpus members, and none of them is a
loop-test copy-and-store anchor).

## The targeted corpus sweep

`render-set.sh` renders 28 anchors and negatives (the io, lock-guard, dup-store, postfix and
CF-06 families plus this slice's own) in each of the four build states; the classification of every
delta is in `01-instrumentation-and-gating.md`. No render outside the loop-test position's own
anchors and negatives moved in any state.
