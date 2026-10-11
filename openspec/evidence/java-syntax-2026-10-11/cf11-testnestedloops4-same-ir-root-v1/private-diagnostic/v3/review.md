# CF11 same-IR diagnostic candidate v3 (private only)

This is a new, test-only patch against the exact `region.rs.base` SHA in `pins.json`; v1 remains unchanged. The optional trace is gated by `cfg(test)` and `JRE_CF11_DIAGNOSTIC`, and the test reads only the pinned adapted `testFor()I` class. It does not change branch decisions, call a parallel election path, expose a public API, or add persistent logs.

The trace now covers **every branch BCI in this one test invocation**, including nested branches at 23 and 39. Each line carries the physical block BCI. `region_at` emits every region entry depth/frame/visited state, explicit depth/reentry refusal, and its actual returned `Run` or `StopReason`, so an earlier nested loop/body refusal is distinguishable from the later branch-10 propagation. The `region_at` return trace preserves call order; a first nested failure appears before any enclosing branch records its refusal. The existing branch election logs all branch candidates and the first concrete `ArmsDoNotMeet` return sites. The test also prints the complete canonical and normal-flow graph/dominator/loop dump from v1.

SSA evidence is from the same Reader `SsaTable`, not reconstructed from operations: it prints the whole `SsaTable` with derived `Debug`, then for each instruction the existing `SsaInstruction::reads()` and `writes()` rows alongside the decoded operation. The actual accessor definitions are pinned in `pins.json` (`crates/jarde-jvm/src/ssa.rs`). These API accessors are defined on the current `jarde-jvm/src/ssa.rs` `SsaInstruction`; the complete debug output includes value definitions and uses.

Input remains `/private/tmp/jarde-cf11-testnestedloops4-baseline-root-v1/legs/javac23-original/classes/cf11/UpstreamNestedLoops4.class` (602 bytes, SHA-256 `8152aa6a52958eff721ff237f8783dca4572764ac750cc2de909b31ef6c036b5`) and paired `javap.txt` (SHA-256 recorded in `pins.json`). Exact focused test command is in `run-command.txt`. No test, Rust compiler, Cargo, JDK, JADX, or Jarde CLI was run while preparing v2.

For restoration, compare the live source with the base pin before applying, then restore it byte-for-byte from `region.rs.base` and verify the SHA after the root's guarded run.


V3 changes only the three diagnostic format placeholders for `CanonicalBlockId` from Display to Debug (`CF11_EDGE`, `CF11_NODE`, and `CF11_INSN`), retaining full block identities. It is based on the same frozen base; v1 and v2 remain unchanged. This is a compile-fix candidate only and has not been compiled or run.
