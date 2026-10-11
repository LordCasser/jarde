# Hoisted iterator full-class comparison wrappers

This private preparation contains two launchers for root review:

- `run_current.py` captures the frozen `TestNestedLoops` baseline (29 collector commands).
- `run_controls.py` captures the two frozen hoisted-iterator controls (43 collector commands).

Both use collector v3 and checker v7, the pinned candidate CLI, the established JDK/JADX/guard/scanner/classfile-parser configuration from the `TestNestedLoops2` baseline receipt, the dynamically reconstructed 4,729-file source closure, and the existing 5 GiB workspace / 1 GiB target / 1-second process-group guard configuration. Each refuses to start if the workspace target exists, verifies frozen input and source pins before capture, checks raw streams, invokes the checker when an execution receipt exists, and records post-run source pins and target absence. Each refuses to overwrite its named output.

The focused CLI receipt remains unchanged at schema `hoisted-iterator-focused-root-v2`, status `passed_focused_candidate_only`. Collector v3 expects a `passed_candidate_build_only` receipt, so each launcher creates its own explicit derived subreceipt from the focused receipt. It verifies the parent receipt SHA/schema/status, exact candidate CLI path/size/SHA/mode, equality of before/after source pins, target absence, and exactly one successful unguarded `cargo build -p jarde-cli --bin jarde-cli --locked` at command index 6. It also revalidates both raw streams for that command against their recorded paths, byte counts, and SHA-256 values. The derived receipt records the parent status verbatim and does not edit or relabel the parent receipt.

The controls launcher classifies a capture with compile failures as `capture_checked_with_compile_failures`; those failures remain visible in the launch receipt. Both launch receipts retain actual command counts and raw-stream pins.

`pins.json` records the wrapper bytes and the immutable inputs/config/tools they bind. Derived subreceipts and collector outputs are runtime artifacts and therefore have no prepared hash.

Static preparation checks only: both launcher files passed `ast.parse` and `symtable.symtable`. Neither launcher, collector, checker, nor any compiler/toolchain was executed as part of this preparation.
