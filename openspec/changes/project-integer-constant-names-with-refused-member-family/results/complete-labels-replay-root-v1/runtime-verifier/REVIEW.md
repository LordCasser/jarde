# Review notes

The verifier is deliberately tied to v5's observed schema: top-level 32 commands with contiguous indices, `fresh_cli_renders` split into old/new groups, and four profile records keyed by the two actual JDK names and two modes. It does not recursively discover arbitrary command shapes. It verifies every captured stream before semantic checks and binds profile command references to the same indexed top-level records.

Actual report: 32 commands, 8 CLI renders, 4 profiles, 12 compile commands, 12 runtime commands, 120 runtime observation rows, and 4,695 live source pins verified. The exact per-profile ten-line output is equal across original, JADX, and Jarde.

No tools were run by this verifier. `execution.json` itself records `observed_complete_pending_independent_root_review`; this result does not replace root review or report-level verifier replay.
