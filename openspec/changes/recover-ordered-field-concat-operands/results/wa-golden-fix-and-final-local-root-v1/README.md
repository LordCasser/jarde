# WA golden fix and final local acceptance

Workspace-v7 found one stale line in WA.main's historical full-class expected text. The original historical file remains unchanged. The permanent test requires the old literal exactly once, substitutes the precise recovered field concat, and compares the entire class; all nine accessor bodies remain exact. See [failed workspace](../full-workspace-wa-golden-failure-root-v7/README.md).

Fresh focused-v11 completed all eight commands: fmt, exact current CI Clippy, 391 library tests, ten integration targets (including all nine accessor tests and complete-class compile/runtime), completed-package cleanup, CLI build, global strict OpenSpec, and finally exact cargo clean. Total 527 passed, zero failed, four ignored; default stack, resource guards and target removal passed. All 4,854 source pins match before/after.

The CLI is byte-identical to focused-v10 (SHA c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd); the sole source delta is the WA test expectation. The previously accepted [full-class matrices](../wrap-load-regression-fix-and-final-local-root-v1/README.md) therefore still apply to this exact product binary. This does not accept the full workspace, own product CI, push, or clean main.

[Compressed capture](capture.tar.gz) contains the receipt, all 16 raw streams, independent acceptance, exact tools, corrected test and binary-identity check. [Manifest](manifest.json) records every member; root reopened and verified every byte/SHA. The CLI binary is omitted to avoid retaining duplicate build output.
