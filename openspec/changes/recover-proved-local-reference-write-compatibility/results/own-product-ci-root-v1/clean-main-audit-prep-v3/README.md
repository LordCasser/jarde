# Local-reference-write clean-main audit preparation

This private, read-only audit v3 adapts the accepted iterator-assignment clean-main audit to the local-reference-write validation chain. It retains the original source-closure reconstruction and product/docs/live Git blob comparison, final build raw-stream checks, the 15-physical-checkout policy (one root checkout and 14 detached clean main ancestors), main/origin/remote checks, and CLI byte binding. It does not unlock, remove, or alter worktrees.

The frozen implementation input is `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-iterator-assignment-foreach/results/own-product-ci-clean-root-v1/audit-clean-main-v2.py`. The new script only adapts the receipt chain and pins: candidate-build receipt schema v3 from root-v5, workspace v2, final-build v2, workspace/final acceptance v2, own-CI acceptance v3, and the local-reference-write CLI.

Known fixed pins:

- Candidate build runner: `/private/tmp/jarde-local-reference-write-candidate-build-runner-root-v5.py`, SHA-256 `bbc5232b45c3703b17d104eeaca44d6e19374a2363319c27e1acb3b6a1a53783`.
- Candidate build CLI: `/private/tmp/jarde-local-reference-write-candidate-cli-root-v5`, SHA-256 `4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95`, 61,486,096 bytes, mode `0555`.
- Full validation v4 scripts: `workspace_runner.py` SHA `8f427510b3bb30d6c32c02cb3fd063ee87d441a8c9e51f0f43516f84f7e64911`; `accept_workspace.py` SHA `e263e38230c4a95bcd40f21746f9e4060a06182bf57f196b30edb3c7ac454ef3`; `final-build-runner.py` SHA `c7525a6b8ad0853204944a09e3d8fb6a58fcfd87287aad12d86adde3e4aaff68`; `accept-final.py` SHA `3a7c461e15f2166ed482194ee0af7809d69da0107bf9d41f60c74f189f2b494d`.
- Schemas: `local-reference-write-candidate-build-root-v3` / `passed_candidate_build_only`; `local-reference-write-full-workspace-root-v2` / `passed`; `local-reference-write-full-workspace-local-acceptance-v2` / `accepted_local_workspace_only`; `local-reference-write-final-lint-build-root-v2` / `passed_candidate_build_only`; `local-reference-write-final-validation-acceptance-v2` / `accepted_local_full_validation_only`; own-CI `local-reference-write-own-ci-acceptance-v3` / `accepted_local_own_ci_observation`.

The candidate-build, workspace, final, and acceptance receipt paths and hashes are command-line inputs. The product commit, post-documentation HEAD, own-CI acceptance path/hash/schema/status, and exact CI capture directory, its `capture-execution-root-v2.json` hash, and run ID are supplied at execution time; the audit hashes that file and compares all three with the accepted own-CI record. The audit checks all bindings against the real receipts; it does not predict future receipt hashes or commits. Source count is derived from the candidate receipt and required to match the workspace/final closures, their accepted manifests, and the live source tree.

The fresh candidate-build root-v5 run passed and records 4,784 source paths; its receipt protocol schema remains candidate-build v3. Workspace execution and acceptance remain runtime inputs; this preparation has not run the audit. The auditor must be run only after the workspace run and acceptance, final build and acceptance, own-CI acceptance, and documentation push are complete. Then pass the actual resulting paths and SHA-256 values below. Keep output outside the repository.

```text
python3 /private/tmp/jarde-local-reference-write-clean-main-audit-prep-luna-v3/audit-clean-main.py \
  <OUTSIDE_REPO_AUDIT_OUTPUT_DIR> <ACTUAL_PRODUCT_COMMIT> <ACTUAL_POST_DOCS_HEAD> \
  --candidate-build-receipt /private/tmp/jarde-local-reference-write-candidate-build-root-v5/execution.json \
  --candidate-build-receipt-sha256 4124576e8d01fa87cb1bb33267de1758ca4367fec68b8f99195b1b81d095aaa1 \
  --workspace-receipt <ACTUAL_WORKSPACE_EXECUTION_JSON> \
  --workspace-receipt-sha256 <ACTUAL_WORKSPACE_RECEIPT_SHA256> \
  --workspace-acceptance <ACTUAL_WORKSPACE_ACCEPTANCE_JSON> \
  --workspace-acceptance-sha256 <ACTUAL_WORKSPACE_ACCEPTANCE_SHA256> \
  --final-build <ACTUAL_FINAL_BUILD_EXECUTION_JSON> \
  --final-build-sha256 <ACTUAL_FINAL_BUILD_RECEIPT_SHA256> \
  --final-acceptance <ACTUAL_FINAL_ACCEPTANCE_JSON> \
  --final-acceptance-sha256 <ACTUAL_FINAL_ACCEPTANCE_SHA256> \
  --own-ci-acceptance <ACTUAL_OWN_CI_ACCEPTANCE_JSON> \
  --own-ci-acceptance-sha256 <ACTUAL_OWN_CI_ACCEPTANCE_SHA256> \
  --own-ci-schema local-reference-write-own-ci-acceptance-v3 \
  --own-ci-status accepted_local_own_ci_observation \
  --ci-capture-path <ACTUAL_CI_CAPTURE_DIRECTORY> \
  --ci-capture-execution-sha256 <ACTUAL_CI_CAPTURE_RECEIPT_SHA256> \
  --ci-run-id <ACTUAL_CI_RUN_ID>
```

This audit v3 is a private adaptation of clean-main audit v1 and the prior local-reference-write audit v2; its inputs are the actual root-v5 candidate run, same-byte CLI, and future actual workspace/final/own-CI receipts. The auditor was not executed. No Git mutation, network request, Cargo/JDK/toolchain action, or receipt acceptance was performed in this preparation.
