# EM-23 receiver-chain baseline collector v2 review

This is a prepared, unexecuted collector. The immutable v1 script and `baseline-root-v1` record are untouched; v2 writes only to `baseline-root-v2`.

The v2 delta is limited to three evidence-handling corrections:

- Runtime equivalence compares only process `exit`, raw `stdout`, and raw `stderr`. Recorder command labels and argv remain stored, but do not make identical executions compare unequal. The original case still checks the independently derived expected output `add=8\nmultiply=20\n` and empty stderr.
- Jarde records the complete serialized `member_family` object and its observed family/projection states. `PreparedStatic` and `PreparedFold` store children under `members[*].child`, so v2 no longer assumes a top-level `child`. The collector makes no source-map verification claim.
- For each JADX source profile and each Jarde rendered root, the collector records whether `+=` and `*=` occur and a source-text SHA-256. Those observations are not acceptance gates; full generated source files and Jarde method records remain captured.

The fixture, fixed tool pins, compile matrix, input class family, runner, and raw command capture are inherited unchanged from v1. Only Python syntax compilation is permitted before root review; no JDK, JADX, Jarde CLI, or collector execution has occurred.
