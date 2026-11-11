# Durable regression inputs

These are the 821 original files directly consumed by current Rust test includes, macro-expanded fixture matrices, or runtime replay helpers. They were recovered byte-for-byte from commit `8ea98e2e11d64f02092ac200d2b596511dcf5e40` after OpenSpec evidence cleanup removed test dependencies. The manifest records each original path, destination, byte count and SHA-256. The total payload is 1,151,819 bytes; 511 files are CLASS inputs.

No fixture or expected rendering was regenerated or weakened. In particular, sealed `EX.class` remains 1,632 bytes with SHA-256 `e6f0eb1e029a2c883d3ee5df5e089eda17fea27833a9244297cede139cd61287`. Runtime inputs include the old comparison records, mutator source, the exact JADX enum outputs and the original constructor-replay inputs. Files that remain tracked at their old paths are not duplicated here.

This directory is part of the durable test corpus, not disposable OpenSpec evidence. Its contents are indexed by `tests/fixtures/corpus-fingerprint.json`. The evidence cleanup must not remove it.

This restoration does not recreate missing architecture acceptance reports, resource baselines or historical holdout receipts. The three untracked S2b dev-v3 JARs are not historical recoveries; their replacement is documented separately under `tests/fixtures/s2b-inherited-generics` as a new, reproducible input baseline.
