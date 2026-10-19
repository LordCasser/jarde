# Next same-run ownership probe preparation

Prepared only; zero probe/compiler/target runs. [Accepted original baseline](../cf09-testbreakinloop2-original-root-v1/README.md) proves an ownership refusal at BCI0, not its internal cause. Archive contains Luna v1/v2, root static architecture review and a 27-pin root preflight. Root rejected the v1 formatting/env-wrapper/unrelated-analysis issues; v2 is the current candidate. The source and same-run interfaces were reviewed, but compilation remains pending.

Next: reopen product/source/input/tool pins, review the private v2 patch and bare Cargo test command, then apply temporary cfg(test) instrumentation under the existing 5GiB/target1GiB/default-stack/per-second guard. Set the probe flag only in the guarded-runner subprocess environment, record it, and finally restore region bytes/removal of the temporary module/target absence plus external input pins exactly. Actual recover traversal and the prefinal Region tree decide the cause; auxiliary calls or source hypotheses are insufficient. Absolute private include paths must be restored or adapted in a new reviewed/pinned preparation on another host. No OpenSpec repair or new mechanism has been accepted.

The archive inventory binds every payload’s bytes/mode; root reopened them all. Historical errors are retained separately from v2.
