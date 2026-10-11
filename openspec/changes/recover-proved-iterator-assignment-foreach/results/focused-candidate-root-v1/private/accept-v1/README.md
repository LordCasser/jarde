# Focused iterator-assignment regression acceptor prep

`accept.py` independently checks the ten-command `iterator-assignment-regressions-root-v1` receipt. It has not been executed. It is limited to focused regression evidence and can only return `accepted_focused_only`; this does not establish full-workspace or runtime acceptance.

The verifier checks the pinned runner before/after/live SHA, receipt hash and schema/status, exact argv/index/cwd, environment and default Rust stack, all twenty raw stream hashes, per-command exit/guard/free-space/target measurements, and test summaries plus named raw outcomes. It independently rebuilds the expanded local closure as the pinned source closure plus top-level `examples/*.rs`; four recorded pin stages must have the same key set, formatter changes must be within the exact three-file allowlist, regeneration must change only the corpus fingerprint, and all final pins must match live bytes. It checks the single canonical `CF11OuterAssignmentTarget.class` and its frozen SHA, final CLI bytes/hash/mode and difference from the old CLI, target absence after final clean, and all three named public API tests.

The API stop record is parsed from the captured raw stderr without a predeclared BCI, usage, limit, or written count. Its actual BCI is cross-checked against the decoded instruction starts of the exact frozen `UpstreamNestedLoops.test(List,List)` method using the separately pinned pure-Python classfile decoder. The API test source is also pinned by the final source closure and checked for the test contract. No Git, Cargo, JDK, Jarde CLI, or shared application command is executed by this preparation.

Root invocation shape after independent review:

```text
python3 /private/tmp/jarde-iterator-assignment-focused-accept-prep-luna-v1/accept.py \
  --receipt /private/tmp/jarde-iterator-assignment-regressions-root-v1/execution.json \
  --receipt-sha256 <actual-execution-json-sha256> \
  --out <new-private-acceptance-json>
```
