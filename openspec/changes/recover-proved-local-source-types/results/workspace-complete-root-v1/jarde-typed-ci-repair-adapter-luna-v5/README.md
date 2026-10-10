# Typed CI repair adapter v5 (private draft)

This copy preserves the reviewed v4 CI capture and acceptance logic, including the original 17 product-source, 10 test-source, 50 canonical-input closure and its separate nine-pin local repair gate. Capture uses `/private/tmp/jarde-typed-ci-capture-root-v5`; acceptance writes only `typed-ci-product-v5/acceptance-root-v5.json`.

The added P5 gate stays outside that historical closure. It binds `tests/p5_bulk_corpus.rs` to the selected product commit and live source at SHA-256 `48ec01a4123021db3b4f6e5942adfbc1bba343009741a7edd767dd15d9009fee`. For each fixed CI seed it derives the six P5 test names from that source, requires one P5 Cargo header, and independently checks the exact five successful tests plus the ignored recorder and 5/0/1 summary. The existing exact-result checker for all other binaries is unchanged.

The verifier requires explicit `--p5-acceptance` and `--p5-acceptance-sha256` arguments. It validates the P5-only cost-repin acceptance, before/after source bytes, AnalysisSteps-only deltas, recorder invocation and archived raw streams, and the pre-repin failing regular-batch row 62 and its archived raw streams. P5 remains separate from the 17/10/50 build closure. The P5 acceptance itself records `full_workspace_status_at_verification: running`, so it cannot establish full-workspace completion.

Static draft only. No Git, Cargo, JDK, CLI, or CI command was run. Validation was limited to Python AST parsing and source/evidence inspection.
