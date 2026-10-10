# Multiply validation/build runner v2 review

Immutable v2 is a deterministic follow-up to v1. v1 remains unchanged; neither script has been executed. It fixes exactly the two review findings: (1) `CI_CLIPPY_ALLOW_LINTS` now contains the workflow's full 29-name allowlist, compared item-for-item with `.github/workflows/ci.yml`; (2) `PRODUCT_PATHS` is restored to the exact 10-path set from the frozen integer runner: `Cargo.lock`, `asserts.rs`, `ast.rs`, `build.rs`, `emit.rs`, `field.rs`, `init.rs`, `report.rs`, `src/class_source.rs`, and `src/facade.rs`. Thus relevant CLI plumbing remains source-pinned along with the multiply implementation.

No command, disk guard, runtime `--source-base` check, test filter, expected summary, source-before/after check, metadata marker, or raw-output capture was changed. The required source-base remains a caller-supplied 40-character SHA; the script checks HEAD against it and does not embed a guessed value. To keep versions non-overwriting, v2 uses `validation-build-root-v2`, `/private/tmp/jarde-field-multiply-cli-v2`, `candidate-cli-v2.json`, and a v2 execution schema. The multiply-only `uncommitted_field_multiply_product` marker remains.

Validation here was static only: Python in-memory compilation passed; AST comparison confirmed the script has exactly the same 10 product pins as the frozen integer runner; extracting `-A clippy::...` entries from the actual CI Clippy step yielded 29, exactly matching v2. No Rust, Cargo, rustfmt, Git, JDK, JADX, or Jarde command was run.

`run-validation-build-luna-v2.diff` is the exact unified delta against v1.
