# Typed CI repair adapter v2 (private draft)

This private draft keeps the accepted typed CI capture and frozen product verifier intact, with the capture output moved to `/private/tmp/jarde-typed-ci-capture-root-v2` and acceptance output moved to `openspec/changes/recover-proved-local-source-types/results/typed-ci-product-v2/acceptance-root-v2.json`. The capture script differs from its repository source only in its private `OUT` path. The verifier preserves the original 17 product, 10 test, and 50 canonical pins and their metadata/build checks.

The verifier adds a separate local repair gate supplied by `--repair-execution` and `--repair-sha256`. It requires the new repair record to be schema `typed-ci-repair-local-root-v2`, status `passed`, and to contain exactly the ordered fmt and focused `p3_meeting` commands. It checks each command's raw stdout/stderr hash and size, exit code, guard state, resource values and environment; it derives all six active test names from the exact `tests/p3_meeting.rs` Git blob and requires the focused raw summary and outcomes to be exactly 6/0/0. The repair's original source pin map must equal the 76-entry union of the historical typed metadata groups. The three new source pins are checked separately against fixed hashes, product Git blobs and live files, and must remain outside that historical closure.

For each of the two fixed CI workspace seeds, the stable raw log parser now finds the `tests/p3_meeting.rs` binary and independently verifies `running 6 tests`, all six source-derived names marked `ok`, and summary 6/0/0. The earlier library 337/0/0, ten typed tests, original 50 canonical-file closure, all four CI jobs and 52 successful steps remain required. Workspace totals are still observed from the raw summaries and compared between seeds, not predeclared.

The earlier local repair execution v1 stopped at the target-size guard during the workspace command, so it cannot pass this adapter's focused repair gate. The v2 execution path and SHA must be passed explicitly; this draft does not claim that the v2 repair or CI has completed successfully. No Git, CI, Cargo, or other tool command was run while preparing these files. Python AST parsing passed.

Files `capture-typed-ci-root-v2.py.diff` and `verify-typed-ci-root-v2.py.diff` show the exact private deltas from the existing repository scripts.

The root can run the verifier with the actual product/run/build inputs and repair record:

```text
python3 /private/tmp/jarde-typed-ci-repair-adapter-luna-v1/verify-typed-ci-root-v2.py \
  --product-commit <actual-40-hex-commit> --run-id <completed-run-id> --source-base <typed-source-base> \
  --metadata <frozen-metadata> --metadata-sha256 <actual-sha256> \
  --cli <frozen-cli> --cli-sha256 <actual-sha256> \
  --build-execution <validation-execution> --build-sha256 <actual-sha256> \
  --validation-runner <runner> --runner-sha256 <actual-sha256> \
  --typed-test-count 10 --typed-test-name <each-exact-typed-test-name> \
  --repair-execution /private/tmp/jarde-typed-ci-repair-root-v2/execution.json \
  --repair-sha256 <actual-execution-sha256>
```
