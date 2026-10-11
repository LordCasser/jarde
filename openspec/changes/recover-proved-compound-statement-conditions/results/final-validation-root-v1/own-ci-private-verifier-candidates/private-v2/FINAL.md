# Current-product own-CI acceptance adapter v2

v2 revises v1 after the root-v3 workspace execution was found failed. The workspace evidence directory is now an explicit required `--workspace` argument, and the known failed `/private/tmp/jarde-compound-full-workspace-root-v3` directory is rejected. Acceptance still requires the pinned 95-batch runner, `passed`, no carried execution, unchanged source pins, and all 359 targets exactly once. The verifier also checks the new source and local outcomes for `cross_case_goto_target_owns_the_shared_forward_continuation` and `looping_arm_backedge_does_not_gain_a_forward_join`.

The commit and run ID remain runtime arguments. No future workspace path, commit, run ID, outcome, or aggregate total is asserted. The acceptance output derives totals and named outcomes from raw records. The CI capture uses the existing v3 capture runner; masked public labels only bind to a unique same-status local outcome, and raw logs are kept intact.

After root has produced a successful fresh full-workspace directory and captured the actual product CI run:

```sh
PRODUCT_COMMIT='<actual 40-hex product commit>'
RUN_ID='<actual completed Actions run id>'
WORKSPACE_DIR='<actual fresh full-workspace evidence directory, not failed root-v3>'
python3 /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-conditional-switch-fallthrough/results/capture-conditional-ci-root-v3.py \
  --product-commit "$PRODUCT_COMMIT" \
  --run-id "$RUN_ID" \
  --out /private/tmp/jarde-compound-current-product-ci-capture
python3 /private/tmp/jarde-compound-own-ci-verifier-luna-v2/verify-own-ci.py \
  --product-commit "$PRODUCT_COMMIT" \
  --run-id "$RUN_ID" \
  --capture /private/tmp/jarde-compound-current-product-ci-capture \
  --workspace "$WORKSPACE_DIR" \
  --out /private/tmp/jarde-compound-current-product-ci-acceptance.json
```

Preparation only made a private source copy, parsed Python AST, and checked that the two requested test identifiers are present in the verifier. No Git, Cargo, JDK, JADX, or CLI command ran; no repository or raw evidence was changed.
