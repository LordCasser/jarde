# Integer-array CI verifier

`verify-int-array-ci-product-luna-v1.py` is a prepared, read-only acceptance verifier for the frozen integer-array candidate. It accepts exactly a product commit and CI run ID. It loads the SHA-pinned static-initializer CI verifier and calls its `verify_ci` function without calling that verifier's `main`, then checks the integer feature's exact CLI metadata, frozen build execution and raw command streams, submitted Git blobs, and all ten integer-array tests in each of the two fixed-seed workspace runs.

Run it from the repository with the pinned Python dependency:

```sh
uv run --no-project --with blake3==1.0.11 python -B \
  openspec/changes/recover-int-array-constant-names/results/verify-int-array-ci-product-luna-v1.py \
  PRODUCT_SHA RUN_ID
```

The verifier writes `ci-product-v1/acceptance-int-array-v1.json` and refuses to overwrite an existing result. It has not been run against a captured CI run; preparing this verifier does not constitute CI acceptance.
