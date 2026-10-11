# Private CF11 same-IR observation acceptance prep v1

This verifier is prepared for root to run against the frozen root-v2 invocation. It has not been executed. It does not build or invoke Cargo, Java, Git, or the observation runner. It writes only the requested external acceptance/diagnostic manifest when invoked.

The verifier pins the actual root-v2 run receipt, invocation receipt, v2 runner, patch copy, closure helper, baseline fixture, javap listing, and production guard/scanner. It checks the exact single named Cargo test, full raw stdout/stderr hashes, complete `begin → lvt/local/accesses → phi_summary/phi → end` trace and API-result line, exact 4-method control table in the candidate test source, and the four method signatures in the frozen class listing. The four controls are verified as source/class definitions; this one-test invocation did not execute that separate API-control test.

It checks 4,744 source pins before, 4,745 during (only the two candidate source files plus the baseline-v1 fixture differ/add), and an exact 4,744 restoration afterward; it also checks both files' base/candidate pins and the real cleanup-only `cargo clean` exit/target-absent evidence. The LVT row is recorded as control evidence only; the verifier makes no Java-type inference from it.

The actual run receipt is currently `failed`: the named test exited zero, but the resource guard recorded `observed_after_exit=true` at 5,352,644,608 free bytes, below the required 5 GiB (5,368,709,120). Cleanup later exited zero and restored the source closure, but that does not convert this run to accepted `observed_same_ir`. The verifier therefore emits a diagnostic manifest with status `failed`, preserving the observed trace facts and guard failure. A future invocation could be accepted only if every guard passes and the runner itself reports `observed_same_ir`.

Verifier SHA-256: `d1de661706853a92790200bb10877c024645145fc743ca1300eb664f9198b772`.
