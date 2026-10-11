# Current-product own-CI acceptance adapter v3

v3 fixes the source mapping for the two cross-case/backedge tests: both belong to `tests/p3_switch_forward_join.rs`, not `crates/jarde-java/tests/p3_conditional_switch_fallthrough.rs`. During preparation, every `NEW_REQUIRED` source/name pair was checked against the current source text; all 25 names were found in their mapped files. The reused v5 verifier's prior `REQUIRED_TESTS` list was also inspected and contains no stale `null_first_exact_builder_writes_refine_only_the_local_type` requirement. The renamed NullThenBuilder test remains required by the current-product mapping.

All v2 acceptance conditions remain: explicit `--workspace`, rejection of failed root-v3 evidence, pinned original 95-batch runner, complete passed no-carry 359-target run, full product Git/live pins, current CLI build source pins, exact two-seed 4-job/52-step CI, complete per-target named-outcome comparison, and unique same-status `***` binding with raw logs untouched.

Invocation and capture procedure are unchanged from v2 FINAL; pass the actual new complete workspace directory with `--workspace`. No commit, run ID, future result, aggregate total, or future workspace path is frozen here.

Preparation read the current source/test map, verified every required function name in its declared file, parsed the verifier AST, and reread all input/artifact pins. No Git, Cargo, JDK, JADX, or CLI command ran; no repository or raw evidence was modified.
