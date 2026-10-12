# Local-reference-write full-validation preparation v4

This private v4 copies the frozen root-v3 validation chain and preserves the root-v3 directory. The workspace runner changes only the Cargo JSON-stream handling and the candidate-build input schema/path pins. A malformed line beginning with `{` is recorded as a skipped non-JSON diagnostic line; valid Cargo JSON objects continue through the existing compiler-artifact parser. The test-target compiler-artifact set must still be unique and have exact cardinality with the Cargo test summary set. On mismatch, the receipt stores the command exit, failed test names, summaries, artifacts, skipped-line details, and raw stdout/stderr before the runner exits.

The triggering historical input is the immutable failed root-v1 batch 53 stream at `/private/tmp/jarde-local-reference-write-full-workspace-root-v1/53.stdout.raw`. Static parsing found three summary blocks totaling 21 named outcomes and four unique test compiler-artifacts. Line 143 is a lone `{` from the decompiled Java diagnostic and is invalid JSON. The summary includes the failed test `non_monitor_handler_and_back_edge_do_not_borrow_the_monitor_split` and Cargo exit 101. With this parser, the line is skipped and the preserved 4-artifact/3-summary mismatch still rejects the batch with the actual failure and outcome details. This is a parser/diagnostics repair; it does not make that Rust test pass or accept the failed historical workspace run.

The fresh candidate-build root-v3 run actually passed `fmt`, CLI build, and cleanup, with its three source closures matching. Use its actual receipt and CLI pins below. The first fresh full-workspace root-v2 attempt using the v4 runner stopped at its first test command when free space fell below the 5 GiB guard; cleanup succeeded, target is absent, and the 4,784 source pins were unchanged. This is a resource-stop receipt, not a workspace test result. Preserve it and start a new workspace run only after adequate free space is available. The workspace/final runs are separate executions; no future receipt hashes or outcome totals are assumed here.

- Candidate-build runner: `/private/tmp/jarde-local-reference-write-candidate-build-runner-root-v3.py`, SHA-256 `385aac0df99f1286d46c399e691782f5b80a65b5c4d6ae9ea0aa3153e2e5bb85`
- Candidate-build receipt: `/private/tmp/jarde-local-reference-write-candidate-build-root-v3/execution.json`, SHA-256 `006e4bcf82f4212a17641f2be44f0c48735d01b7309e175a8877c7d4fc6b56c1`
- Candidate CLI: `/private/tmp/jarde-local-reference-write-candidate-cli-root-v3`, SHA-256 `4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95`
- Candidate schema/status: `local-reference-write-candidate-build-root-v3` / `passed_candidate_build_only`

Run the workspace from a fresh, absent `target` with at least 5 GiB free:

```text
python3 /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/workspace_runner.py \
  --candidate-build-receipt /private/tmp/jarde-local-reference-write-candidate-build-root-v3/execution.json \
  --candidate-build-receipt-sha256 006e4bcf82f4212a17641f2be44f0c48735d01b7309e175a8877c7d4fc6b56c1 \
  --candidate-cli /private/tmp/jarde-local-reference-write-candidate-cli-root-v3 \
  --candidate-cli-sha256 4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95 \
  --out /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/workspace-run-v4
```

After an actual passing run, independently accept it using the actual receipt and candidate-build inputs:

```text
python3 /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/accept_workspace.py \
  --workspace /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/workspace-run-v4 \
  --out /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/workspace-acceptance-v4.json \
  --candidate-build-receipt /private/tmp/jarde-local-reference-write-candidate-build-root-v3/execution.json \
  --candidate-build-receipt-sha256 006e4bcf82f4212a17641f2be44f0c48735d01b7309e175a8877c7d4fc6b56c1
```

The final runner and acceptor use the same v4 workspace source chain and candidate-build v3 schema. Their argument forms remain the frozen root-v3 forms; pass the actual workspace execution and acceptance paths/hashes, and the fixed candidate-build receipt/CLI pins. Final validation retains the exact CI clippy command, strict OpenSpec validation, CLI byte-identity check, default Rust stack, 5 GiB free / 1 GiB target / one-second guards, and exact cleanup-only `cargo clean` exception.

After a fresh passing workspace run and independent acceptance, run final validation with actual workspace receipt and acceptance hashes:

```text
python3 /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/final-build-runner.py \
  --candidate-build-receipt /private/tmp/jarde-local-reference-write-candidate-build-root-v3/execution.json \
  --candidate-build-receipt-sha256 006e4bcf82f4212a17641f2be44f0c48735d01b7309e175a8877c7d4fc6b56c1 \
  --workspace-receipt <ACTUAL_WORKSPACE_EXECUTION_JSON> \
  --workspace-receipt-sha256 <ACTUAL_WORKSPACE_RECEIPT_SHA256> \
  --workspace-acceptance <ACTUAL_WORKSPACE_ACCEPTANCE_JSON> \
  --workspace-acceptance-sha256 <ACTUAL_WORKSPACE_ACCEPTANCE_SHA256> \
  --candidate-cli /private/tmp/jarde-local-reference-write-candidate-cli-root-v3 \
  --candidate-cli-sha256 4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95 \
  --out /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/final-build-v4
```

Then independently accept the final receipt with all actual receipt hashes:

```text
python3 /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/accept-final.py \
  --candidate-cli /private/tmp/jarde-local-reference-write-candidate-cli-root-v3 \
  --candidate-cli-sha256 4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95 \
  --final-receipt /private/tmp/jarde-local-reference-write-full-validation-prep-luna-v4/final-build-v4/execution.json \
  --final-receipt-sha256 <ACTUAL_FINAL_RECEIPT_SHA256> \
  --candidate-build-receipt /private/tmp/jarde-local-reference-write-candidate-build-root-v3/execution.json \
  --candidate-build-receipt-sha256 006e4bcf82f4212a17641f2be44f0c48735d01b7309e175a8877c7d4fc6b56c1 \
  --workspace-receipt <ACTUAL_WORKSPACE_EXECUTION_JSON> \
  --workspace-receipt-sha256 <ACTUAL_WORKSPACE_RECEIPT_SHA256> \
  --workspace-acceptance <ACTUAL_WORKSPACE_ACCEPTANCE_JSON> \
  --workspace-acceptance-sha256 <ACTUAL_WORKSPACE_ACCEPTANCE_SHA256>
```

Frozen script pins (`pins-root-v4.json`):

- `workspace_runner.py`: `8f427510b3bb30d6c32c02cb3fd063ee87d441a8c9e51f0f43516f84f7e64911`
- `accept_workspace.py`: `e263e38230c4a95bcd40f21746f9e4060a06182bf57f196b30edb3c7ac454ef3`
- `final-build-runner.py`: `c7525a6b8ad0853204944a09e3d8fb6a58fcfd87287aad12d86adde3e4aaff68`
- `accept-final.py`: `3a7c461e15f2166ed482194ee0af7809d69da0107bf9d41f60c74f189f2b494d`

The runner and workspace acceptor are frozen for the root rerun. This v4 preparation performed only AST and historical raw-stream parsing checks; it did not run Cargo, Java, either acceptor, or the verifier. It makes no claim that the next workspace run or the known failing scope test passes.
