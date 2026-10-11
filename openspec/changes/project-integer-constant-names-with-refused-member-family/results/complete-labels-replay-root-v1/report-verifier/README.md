# CF12 label report verifier v3

This is an independent, read-only report comparison for the eight fresh CLI captures in `jarde-cf12-labels-full-replay-root-v4`. It does not launch the CLI, JDK, Rust, or Cargo.

`verify_reports.py` binds both executable paths and SHA-256 pins, each frozen input class SHA-256, captured stdout path/length/SHA-256, and all eight successful command records. It checks exact class report identities (BLAKE3), exact root and Inner constant projections with physical field/method anchors, UTF-8 spans, unchanged physical method reports/maps/BCIs, exact allowed source substitutions in both class and per-method source text, and default/all semantic equality.

The Inner source transformation is exactly `case 3294:` to `case CONST_CDE_PRIVATE:` in full source and `f1`; root transformations remain the two exact root constant substitutions. CLI `--class` selection is verified in dotted Java-name form.

Observed result: `report-verification.json` reports `report_comparison_verified`. The runner execution as a whole remains failed because its later JDK manifest handling encountered a list/dict schema mismatch; that unrelated step is not validated here. This report-only result is not full runtime or dual-JDK acceptance.

See `pins.json` for execution, CLI, row, and output hashes. `verify_replay_manifest.py` remains the separate runtime verifier and was not run.
