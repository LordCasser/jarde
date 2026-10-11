# Static review — clean-main audit v2

- Derived from private clean-main audit prep v1; all original versions remain intact.
- Target runner `/private/tmp/jarde-for-condition-final-lint-build-root-v3.py`, CLI `/private/tmp/jarde-for-condition-final-cli-root-v3`, and receipt `/private/tmp/jarde-for-condition-final-lint-build-root-v3/execution.json` are not available yet. The exact root-v3 runner SHA-256 is `85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e`. CLI SHA/size, receipt SHA, and HEAD are not predicted: the bound receipt supplies CLI identity and actual CLI bytes are checked against it; receipt SHA and HEAD are required runtime inputs.
- Receipt schema remains `for-condition-final-lint-build-root-v1`; source pins must be 4,718 and `source_pins_before == source_pins_after`. The five exact argv rows include CI-workflow-derived clippy and all five require real exit 0/no guard stop. No nonexistent `source_pins_after_format` field is read.
- Existing v1 checks are retained for exact Git tree source closure, batched live blob OIDs, real origin/main equality, all 15 clean worktrees, non-root detached state, ancestor proof, both target directory absences, and 5-GiB free space.
- Root-v3 workspace failed at command 75 (`cargo test ... --test recover_unicode_identifiers ...`) because the frozen F1 golden still expects two while loops. Command 76 clean succeeded; target was removed and 4,718 before/after source pins matched. Do not archive that run as full-workspace success.
- Static AST/symbol-table and file-pin checks only. No Git/audit/toolchain/CLI was run.
