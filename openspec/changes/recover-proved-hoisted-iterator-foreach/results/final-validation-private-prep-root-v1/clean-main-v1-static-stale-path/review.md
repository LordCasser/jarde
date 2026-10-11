# Private clean-main audit wrapper review

Prepared `/private/tmp/jarde-hoisted-iterator-clean-main-prep-luna-v1/audit-clean-main.py` as a private, read-only Git audit wrapper. It was not invoked. No Git, network, Cargo, JDK, or other toolchain command was run during preparation.

The wrapper binds the final build receipt to schema `hoisted-iterator-final-lint-build-root-v2`, its runner SHA, the actual references list (workspace source-closure runner, guard, scanner), five exact command rows, raw stream hashes, cleanup/target-absent facts, default Rust stack, 5 GiB/1 GiB/1-second guard settings, and the 4,732-entry source closure. It checks the final CLI at the root-v2 runtime path and compares its actual bytes with the separately located root-v3 regression CLI. The two paths are deliberately treated as distinct while requiring matching mode, digest, and bytes.

The receipt is supplied by `EXPECTED_BUILD_RECEIPT_SHA256`; the wrapper also checks its regression, workspace, and workspace-acceptance receipt path/hash/schema/status bindings against the actual files. The workspace helper dynamically rebuilds source pins. It then compares every pinned live source blob against both the product commit and docs-close commit, requires the product commit to be an ancestor of docs close and docs close to be an ancestor of main, and checks `HEAD`, `main`, `origin/main`, the remote main head, and local branch inventory.

Worktree checks require 15 distinct physical paths: the root on `main` and 14 detached auxiliary worktrees. Each must be clean, an ancestor of main, and have neither `target` nor `fuzz/target`. Lock/prunable fields are recorded as evidence, not used to unlock, remove, or bypass a protected worktree. The script checks free disk space and keeps raw stdout/stderr for each future audit command.

Static validation: Python `ast.parse` and `symtable` passed; module globals were checked for unresolved names, accounting for the compiler-generated `__conditional_annotations__` symbol. The final builder and acceptor source were read field-by-field to match their schemas and bindings. This is not a runtime acceptance result.

At review time the final-build receipt and workspace-acceptance file do not yet exist. The existing workspace execution receipt is present but currently has status `failed` (SHA is recorded in `pins.json`); the wrapper therefore cannot accept the current run. A later successful, bound workspace/final-build set is required. The root owns that execution and the clean-main invocation.
