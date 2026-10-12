# Local-reference-write own-CI verifier preparation v5

This is a private v5 copy of the own-CI verifier prep v4. V4 and its failed invocation are preserved. The product CI run `38197592542` succeeded; its captured evidence is at `/private/tmp/jarde-local-reference-write-own-ci-capture-root-v1` and contains 4 jobs / 52 steps. The v4 verifier attempt failed with `NameError: recorded_runtime_cli is not defined` in `verify_final_build`; the preserved stderr is `/private/tmp/jarde-local-reference-write-own-ci-accept-invocation-root-v1/stderr.raw`.

V5 changes only the three references to that undefined name, replacing them with `recorded_candidate_cli`, which is defined and validated earlier in the same function. It retains all verifier checks and schemas, including the candidate-build receipt/runner/CLI pins below; the receipt protocol schema remains v3 and the output acceptance schema remains v3. Checks cover complete workspace source/include/fixture/example closure against live and product Git blobs; metadata-derived target and batch coverage; per-target raw named outcomes; all required tests from the pinned workspace runner; CI capture/job/step validation; both CI seed runs compared target-by-target; final fmt/clippy/strict/build command and cleanup receipts; and byte identity across candidate-build, workspace and final CLI.

The candidate-build root-v5 inputs are known:

- Receipt `/private/tmp/jarde-local-reference-write-candidate-build-root-v5/execution.json`, SHA-256 `4124576e8d01fa87cb1bb33267de1758ca4367fec68b8f99195b1b81d095aaa1`
- Runner `/private/tmp/jarde-local-reference-write-candidate-build-runner-root-v5.py`, SHA-256 `bbc5232b45c3703b17d104eeaca44d6e19374a2363319c27e1acb3b6a1a53783`
- CLI `/private/tmp/jarde-local-reference-write-candidate-cli-root-v5`, SHA-256 `4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95`
- Candidate receipt schema/status `local-reference-write-candidate-build-root-v3` / `passed_candidate_build_only`

The successful candidate-build root-v5 run reports 4,784 source pins, with its pre-format, post-format, and post-run closures equal. This number is evidence about that candidate receipt only; the verifier reconstructs and checks all closure and test totals dynamically. A prior full-workspace attempt stopped for a historical exact-output assertion in Billing AnalysisSteps. Use the actual completed workspace/final receipts and actual product commit and capture evidence. The product CI capture above is the real successful run; the v4 verifier failure remains preserved. No successful own-CI acceptance receipt has been produced yet. Do not reuse the earlier 73adae CI evidence.

The future invocation is:

```text
python3 /private/tmp/jarde-local-reference-write-own-ci-verifier-prep-luna-v5/verify-own-ci.py \
  --product-commit <ACTUAL_PRODUCT_HEAD> \
  --run-id <ACTUAL_CI_RUN_ID> \
  --capture <ACTUAL_CI_CAPTURE_DIRECTORY> \
  --capture-receipt-sha256 <ACTUAL_CI_CAPTURE_RECEIPT_SHA256> \
  --candidate-build-receipt /private/tmp/jarde-local-reference-write-candidate-build-root-v5/execution.json \
  --candidate-build-receipt-sha256 4124576e8d01fa87cb1bb33267de1758ca4367fec68b8f99195b1b81d095aaa1 \
  --candidate-cli /private/tmp/jarde-local-reference-write-candidate-cli-root-v5 \
  --candidate-cli-sha256 4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95 \
  --workspace <ACTUAL_WORKSPACE_RUN_DIRECTORY> \
  --workspace-receipt-sha256 <ACTUAL_WORKSPACE_RECEIPT_SHA256> \
  --workspace-runner /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/workspace_runner.py \
  --workspace-runner-sha256 8f427510b3bb30d6c32c02cb3fd063ee87d441a8c9e51f0f43516f84f7e64911 \
  --workspace-schema local-reference-write-full-workspace-root-v2 \
  --workspace-named-manifest <ACTUAL_WORKSPACE_ACCEPTANCE_JSON> \
  --workspace-named-manifest-sha256 <ACTUAL_WORKSPACE_ACCEPTANCE_SHA256> \
  --workspace-acceptor /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/accept_workspace.py \
  --workspace-acceptor-sha256 e263e38230c4a95bcd40f21746f9e4060a06182bf57f196b30edb3c7ac454ef3 \
  --final-build <ACTUAL_FINAL_BUILD_EXECUTION_JSON> \
  --final-build-sha256 <ACTUAL_FINAL_BUILD_RECEIPT_SHA256> \
  --build-runner /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/final-build-runner.py \
  --build-runner-sha256 c7525a6b8ad0853204944a09e3d8fb6a58fcfd87287aad12d86adde3e4aaff68 \
  --build-schema local-reference-write-final-lint-build-root-v2 \
  --final-cli /private/tmp/jarde-local-reference-write-candidate-cli-root-v5 \
  --out <NEW_OWN_CI_ACCEPTANCE_JSON>
```

The verifier retains the prior capture schema and collector/parser pins. The capture still must match the actual selected product commit and run ID, and all successful job/step and seed-run checks remain required. The detailed final-build receipt is checked directly, along with exact links to the actual candidate-build receipt and accepted workspace manifest.

Verifier and README SHA-256 values are recorded in `pins.json`. This v5 directory has not been executed; no Cargo, Java, Git, or network operation was run during this repair. The v4 directory and its actual failed invocation remain unchanged.
