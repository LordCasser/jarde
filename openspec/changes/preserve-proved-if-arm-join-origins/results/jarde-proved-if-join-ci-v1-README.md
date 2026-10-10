# Private If-arm-join CI replay draft

This private draft adapts the accepted For CI capture and verifier for `preserve-proved-if-arm-join-origins`. It writes CI evidence and acceptance only under `/private/tmp/jarde-proved-if-join-ci-v1/`; it does not change repository files. The capture script uses `gh run view` only when root explicitly runs it after review. This preparation did not call Git, GitHub CLI, Cargo, JDK tools, or the candidate CLI.

The verifier reuses the already pinned static-CI and integer-array verifier functions, with the exact gateway and fingerprint raw-output checks retained. It derives all 12 gateway test names from the submitted product commit's Rust test source, requires the three known If-join tests, and takes the required new region unit-test name as an explicit argument because that name must come from the final product. It validates the two fixed seeds, 354 workspace result records and 97 ignored results from captured raw logs, while expected library, gateway, and aggregate passed counts are explicit arguments. The build check verifies the live candidate CLI and metadata, build-vN execution, matching root runner, v9 guarded-runner template, and every source/test/canonical pin against both the product commit's Git blobs and current files. Canonical inputs include both `include_bytes!` and `include_str!`; the expected union is 27 canonical files, for 52 total pins (17 product, 8 test, 27 canonical).

After root freezes the If CLI, metadata, build, final product commit, CI run, and exact internal test name, root can run the capture script with actual values:

```sh
python3 /private/tmp/jarde-proved-if-join-ci-v1/capture-ci-product-root-v1.py \
  --product-commit ACTUAL_PRODUCT_COMMIT \
  --run-id ACTUAL_CI_RUN_ID \
  --expected-lib-count ACTUAL_LIB_COUNT \
  --expected-gateway-count ACTUAL_GATEWAY_COUNT \
  --expected-total-passed ACTUAL_TOTAL_PASSED
```

Then root can run the independent verifier with the same frozen values plus the actual metadata/CLI hashes, build path/schema, and internal test name:

```sh
python3 /private/tmp/jarde-proved-if-join-ci-v1/verify-ci-product-root-v1.py \
  --product-commit ACTUAL_PRODUCT_COMMIT \
  --run-id ACTUAL_CI_RUN_ID \
  --metadata-sha256 ACTUAL_METADATA_SHA256 \
  --cli-sha256 ACTUAL_CLI_SHA256 \
  --build-path openspec/changes/preserve-proved-if-arm-join-origins/results/validation-build-root-vN/execution.json \
  --build-schema preserve-proved-if-arm-join-origins-validation-build-root-vN \
  --expected-lib-count ACTUAL_LIB_COUNT \
  --expected-gateway-count ACTUAL_GATEWAY_COUNT \
  --expected-total-passed ACTUAL_TOTAL_PASSED \
  --required-lib-test-name region::tests::ACTUAL_TEST_NAME
```

All placeholder values must be replaced with root-verified artifacts and counts. This draft has not captured a run or accepted any candidate.
