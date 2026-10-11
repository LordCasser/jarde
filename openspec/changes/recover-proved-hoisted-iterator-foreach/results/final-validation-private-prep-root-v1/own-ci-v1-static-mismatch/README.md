# Hoisted-iterator own-CI verifier candidate v1

This is a private verifier candidate for the `recover-proved-hoisted-iterator-foreach` product. It has not been run against CI or receipts and makes no acceptance claim.

The invocation takes the product commit and Actions run ID, captured run evidence, fresh workspace directory, its normalized `workspace-acceptance.json`, the frozen workspace runner and acceptor, regression receipt, final-build receipt and runner, and final CLI path as explicit inputs. The expected CLI SHA defaults to the already runtime-tested regression candidate `8bdb868ab58e4c9196970438c71de529996c66ba826c84fe216794d328fdfc9a`; override it only when the accepted candidate identity changes.

At invocation, the verifier independently reconstructs the source/include/fixture closure from live files, compares all pins before and after the local run, and checks every closure blob at the selected product commit. It parses the saved Cargo metadata and independently derives exact target inventory and batch commands. Each command’s raw stdout and stderr are byte/SHA checked; each target’s parsed outcomes must match both the workspace acceptance manifest and the corresponding CI outcome in each of the two workflow seeds. The workspace acceptance manifest must bind the exact workspace receipt, metadata bytes, runner, regression receipt, all target rows, local totals, required test names, and command streams.

The final-build receipt must bind the regression, workspace, and workspace-acceptance receipts; retain default `RUST_MIN_STACK`; pass the guarded format, CI clippy, global strict OpenSpec, CLI build, and final clean commands; and prove byte identity between the newly built CLI and the runtime-tested candidate. Every command is checked for successful exit, raw streams, the 1 GiB target ceiling, the 5 GiB free-space floor, and the one-second guard interval. The CI capture must contain exactly the four expected jobs and every job step must have completed successfully.

The verifier does not embed anticipated target, batch, outcome, or aggregate totals. The current workspace was read-only observed to reconstruct 4,732 live closure entries during preparation; that number is not a verifier constant or acceptance result. The current local runner/acceptor/final-build runner and regression receipt are SHA-pinned in `pins.json`. The product final build and own-CI run are not yet accepted here.

Only static checks were run on this candidate: Python AST parsing and a symbol-table unresolved-global audit. The verifier itself was not executed. No Git, Cargo, network, JDK, JADX, or product CLI command was run.

Example argument shape for the root-owned final invocation (replace commit, run ID, capture path, and result path with the actual frozen receipts):

```text
python3 /private/tmp/jarde-hoisted-iterator-own-ci-verifier-prep-luna-v1/verify-own-ci.py \
  --product-commit <40-hex-commit> --run-id <actions-run-id> \
  --capture <captured-ci-directory> \
  --workspace /private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace-run \
  --workspace-runner /private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace_runner.py \
  --workspace-runner-sha256 711fbd12ba12f8e559340f5dc8bcb3edd6a5dd6b075c9cf54677353565bc506e \
  --workspace-named-manifest /private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/workspace-acceptance.json \
  --workspace-acceptor /private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/accept_workspace.py \
  --workspace-acceptor-sha256 16683decff47cdae9cc73b7abadcbc72bdf1ce9a71733b05d5e715cbdb380df9 \
  --regression-receipt /private/tmp/jarde-hoisted-iterator-regressions-root-v3/execution.json \
  --regression-receipt-sha256 5a5ca9c44f10fa4f1f057b16e2efd3e40687e266eb1b8fee8f762167931a108f \
  --final-build <final-build-directory>/execution.json \
  --build-runner /private/tmp/jarde-hoisted-iterator-full-validation-prep-luna-v1/final_build_runner.py \
  --build-runner-sha256 48af92704170947376c798a303ed9e8a492b2f55ef55b4762bc2b1439a203ccd \
  --final-cli /private/tmp/jarde-hoisted-iterator-candidate-cli-root-v2 \
  --out <new-private-acceptance-json>
```
