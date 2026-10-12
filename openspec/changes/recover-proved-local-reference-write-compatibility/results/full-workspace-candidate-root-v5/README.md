# Fresh full workspace v5

Root and independent acceptor completed all 96 fresh test batches, 363 metadata targets, 99 commands and 198 raw streams. Input closure contains 4784 hashes, unchanged before/after/live; target absent before and after. Default Rust stack, fixed seed 5350648285461741569, 5GiB minimum free and 1GiB maximum target with one-second guard; fixed final cargo clean exception only. Earlier four failed runs remain separate and were not stitched into this result.

Actual totals: `{"passed": 3489, "failed": 0, "ignored": 97, "named_test_outcome_count": 3586, "target_summary_count": 363, "per_target_named_outcomes_present": true}`. This acceptance covers the local workspace only; final Clippy/strict/CLI, product CI and clean-main delivery remain open.
