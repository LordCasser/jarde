# Root static review of typed replay v2

Root read both complete private-replay-luna-v2 scripts (257 + 276 lines) and the README while the discarded-call product's own CI was pending. No typed candidate was applied or replayed during this review. The v2 files remain immutable draft history.

The current replay cannot be accepted as written:

- Its controls compare method maps with the historical pre-pop root-v6 reports. The independently accepted post-pop/pretyped baseline must be collected before applying typed production; only exact proved statement-origin additions may explain that intermediate delta. A typed candidate cannot be its own expected map.
- The affected-method check compares `item.identity.name` and `.descriptor` directly with `item.name` and `.descriptor`. The actual frozen class-source JSON states the former as raw byte lists and the latter as objects containing `raw`, `escaped`, and `utf16`. Compare the exact raw lists and owner identity instead.
- Boundary commands are recorded with label `render`, but the final verifier counts only `render-default`/`render-all` and expects 36. Its collector records 32 CF12 renders in those labels and four boundary renders under a different label. Bind each actual command to its profile and count the explicit matrix.
- `subprocess.run(timeout=90)` does not implement the approved one-second disk/process-group guard. Use the existing guarded runner; retain original argv, elapsed time, stop and raw streams without a new execution framework.
- The collector checks that root-v6 acceptance exists and records its current digest, rather than checking the pinned accepted digest and all consumed inventory before running tools. Historical inputs and source/class/helper/SDK closure must be independently pinned before reuse.
- The verifier checks old runtime classification booleans instead of independently recomputing the historical raw oracle relationships. It must bind exact original/class/source/JDK command records and raw stdout/stderr when using those observations.
- The README permits accurate refusal of an unrelated existing boundary, but the verifier requires every whole boundary class to compile and run. Actual failures must remain recorded and root-reviewed. Do not weaken the two positive anchors or treat an unrelated refusal as runtime success.
- `javap_method_bcis` detects only method headers beginning with a limited modifier list. Package-private methods and constructor boundaries require exact method-qualified parsing. Physical BCI coverage must inspect actual primary/derived origins belonging to the same physical method, not recursively collect any JSON `bci` field.

The collector/verifier are drafts, not product evidence. Their schema and anticipated metadata contract need rechecking against the eventual actual typed build. Vestigial `if False` expressions can be removed in a new version; no historical raw or draft should be rewritten.

Existing invocation_argument code already casts primitive widening to the descriptor-required type. That architecture is sufficient to preserve an actual append(I) after presenting a local as char; a fresh complete-class overload replay still must prove this behavior. No new overload mechanism is justified by this static review.
