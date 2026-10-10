# Conditional boundary baseline preflight v3

This is an unrun private collector for five full-class boundary fixtures. It uses the frozen typed CLI and build only as the unpatched conditional-switch baseline. It does not run a future conditional product and does not establish a CFG or product result. The fixture source and Runner remain pinned and unchanged; the only permitted Runner adaptation is adding the generated source package declaration.

The frozen baseline bindings are the typed CLI `/private/tmp/jarde-proved-local-source-types-cli-v1` (SHA-256 `e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403`), metadata `openspec/changes/recover-proved-local-source-types/results/candidate-cli-typed-root-v1.json` (SHA-256 `6295d4cc0c65bd5b5c476f71e44cf3abf513f77d854854bfdea777cbaf2be943`), and build execution `openspec/changes/recover-proved-local-source-types/results/validation-build-root-v2/execution.json` (SHA-256 `18a9750cdeb0d8bc45786f5d0766cbb256f1c8a87b1daf350965e910535a6413`). They bind source base `5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02`, the uncommitted typed-product marker, runner `run-validation-build-root-v2.py`, and the absolute v9 guard template at SHA-256 `51103f51323197db7663279776c80bd1eab95adbf5e98e26f9360293d82b8c33`. All 17 product, 10 test, and 50 canonical source pins are checked live before and after collection.

The collector imports the pinned guard through `importlib` and binds `ROOT`, `OUT`, and `command_stream` through `run_command.__globals__`. It records every argv, UTC start/duration, exit status, guard state, and raw stdout/stderr hash in a per-command checkpoint. It uses the pinned JDK manifest and live tool hashes. Every javac call pins English language/country. JADX decompilation, generated-source compilation, and runtime use the JDK 23 environment even when the source class came from the JDK 8 leg, since the launcher loads Java 11+ helper classes. Original and candidate legs otherwise remain separate.

The collector preserves JADX failures and still attempts both candidate profiles. A failed compile is recorded without requiring an impossible runtime command; a successful compile must have its runtime observation. `collector_completed` means all eligible observations and ending pins were captured. It is independent of comparison failures, so a completed observation can accurately retain a source or runtime mismatch. `product_acceptance` remains false. Output requires a fresh `--out` directory, and must not overwrite a prior draft or existing observation.

Root's v2 run exposed the path-type error in ending tool hashes and the JDK 8 environment issue for JADX. The v3 draft fixes both, adds stable javac locale flags, and checks every map origin's complete physical method identity against the containing method identity. The previous v2 raw output remains unchanged. This v3 draft has only been AST parsed; no JDK, JADX, CLI, Cargo, or Git command has run for it.

After review, run with a fresh path, for example:

```text
python3 /private/tmp/jarde-conditional-boundary-preflight-luna-v3/prepare-boundary-preflight-root.py \
  --out /private/tmp/jarde-conditional-boundary-preflight-root-v2
```

Collector SHA-256: `4a52644d8dee83d6c15014f03f4cc5950aaa4d0a39342f0036f488a8f386f917`.
