# Proved latch origins CI verifier

This private verifier is prepared for root review and has not run. It adapts the reviewed CI/product verifier from the previous change to this change's build-v1, frozen candidate, and CI evidence layout. It requires the supplied product commit and run ID as arguments, then verifies that the CI run belongs to that product commit. It does not prefill a future product SHA or run ID.

It binds the candidate metadata SHA-256 to `aeb3a081e3f05e27f64f5afea48d4fc4a2ccaa410209910eb40065436f54e54c`, CLI SHA-256 to `d2d9773d94011a680e18d968ea1b3684036791ceb7dfa769455adf9247f0d33a`, build schema/path to `preserve-proved-for-latch-origins-validation-build-root-v1`, source base to `87090b3b4735693d0930f24f817d19f41cb63d98`, and runner/template identities to the recorded build and pinned v9 template. It verifies all 50 candidate/test/literal-include pins against the supplied commit and current files, all 12 build commands and 24 raw streams, both fixed-seed workspace runs, four successful CI jobs and 52 steps, the nine-test gateway binary, the two region tests, and eleven prior integer regression tests. The expected workspace totals are 3385 passed, 0 failed, 97 ignored over 354 result records for each seed.

Invocation after CI capture (fill in the actual product SHA and Actions run ID):

```text
python3 /private/tmp/preserve-proved-for-latch-ci-verifier-v1/verify-ci-product-root-v1.py PRODUCT_SHA RUN_ID --metadata-sha256 aeb3a081e3f05e27f64f5afea48d4fc4a2ccaa410209910eb40065436f54e54c --build-path openspec/changes/preserve-proved-for-latch-origins/results/validation-build-root-v1/execution.json --build-schema preserve-proved-for-latch-origins-validation-build-root-v1
```

A successful acceptance record is written exclusively to `results/ci-product-v1/acceptance-proved-for-latch-ci-root-v1.json`. This verifier checks this product's own CI result and does not claim a broader class-replay gate.
