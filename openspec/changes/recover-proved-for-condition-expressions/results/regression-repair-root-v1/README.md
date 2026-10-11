# For-condition regression archive prep

This directory contains a verified, private copy plan for the four root executions and the two private F1 test candidates. No archive copy was performed while preparing this package. The shared repository was not modified.

## Verified execution outcomes

- **workspace v1:** the first lib/examples batch completed with 224 passed and 0 failed. The run then stopped because the invocation script refused to remove the out-of-scope `target/debug/examples/navigate_artifact-…` executable. The workspace matrix did not complete.
- **workspace v2:** 30 earlier test batches completed before command 32 failed. Its raw output records the stale F1 `while` literal assertion failing after the generated F1 headers had become `for` loops. Command 33 `cargo clean` succeeded. These batches are partial progress, not a full-workspace pass.
- **F1 regression v1:** the focused five-test run had 4 passed and 1 failed. The failing assertion was the old F2 `stepTwoWithQuote` while expectation; the raw failure shows the generated `for (local2 = 0; local2 < arg0.length; local2 = local2 + 2)` and the original body/try-catch remaining in place. In that same test, the F1 full-class comparison ran before the F2 assertion and completed.
- **F1 regression v2:** the focused five-test suite passed 5/5. Formatting and cleanup commands succeeded, the target directory was absent after cleanup, both source pin maps remained equal at 4718 entries, and the invocation script SHA stayed unchanged. This focused regression is not a fresh full-workspace run.

For all four executions, each raw stdout/stderr stream was checked against its command receipt for exact byte length and SHA-256. Existing pin entries in both private F1 prep `pins.json` files were also checked against the referenced files. See `manifest.json` for the full per-file inventory and audit results.

## Archive contents and copy

`copy-plan.json` lists every execution receipt, raw stream, metadata file, invocation script, and file in the F1 prep v1/v2 directories. The plan uses paths relative to a destination root chosen by the reviewer. `copy_archive.py` validates sources before copying and refuses to overwrite a differing destination file. It defaults to a dry run; copying requires both `--destination DIR` and `--apply`.

The plan does not include this prep directory's README, manifest, plan, or copy script, avoiding recursive self-pinning. The reviewer can copy these four documentation/control files separately after reviewing them.
