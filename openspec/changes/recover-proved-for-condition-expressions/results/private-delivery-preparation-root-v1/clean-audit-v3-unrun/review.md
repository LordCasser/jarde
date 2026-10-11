# Static review — clean-main audit v3

- v2 remains unchanged in `/private/tmp/jarde-for-condition-clean-main-audit-prep-luna-v2/`.
- Reinstates exact known candidate constants: CLI path `/private/tmp/jarde-for-condition-final-cli-root-v3`, 61,213,840 bytes, SHA-256 `51c9a1326031fd10444f7e18a055c6daecef233423c90478257becf24d1e0bd4`, mode `0555`. It checks both actual file identity and exact equality of the receipt `candidate_cli` object.
- Pins final build runner root-v3 SHA-256 `85e25bb6d87a5993305c0fdeb3c58cd9300886701cf8158267df267ce0b92f1e`, guard and scanner. Build receipt SHA and actual audited HEAD remain explicit runtime inputs.
- Preserves v2 source closure and Git audit logic: 4,718 pins and before==after, live content SHA, expected Git tree blob and batched live object IDs, real `origin/main`, 15 individually checked worktrees, clean/non-prunable/unlocked/main-ancestor state, detached non-root checkouts, `target` and `fuzz/target` absence, 5-GiB free-space threshold, and captured command streams.
- Receipt still must carry schema `for-condition-final-lint-build-root-v1`, `passed_candidate_build_only`, runner before/after pins, clean target, focused/final-lint flags, candidate byte-identity flag, five expected command argv records, and all successful guard results.
- Static-only preparation; no audit or Git/toolchain/CLI run occurred. Candidate CLI and receipt availability must be checked at execution time.
