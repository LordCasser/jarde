# Private assignment clean-main auditor prep v2

This is a new, minimal follow-up to the reviewed v1 auditor. V1 remains unchanged. V2 has not been executed; full validation and the final external clean-main audit are still pending.

V2 recognizes that the focused and final build receipts bind the same immutable executable path: `/private/tmp/jarde-iterator-assignment-candidate-cli-root-v1`. It requires both receipts to identify that exact path and enforces the pinned 61,314,560-byte size, mode `0555`, and SHA-256 `526e2e5a6db8a798a92d491123264bbb4d7105c0c1c3c2c9495c1b316eb89dd5`. It retains the final receipt's exact build-command and byte-identity checks.

The own-CI acceptance path/SHA/schema/status remain actual invocation inputs. Schema/status must equal the successful v2 contract `iterator-assignment-own-ci-acceptance-v2` / `accepted_local_own_ci_observation`. The audit also binds its `product_commit`, complete local-and-both-CI-seed named-outcome comparison flag, workspace execution SHA, workspace named-manifest SHA, and final build record SHA to the supplied product commit and pinned receipts.

All other clean-main, source-closure, runner/acceptor, 15-checkout, real-remote, detached auxiliary worktree, main ancestry, target absence, and guarded-command checks are inherited from v1. The five validation receipt identities and own-CI evidence identity remain caller-supplied; no future receipt path, digest, product commit, or docs-close commit is guessed.

Static Python AST parsing passed. No auditor, Git, Cargo, Java, or CLI command was run.
