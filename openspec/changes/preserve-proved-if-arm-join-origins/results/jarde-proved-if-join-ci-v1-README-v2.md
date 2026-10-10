# Private If-arm-join CI replay draft v2

This revision keeps the earlier private CI draft intact and binds the actual frozen If CLI, metadata, build execution, source base, and runner reported by root. It writes only under `/private/tmp/jarde-proved-if-join-ci-v1/ci-product-v2` and its v2 acceptance path. Root must adjust paths when copying these reviewed scripts into the change results directory.

The verifier uses the actual CLI SHA `7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6`, metadata SHA `6f785a03e50565bc5d90bd6a6ae85f1bb789e31647811421e5f21d41ee7030ca`, build-v1 execution SHA `7f0ac81bd57faa67082bdc1f0231650a96c9d2658d00690f086c4039c49d6458`, source base `d9855c2764860c7c2e20e40e700abeaa59f00a55`, and live validation-runner SHA `f139f117354f4b96762737190c03a025f3a02d3efc078062b715c63d83518f6b`. It checks the new library test `build::tests::exception_if_join_transfer_rejects_a_canonical_exception_edge` against `crates/jarde-java/src/build.rs` and requires the frozen build summary to contain that test plus the two existing region tests. For the gateway build summary it requires exactly the three known If-join test names. In both workspace seed logs it independently checks every one of the 12 gateway test names derived from the submitted product commit, so the required-summary subset does not hide failures in the other tests.

The verifier retains the 4-job/52-step CI checks, two fixed seeds, exact 354 workspace result records and 97 ignored results from the raw logs, and the corrected fingerprint block check. It checks all 52 product/test/canonical source pins against both the product commit's Git blobs and live files; canonical paths include both `include_bytes!` and `include_str!`. Expected library, gateway, and total passed counts remain explicit arguments so root supplies the final observed values. This draft has not called Git, GitHub CLI, Cargo, JDK tools, or the candidate CLI.

After root provides the actual CI product commit/run ID and confirms the expected totals, capture the raw CI data:

```sh
python3 /private/tmp/jarde-proved-if-join-ci-v1/capture-ci-product-root-v2.py \
  --product-commit ACTUAL_PRODUCT_COMMIT \
  --run-id ACTUAL_CI_RUN_ID \
  --expected-lib-count 336 \
  --expected-gateway-count 12 \
  --expected-total-passed ACTUAL_TOTAL_PASSED
```

Then run the independent verifier:

```sh
python3 /private/tmp/jarde-proved-if-join-ci-v1/verify-ci-product-root-v2.py \
  --product-commit ACTUAL_PRODUCT_COMMIT \
  --run-id ACTUAL_CI_RUN_ID \
  --expected-lib-count 336 \
  --expected-gateway-count 12 \
  --expected-total-passed ACTUAL_TOTAL_PASSED
```

Product commit, run ID, and total passed count are intentionally not guessed. The pinned local build establishes the library/gateway counts of 336 and 12; final CI acceptance still requires raw CI evidence and the actual total.
