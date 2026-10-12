# Six-capture CF11 baseline summary checker v2

This private v2 copies the v1 summary checker and fixes two review findings. Original oracle rows are parsed as exactly five pipe-delimited fields (`label`, `exception=…`, `left=…`, `right=…`, `effects=<nonnegative integer>`), and each family must match the exact 17 combined, 5 isolated String, or 6 isolated Integer case labels listed in `/private/tmp/jarde-cf11-structured-list-controls-prep-luna-v2/README.md`.

At startup, the checker reads this directory's `pins.json`. It verifies every saved observation and execution receipt against its pinned path, byte length, and SHA-256 before parsing the captures. It also compares the BLAKE3 package file inventory and hashes against the pinned before-import, after-import, and after-validation snapshots. The checker retains v1's raw command/stream, source/tool, CLI, class identity, report provenance, handler-gap, positive/negative compile, and no-debug javap checks.

The output remains an observation summary only. `product_acceptance` and `task1_2_complete` are false because stopping and negative-SSA attribution remain unresolved. This prep has not run the full checker. After root review, run:

```sh
python3 -B /private/tmp/jarde-cf11-structured-list-baseline-summary-prep-luna-v2/summarize_baselines.py \
  --out /private/tmp/jarde-cf11-structured-list-baseline-summary-root-v2.json
```

The output path must be new; existing summaries are never overwritten. No Cargo, JDK, Git, network, or shared-repository operation is part of this checker.
