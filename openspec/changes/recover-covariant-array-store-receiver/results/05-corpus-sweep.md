# 05 · Corpus sweep and delta classification (task 3.1)

Three two-leg scans, the workspace's own precedent (`recover-double-brace-allocation-site`'s
`scan-corpus.sh` / `scan-corpus-jars.sh` / `scan-evidence.sh`): every capture is rendered twice —
once through the **baseline** binary (HEAD `0471c6c9`, `fab06c59…`) and once through the **change**
binary (the admission, `6e1d6676…`) — and the two trees are compared with `diff -r`. Each script is
self-tested (both binaries executable, equal capture counts, no empty capture).

| scan | corpus | captures per leg | render deltas | plane deltas | timing-only |
| --- | --- | ---: | ---: | ---: | ---: |
| [scan-corpus.sh](scan-corpus.sh) → [scan-single.txt](scan-single.txt) | every `tests/fixtures/**/*.class` through `--policy single-class` | 907 | 6 | 6 | 294 |
| [scan-corpus-jars.sh](scan-corpus-jars.sh) → [scan-family.txt](scan-family.txt) | every fixture **directory** as one jar, each class through `--policy plain-jar` (families resolve against each other) | 907 | 6 | 6 | 322 |
| [scan-evidence.sh](scan-evidence.sh) → [scan-evidence.txt](scan-evidence.txt) | every `openspec/evidence/**/*.jar` entry and every loose `*.class` | 2,725 | 1 | 1 | 1,438 |

The classifier ([classify-scan.py](classify-scan.py)) reads each full `diff -r` report and splits it:
a `.txt` pair is a **render** delta; a `.err` pair differing beyond `elapsed_millis` is a **plane**
delta; anything else is **timing** noise (no two runs share a duration). Full reports:
[scan-single-diff.txt](scan-single-diff.txt), [scan-family-diff.txt](scan-family-diff.txt),
[scan-evidence-diff.txt](scan-evidence-diff.txt); classifications:
[single-classification.txt](single-classification.txt), [family-classification.txt](family-classification.txt),
[evidence-classification.txt](evidence-classification.txt).

## Every render delta, classified

All 13 render pairs — six (single-class) + six (family-jar) + one (evidence) — are the same three
presentations, on both legs, plus the patrol's own jar in the evidence corpus. Verbatim
([render-deltas.txt](render-deltas.txt)):

1. **`AS` (both legs, both fixture scans, and the patrol's `as.jar::AS.class` in the evidence scan)**
   — the anchor. `local0[0] = java.lang.Integer.valueOf(1);` →
   `((java.lang.Object[]) local0)[0] = java.lang.Integer.valueOf(1);` and the same for
   `storeNumber`'s `Double`. `storeRight` is **not** in any delta (same-type control unchanged).
2. **`SD` (both legs, both fixture scans)** — the driver: `catchWrong`'s `Integer` store,
   `catchNumber`'s `Double` store and `catchElement`'s element-receiver store widened; every control
   and negative absent from the deltas.
3. **`SC` (both legs, both fixture scans)** — the boundary probe: `local0[0] = "s";` →
   `((java.lang.Object[]) local0)[0] = "s";`, exactly as [02-gating.md](02-gating.md) states.
4. **`UB` is in no delta at all** — the unproven receiver keeps its text, which is the negative the
   change promises.

Outside this change's own new fixtures and the patrol's own `as.jar`: **zero** render deltas. Every
plane delta belongs to a class that also has a render delta (`0` plane deltas without a paired
render delta in all three scans), which is what the bookkeeping planes must do when the text itself
moves (segment counts, `text_bytes`, `text_digest`, `output_bytes`).

## Census and fingerprint

- **reader fixture census** (`crates/jarde-reader/src/classfile.rs`,
  `repository_class_fixtures_validate_without_false_target_rejections`): `(899, 3867, 334, 2445, 8)`
  → **`(907, 3911, 344, 2453, 8)`** — the eight new class files, forty-four bodies (twenty-two per
  leg), ten handler records (`AS.main`'s two `ArrayStoreException` rows and `SD`'s three catches per
  leg) and eight branch targets (`AS.main`'s two `goto`s and `UB.merged`'s `ifeq`/`goto` pair per
  leg). The recorded expectation carries the new measurement and the paragraph explaining it.
- **corpus fingerprint** (`tests/fixtures/corpus-fingerprint.json`, regenerated with the manifest's
  own `--ignored regenerate_corpus_fingerprint`): 1,666 → 1,678 files — **+12 added, 0 removed, 0
  changed** (the four sources and the eight class files of this change's fixture directory; every
  other entry's path, byte count and digest is untouched).
