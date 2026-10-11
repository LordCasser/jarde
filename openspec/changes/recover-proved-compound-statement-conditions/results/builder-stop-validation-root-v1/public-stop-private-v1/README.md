# Compound whole-recovery stop tests (private candidate)

Base test file SHA-256: `d7445477041184c58a002b09fcf81b38e1c76df8906d619e1e871f1b62bf49de`. The candidate only changes the existing test helper and adds two tests; it does not edit product code or public APIs.

`Run` now captures `analysis_steps` immediately after the recovery call from the exact budget passed to that call. The Compound test first records a successful `recover` usage, then reruns the same frozen whole-class `recover` request with every other limit unchanged and `analysis_steps = success_usage - 1`. It requires a real `AnalysisSteps` Stop with zero written bytes, that exact limit, and an `at` BCI that is an instruction in the frozen physical method; the test prints the observed limit and BCI for root to pin. It also requires `RecoveryContent::NotProduced`, no artifact binding, and empty text/source map.

A separate token-pre-cancelled Compound request checks atomic absence from the start. This pre-cancelled case is deliberately not described as a late stop. The budgeted test measures the public `recover` call over the already prepared real IR in the existing fixture helper; its BCI identifies the observed stop site only and does not claim which private prepare/emit stage stopped. Private Builder stage coverage remains the responsibility of Builder unit tests that call those stages directly.

No Cargo, Rust formatter, JDK, Git, or test execution was performed. This is an unexecuted private source candidate for root integration and validation.
