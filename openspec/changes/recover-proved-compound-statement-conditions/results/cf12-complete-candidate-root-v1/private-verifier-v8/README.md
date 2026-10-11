# CF12 compound verifier v8 (private candidate)

This verifier is based on root v7 and is intentionally strict for the new compound candidate. It accepts the full-class replay only when the replay CLI matches the separately pinned compound CLI build record, the build record verifies its exact 19 live source pins and five successful raw commands (17 region tests and 6 public tests), and its post-clean absence claim is recorded. The old `--class-digests` argument remains the historical digest-oracle execution with its original CLI, rustc/oracle argv, raw stdout, and exact eleven class inputs; that CLI is verified as oracle provenance and is not compared to the new replay CLI.

Acceptance additionally requires all 20 JDK/default/all comparisons to have matching method identity, source text, and source map, both Switch2 and FT2 to compile and run identically on JDK 8 and JDK 23, and source-map BCI sets to equal the complete frozen javap instruction set: 78 instructions for Switch2 and 90 for FT2 in all four JDK × default/all profiles. Existing environment classification, including major-version-55 failures, remains derived from the raw command streams.

Invocation for root-controlled real results:

```sh
python3 /private/tmp/jarde-cf12-compound-verifier-luna-v8/independent-verifier-luna-v8.py \
  --execution /private/tmp/<new-compound-ten-fixture-replay>/execution.json \
  --class-digests /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/candidate-full-class-root-v1/cli-build-passed/execution.json \
  --candidate-build /private/tmp/jarde-compound-condition-focused-and-cli-root-v1/execution.json
```

No replay or verifier execution was performed while preparing this private candidate. Static Python AST parsing succeeded. Root must run it against the actual newly completed replay and inspect any rejection without weakening these gates.
