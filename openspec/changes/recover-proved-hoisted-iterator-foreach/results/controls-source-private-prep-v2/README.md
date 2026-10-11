# Hoisted iterator controls source preparation v2

This version repairs the manifest/source mismatch found before collection in baseline-prep v1. The Java files are byte-identical to source-prep v1. `Runner.java` is 4,253 bytes with SHA-256 `4242d158…c6a5ac01`; the v1 manifest incorrectly recorded 4,152 bytes and SHA-256 `809bcdf0…f5346f0a`. The v2 manifest is regenerated from the actual source bytes and records 4,253 bytes / `4242d158…c6a5ac01`.

No Java source change was made in v2. The positive and refusal classes remain byte-identical to v1 as well. The diff review is in `source-review.md` and `source-review.diff`; it shows only corrected manifest metadata. This copy is immutable (0444) and is the source of the matching frozen inputs in baseline-prep v2.

The original v1 preflight failure is preserved at `/private/tmp/jarde-cf11-invocations-root-v1/hoisted-iterator-controls-baseline-root-v1/` (empty stdout; stderr reports that `inputs/Runner.java` differs from the manifest). No collector or compiler command started in that attempt. No Java/JDK, JADX, CLI, Cargo, Git, or network command was run while preparing v2.
