# Integer-array CI verifier v2

`verify-int-array-ci-product-luna-v2.py` is prepared for the v2 frozen CLI and ten-command guarded build. It retains the v1 pinned CI helper and product Git-blob checks, and requires all eleven integer-array tests—including the try-resource/catch-name budget test—in both fixed-seed workspace runs. It verifies all twenty build raw streams and the v2 CLI, metadata, build record, and source pins.

Run after the matching CI evidence has been captured:

```sh
uv run --no-project --with blake3==1.0.11 python -B \
  openspec/changes/recover-int-array-constant-names/results/verify-int-array-ci-product-luna-v2.py \
  PRODUCT_SHA RUN_ID
```

The output is `ci-product-v1/acceptance-int-array-v2.json`; an existing output is never overwritten. This verifier has not been run and does not claim CI acceptance.
