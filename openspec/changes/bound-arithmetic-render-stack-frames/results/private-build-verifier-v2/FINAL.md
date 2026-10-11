# Arithmetic-stack focused CLI build evidence adapter v2

Private read-only adapter for the final arithmetic-stack runner v2. It requires schema `arithmetic-stack-focused-and-cli-root-v2`, build record `/private/tmp/jarde-arithmetic-stack-focused-and-cli-root-v2/execution.json`, CLI `/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2`, and runner SHA-256 `dc0c74fa9e5944a2cc19a7968bed6c66e700ba7f1c798cab6d32167f04f24844`. The exact five commands/phases and CLI mode, live CLI bytes/hash, command cwd/argv/exit/guard state, pinned raw streams, target cleanup, guard/scanner identities, and source before/after equality are checked.

Product pins are derived at validation time from the SHA-pinned runner's actual `compound_paths()` and its guard module. The current dynamic set contains 132 paths, but this count is informational and is not hard-coded. The full `p3_immediate_functional_receivers` test target is checked using raw Cargo summary and the matching runner record; its count is not hard-coded. CLI SHA is an independent caller-provided argv pin.

No build/replay or toolchain was run by this adapter task. The root runner was reported as currently executing; this package does not claim that its output passed verification.
