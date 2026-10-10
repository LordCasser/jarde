# Conditional-switch CI capture and acceptance adapter

This is a private, input-driven adapter for the conditional-switch product. It contains no product commit, Actions run ID, final target count, workspace totals, or claimed acceptance result. The previous v1 draft is preserved separately. A verifier run may write acceptance only after every supplied record passes its checks.

## Evidence required

The verifier requires the exact product commit and Actions run ID, a fresh capture directory, a completed local full-workspace execution and its Cargo metadata, the conditional candidate metadata/frozen CLI/validation build and runner, and the conditional replay execution plus the exact independent-verifier invocation record. Supply SHA-256 values for every saved top-level evidence file and the replay argv/stdout. The local workspace run must be the new v4 run at `/private/tmp/jarde-conditional-workspace-batches-root-v4`; the verifier pins its runner path and SHA (`ab20981593ed3f60dabda6f2d6acf94365e6c57fe296d0fa0836557bfce72e68`). Earlier v2/v3 runs failed and cannot be used as a baseline.

The workspace record must say `passed`, report unchanged source pins, and contain no carried execution. The verifier requires the corpus fingerprint fixture in the source pin set, compares every source pin with the exact product commit and live bytes, compares saved Cargo metadata to the metadata command's raw stdout, and verifies the pinned target-size scan and guard scripts. It derives the target inventory from Cargo metadata and requires each target to appear exactly once in the raw local run. Every batch's Cargo compiler-artifact, stderr `Running` line, stdout result block, target selectors, named outcomes, ignored reason, and summary must agree. It does not hardcode the reported 359 targets, 95 batches, lib344, or any aggregate test count.

The local runtime output must include the six region proofs, the Builder proof, the selected conditional and boundary tests, the nine existing typed-boundary tests, and the two scope-boundary tests. The verifier checks those names in their actual target output. It does not infer test names from Rust syntax.

For CI, the capture verifies exact head, run URL/ID, four jobs, every job and step result, and 52 steps. The verifier reads both seed values and both workspace commands from the product commit's workflow. For each seed it pairs every `Running` header with the immediately corresponding named test block, resolves the header to the local target identity, and compares exact test names/statuses/ignored reasons and summary counts. The two seeds and the local baseline must each cover every metadata target exactly once.

The replay verifier accepts only a saved invocation record whose file SHA, argv SHA, cwd, exit state, and stdout/stderr raw streams are bound. Its explicit args must point to the same execution, CLI, metadata, validation build, source base, and two JDK homes supplied to this verifier. The verifier script itself must match product Git/live bytes, and its stdout must contain the expected replay success record. The replay execution's candidate tuple must match those same inputs.

## Run after evidence is ready

1. Finish a clean v4 full-workspace run and preserve `execution.json`, `metadata.json`, and all raw streams. Record the SHA-256 of the two JSON files. Do not use the failed v2/v3 records.
2. After the product commit's exact Actions run succeeds, run `capture-conditional-ci-root-v2.py` with the actual lowercase commit, numeric run ID, and a new output directory. The collector refuses to overwrite an existing directory.
3. Run the complete conditional replay and its independent verifier, retaining the replay execution, invocation record, argv SHA, and raw verifier stdout SHA.
4. Invoke `verify-conditional-ci-root-v2.py` with the required commit/run, capture and its execution SHA, workspace execution/metadata and SHAs, source base `2d70da515896c25ce022b8c28f4935ff2e105026`, candidate metadata/CLI/build/runner and SHAs, replay execution/invocation and SHAs, JDK 8/23 homes, and a new acceptance path. It rejects an existing acceptance path and writes the observed per-target comparisons and measured totals only after all checks succeed.

The v4 local runner uses the separate target-size adapter at `openspec/changes/recover-proved-conditional-switch-fallthrough/results/target-size-scan-root-v1.py`; its SHA is `d954cd53555e261b2033ead9fa601db51ef24a0a2602d2cb770a5413dbf0a8a7`. The adapter is checked as a separate tool pin and against product Git/live bytes, not treated as one of the product source paths.
