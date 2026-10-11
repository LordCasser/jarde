# For-condition clean-main audit preparation v2

This is a private revision of clean-main audit v1 for the future root-v3 final lint/build runner and CLI. It preserves source closure, live SHA pins, Git tree source blobs/live blob OIDs, remote/main proof, and the exact 15-checkout clean/detached/main-ancestor/target absence checks.

Invocation after the build receipt exists and the product/docs commit is ready:

```text
python3 clean-main-audit.py OUTPUT_DIR EXPECTED_HEAD EXPECTED_BUILD_RECEIPT_SHA256
```

The HEAD and build receipt SHA are runtime values; neither is predicted. The root-v3 final build runner is frozen at SHA-256 `85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e`. The receipt schema remains `for-condition-final-lint-build-root-v1`. The receipt supplies the candidate CLI identity, which the audit checks against the actual root-v3 CLI file and `0555` mode. It requires 4,718 exact source pins and only `source_pins_before == source_pins_after`, then verifies the 5 command rows (fmt, workflow-derived clippy, OpenSpec strict, build, clean) and their actual success/guard results.

The audit was not run. No Git or toolchain was run. Root should use it only after workspace/root-v4 and product/docs work finish and the final build receipt is frozen.
