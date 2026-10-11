## 1. Preconditions and real-path observation

- [ ] 1.1 Wait until `recover-proved-conditional-switch-fallthrough` has its own exact CI accepted and final clean delivery recorded; keep this change unapplied before that point.
- [ ] 1.2 Re-read the accepted `cf12-upstream-remaining-root-v1` README, root acceptance, frozen source/class pins, and the two-remaining-structure plan. Freeze a new read-only candidate baseline and keep the older Jarde failures as historical failures.
- [ ] 1.3 Instrument only private diagnostic runs to identify the actual first rejection in both methods. Confirm or disprove FT2 `next=175` versus outer boundary 197 and TestSwitch2 candidate/overlap hypotheses before choosing a production patch.

## 2. Local proof changes and focused tests

- [ ] 2.1 Prepare a private bounded-tail continuation candidate for FallThroughCase2, reusing the existing switch-arm/continuation checks and retaining enclosing-frame ownership.
- [ ] 2.2 Prepare a private finite-DAG shared-join candidate for TestSwitch2, limited to a unique decoded candidate and fully proved Join/Return/Throw outcomes.
- [ ] 2.3 Have the root review both candidates and merge them serially into the same `region.rs`; do not create branches or worktrees, and keep the final production patch coherent.
- [ ] 2.4 Add real-class focused coverage proving the FT2 internal join remains case-external, its straight outer tail is emitted once, and the outer condition and switch-break origins remain exact.
- [ ] 2.5 Add real-class focused coverage proving TestSwitch2's shared continuation is emitted once and each early return remains in its original path with exact source origins.
- [ ] 2.6 Add only the negative ownership boundaries required by these proofs: ambiguous candidate, external predecessor, shared non-join block, invalid case-entry crossing, cycle/nested switch/unknown terminal, and non-Normal canonical edge.
- [ ] 2.7 Exercise actual budget and cancellation Stops through the new scans; verify accurate dispatch location, usage, and atomic absence of partial source/map. Run existing straight-fallthrough, grouped-label, switch-break, loop, and control-flow regressions.

## 3. Root-owned complete validation and delivery

- [ ] 3.1 Run formatting and the exact applicable test/lint checks under the root's resource guard; preserve every failure raw and do not inflate budgets to pass.
- [ ] 3.2 Freeze a new CLI plus source, class, helper, SDK, JDK, and invocation pins; compare full original/JADX/Jarde-default/Jarde-all classes across all ten CF12 fixtures, preserving upstream checks, complete generated-class sets, runtime stdout/stderr, and physical source origins. Record actual profile outcomes without assuming success.
- [ ] 3.3 Independently verify the candidate replay, strict OpenSpec validation, product CI and clean delivery; update verification/ledger/handoff only from accepted evidence and do not claim all CF12 or the 71-unit ledger complete from this structural milestone.
