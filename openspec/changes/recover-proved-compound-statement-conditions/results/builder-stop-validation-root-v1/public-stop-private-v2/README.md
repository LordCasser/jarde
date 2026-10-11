# Compound whole-recovery stop test candidate v2

This private increment is based on the v1 candidate SHA-256 `ff328dccc0e5803e64090d6c8fb35ac2190d8ebbcc2ebf9bd6c9db0c682fba0e`; the underlying tracked test-file base remains SHA-256 `d7445477041184c58a002b09fcf81b38e1c76df8906d619e1e871f1b62bf49de`.

The helper now captures both `analysis_steps` and `output_bytes` from the same budget used by the public recovery call. The late whole-request stop test compares `StopReason::Budget.written` with the measured `output_bytes`, instead of assuming zero. This accounts for bytes charged by emission before the final report is withheld atomically. It still requires `RecoveryContent::NotProduced`, no artifact binding, empty text and map, and an observed physical instruction at the stop BCI. The log records successful usage, derived limit, stop BCI, stopped usage, `written`, and measured output bytes; it does not assign the BCI to a private phase.

The separate pre-cancelled test remains a pre-cancellation check. The Builder's direct prepare/emission boundary checks are covered independently.

No Cargo, rustfmt, JDK, product CLI, Git, or tests were run while preparing this private candidate. Root validation is still required.
