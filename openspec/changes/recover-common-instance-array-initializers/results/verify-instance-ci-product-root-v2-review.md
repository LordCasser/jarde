# Instance CI verifier v2 review

This is a minimal, unexecuted successor to `verify-instance-ci-product-root-v1.py`; v1 remains unchanged. It preserves the same fixed product, evidence, and validation scope, and changes only the output/schema version plus two corrections based on the captured JSON shapes.

- The checkpoint execution record stores command output under `command["streams"]["stdout"]` and `command["streams"]["stderr"]`. The v2 reader now verifies those stream records through the existing `record_data` path, preserving byte-count and SHA-256 checks.
- The static eight-leg summary now uses JSON-safe string keys (`family/jdk_leg`) and object values (`bytes`, `sha256`) instead of tuple keys, so final `json.dumps` can serialize the accepted result. The uniqueness and four-family-leg grouping checks remain unchanged.
- The result is written only to `ci-product-v1/instance-ci-acceptance-root-v2.json`, and the schema is `instance-array-ci-product-acceptance-root-v2`; v1's result path/schema remain intact.

Static preparation review confirmed the checkpoint command-row schema has `argv`, `cwd`, `duration_seconds`, `exit`, `started_at`, and `streams`; both stream objects contain `bytes`, `path`, and `sha256`. The prior static-leg records have four distinct `(family, jdk_leg)` groups. No verifier, CI query, Git command, or toolchain was run for v2 preparation.
