# Hoisted iterator controls baseline preparation v2

V1 is preserved. Its recorded preflight failed before the collector started: the launcher compared `inputs/Runner.java` (4,253 bytes, SHA-256 `4242d158…c6a5ac01`) with a manifest entry that still said 4,152 bytes, SHA-256 `809bcdf0…f5346f0a`. I updated Runner after generating the v1 manifest and failed to regenerate that manifest. The preserved evidence is `/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-controls-baseline-root-v1/{execution.json,stdout.raw,stderr.raw}`; stdout was empty and stderr names the frozen-source/manifest mismatch.

V2 creates a separate immutable source-prep tree and a separate 0444 collector input tree. The Java sources are unchanged byte-for-byte from v1; the corrected manifest is generated from those exact files. See `../jarde-hoisted-iterator-controls-source-prep-luna-v2/source-review.md` and `source-review.diff` for the exact diff and source hashes. The launch script validates every manifest entry against both trees, including byte counts, hashes, and read-only modes, before the collector starts.

The baseline launch path is also new: output is `/private/tmp/jarde-hoisted-iterator-controls-baseline-root-v2`, with a separate v2 launcher receipt. It uses the pinned collector v3/checker v7, current final CLI/build receipt, dynamic 4,727-source closure comparison, the original dual-JDK/JADX/class-source flow, 5-GiB guard and raw stream verification. It does not prefill the command count or predict compile results. Generated compile failures are preserved in the actual rows and explicitly reported in the launcher receipt status.

Only Python AST parsing and file/manifest/pin comparisons were performed. The v2 launcher has not been run; no Java/JDK, JADX, CLI, Cargo, Git, network, or collector command was used. Root can run it after confirming the workspace `target` is absent and no other workspace build is active:

```text
python3 /private/tmp/jarde-hoisted-iterator-controls-baseline-prep-luna-v2/run_controls_baseline.py
```
