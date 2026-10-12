# Workspace-v8 reader census failure

The fresh run ended with exit 1 at command 93 in the first seed. The sole named failure was `classfile::tests::repository_class_fixtures_validate_without_false_target_rejections`; 177 other reader tests passed. Every fixture class, member walk, method body and control-flow target had passed before the final census comparison. The recorded population was `(1163, 5157, 501, 3279, 10)`, versus the old `(1140, 5018, 474, 3141, 10)`. This is a stale census assertion, not a fixture decode rejection.

All 95 records, exact runner, metadata and the reader source matching the failure receipt are preserved in [capture.tar.gz](capture.tar.gz). All referenced raw streams are included. Source pins were unchanged; no resource guard fired; command 94 was successful exact cargo clean and target removal. This failed run does not accept the whole workspace or second seed.

Root reopened every [manifest](manifest.json) member and verified bytes/SHA. The newly added 23 classes and their census delta must be independently measured before updating the expected tuple; structural validation assertions remain unchanged.
