# Current-product own-CI acceptance adapter

This private verifier binds the exact `--product-commit` and `--run-id` to a fresh local full-workspace execution, the final v2 candidate CLI build record, and the captured GitHub CI logs. It reuses the reviewed v5 conditional-CI parsing helpers, the frozen `build_evidence.verify_build`, and the current CI capture runner. It does not use prior CI acceptance records.

The workspace evidence path and schema are fixed to `/private/tmp/jarde-compound-full-workspace-root-v3` and `compound-full-workspace-batched-root-v1`. Acceptance requires status `passed`, no carried execution, unchanged source pins, all 359 Cargo targets executed once, and every source pin bound to both the provided product commit and live files. The CI check requires exactly four successful jobs and 52 successful expanded steps, then compares both configured seeds and all per-target named outcomes with the local run. A `***` label is retained in capture raw logs and maps only when exactly one same-status local test name matches; each mapping is recorded in the acceptance output.

The verifier also reads the current-product test source text from Git and requires the checklist regressions, the jarde-jvm lib result 243/0/0, jarde-java lib 351/0/0, public target 8/0/0, and the renamed NullThenBuilder test. Workspace totals and named-outcome counts are reconstructed from raw outputs rather than predeclared.

The runner SHA was checked against the actual private runner bytes: `2ac64ccdf7ed6c8f90f37b8ae34693eb16206cba493ca4a3e261de993a08235b`. The checklist is pinned at `c64e9928cfeef595cf4c1132ceb43b0287773fe8e84c47c3223a94a54a108563`; the v5 helper at `bea37c7d4d37182168a7c1cbbde3ee31c94d61f6bc32667bedbdd02ce7e5b0ca`; and the build-evidence helper at `8b06e82de2fc696b3b82d32b444026b57a2d28f8c5cc51fef87d4ffcbe4ffec3`.

Root invocation after the fresh workspace and CI capture are complete:

```sh
PRODUCT_COMMIT='<actual 40-hex product commit>'
RUN_ID='<actual completed Actions run id>'
python3 /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py \
  --product-commit "$PRODUCT_COMMIT" \
  --run-id "$RUN_ID" \
  --out /private/tmp/jarde-compound-current-product-ci-capture
python3 /private/tmp/jarde-compound-own-ci-verifier-luna-v1/verify-own-ci.py \
  --product-commit "$PRODUCT_COMMIT" \
  --run-id "$RUN_ID" \
  --capture /private/tmp/jarde-compound-current-product-ci-capture \
  --out /private/tmp/jarde-compound-current-product-ci-acceptance.json
```

This package contains no future product commit, CI run ID, aggregate test total, or acceptance result. Preparation ran Python AST parsing and read-only source-text checks only; it did not run Git, Cargo, JDK, JADX, CLI, or alter the repository or raw evidence. The currently observed workspace execution was still `running` at preparation time, so the verifier will require root to rerun acceptance after that record is complete and `passed`.
