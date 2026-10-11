# Hoisted iterator full validation preparation

This private package contains two root-runner candidates and independent read-only acceptors. It has not run Cargo, OpenSpec, the CLI, or a toolchain.

`workspace_runner.py` takes the completed regression receipt and its SHA, then derives the Cargo target inventory and batches from fresh `cargo metadata`. It requires the 4,732-entry source closure frozen by the actual regression receipt, the already runtime-tested CLI bytes, no existing `target`, at least 5 GiB free space, a 1 GiB target ceiling, the default Rust stack, seed `5350648285461741569`, and a one-second guard poll. It captures every command's stdout/stderr and per-target named test outcomes, removes completed test executables, and always attempts guarded `cargo clean` after arming cleanup.

`accept_workspace.py` independently parses the raw Cargo streams using the pinned committed CI helper. It derives totals and the command plan from the captured metadata, validates all targets and all named outcomes, confirms the 19 required tests, and writes `workspace-acceptance.json`. Its `all_target_outcomes` entries have `key` (`[package, target, kind]`), `source_path`, `counts` (`[passed, failed, ignored]`), and every `{name, status}` outcome. `required_named_tests` and `required_named_tests_passed` remain in the runner receipt. Counts such as target, batch, command, passed, and ignored totals are never prefilled from estimates.

`final_build_runner.py` accepts the regression, workspace, and workspace-acceptance receipts with their SHA-256 values. It runs the exact CI fmt and clippy commands, global strict OpenSpec validation, and locked CLI build; it requires the fresh CLI to be byte-identical to the CLI already exercised by runtime tests, then guarded-cleans. `accept_final_build.py` independently validates the receipt chain, all five commands and raw streams, resource limits, source closure, and byte identity.

The root agent owns execution. Suggested sequence:

1. Run `workspace_runner.py --regression-receipt <path> --regression-receipt-sha256 <sha>`.
2. Run `accept_workspace.py --workspace <workspace-run> --out <new-manifest> --regression-receipt <path> --regression-receipt-sha256 <sha>`.
3. Run `final_build_runner.py` with the regression receipt, workspace `execution.json`, and acceptance manifest, each paired with its SHA-256.
4. Run `accept_final_build.py` with the four receipts/manifests and their SHA-256 values.

This is a local validation candidate only. No full-workspace or final-validation pass is claimed here.
