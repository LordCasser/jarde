# Hoisted-iterator own-CI verifier candidate v2

Private verifier candidate for `recover-proved-hoisted-iterator-foreach`. It has not been executed against an own-CI run or final-build receipt; it does not establish acceptance.

The verifier receives the actual product commit and Actions run ID, capture directory, fresh full-workspace directory and normalized workspace acceptance manifest, the exact frozen workspace runner/acceptor, the accepted regression receipt, final-build receipt/runner, and final CLI file. It reconstructs source/include/fixture inputs live, checks workspace and final-build before/after pins against selected commit blobs and local bytes, derives Cargo target/batch coverage from saved metadata, and verifies each local named outcome from the raw command streams against the per-target manifest and both CI seed runs. It checks all four CI jobs and every step, plus the 5 GiB free-space, 1 GiB target, one-second polling, cleanup, and default Rust stack records. No anticipated target, batch, test, or aggregate totals are constants.

This v2 corrects two producer-contract mismatches found in root's static read of v1. The final-build runner records the runtime-tested CLI at `jarde-hoisted-iterator-candidate-cli-root-v2`; the regression receipt separately records `jarde-hoisted-iterator-candidate-cli-root-v3`. The verifier now checks both recorded paths, modes, sizes, hashes, and exact bytes, and requires the final CLI argument to match the final-build runner's recorded path. The final-build runner's helper references are workspace runner + guard + scanner; its own path/hash is independently verified through the `runner` field. The v1 candidate remains unchanged as the first draft.

Current producer contracts were statically read from the frozen workspace runner, workspace acceptor, and final-build runner. The acceptance-manifest checks bind its `evidence` and `contract` rows, `actual_workspace_totals`, `all_target_outcomes`, required test names, and command stream references to the exact local execution receipt. The regression v3 receipt has also been read to confirm the distinct CLI path and same pinned bytes.

Only Python AST parsing and a symbol-table unresolved-global audit were run on this verifier candidate. It was not executed. No Git, Cargo, network, JDK, JADX, or product CLI command was run.

The root-owned final invocation should use the actual product commit/run ID and fresh receipts, with inputs shaped as follows:

```text
python3 /private/tmp/jarde-hoisted-iterator-own-ci-verifier-prep-luna-v2/verify-own-ci.py \
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
