# Final CLI v2 replay verifier adaptation

The accepted 28-command verifier is reusable for its full runtime matrix, whole-class default/all text and source-map checks, command order, exact 65-row behavior, and original-versus-JADX difference checks. Its necessary adaptations are the fixed CLI path (`candidate-cli-root-v2`) and a check against the actual final build record; the 28-command envelope and all existing comparisons remain intact. The existing CF12 v8 verifier already checks the 179-command replay envelope, ten full-class fixtures, method text and source maps, physical javap BCI coverage, call order/provenance, and historical JADX outcome differences. Its only build-gate adaptation replaces the old 19/17/6 fixed counts with exact dynamic product-path recomputation and raw-stream parsing against the build record's observed summaries. The CF12 gate now also explicitly requires all 179 referenced commands.

`build_evidence.py` validates the v5 schema, fixed CLI v2 path and live CLI pin, actual `build.json` commands/raw streams and post-clean state, dynamically recomputed product set from the pinned v5 runner, and current source hashes. It does not assert a product-path count or hardcode compound/public test counts. The CLI digest is read from the actual build record and passed to collector/verifier argv at replay time.

No build, replay, JDK, or CLI was run while preparing this adapter. Python AST parsing was used only for the private verifier files.

Root invocation sequence (run only after the final build is complete):

```sh
python3 /private/tmp/jarde-compound-condition-focused-and-cli-runner-root-v5.py /private/tmp/jarde-compound-condition-focused-and-cli-root-v2
CLI_SHA256="$(python3 -c 'import json; print(json.load(open("/private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json"))["cli"]["sha256"])')"
python3 /private/tmp/jarde-compound-condition-runtime-compare-root-v3.py --cli /private/tmp/jarde-compound-condition-candidate-cli-root-v2 --cli-sha256 "$CLI_SHA256" --out /private/tmp/jarde-compound-final-runtime-cli-v2
python3 /private/tmp/jarde-compound-final-replay-verifier-luna-v1/runtime-verifier-v3.py --execution /private/tmp/jarde-compound-final-runtime-cli-v2/execution.json --candidate-build /private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json --cli-sha256 "$CLI_SHA256" --out /private/tmp/jarde-compound-final-runtime-cli-v2/verification.json
python3 /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/candidate-full-class-root-v1/jarde-cf12-ten-fixture-replay-runner-luna-v2.py --cli /private/tmp/jarde-compound-condition-candidate-cli-root-v2 --cli-sha256 "$CLI_SHA256" --out /private/tmp/jarde-compound-cf12-final-cli-v2
python3 /private/tmp/jarde-compound-final-replay-verifier-luna-v1/cf12-verifier-v9.py --execution /private/tmp/jarde-compound-cf12-final-cli-v2/execution.json --class-digests /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/candidate-full-class-root-v1/cli-build-passed/execution.json --candidate-build /private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json
```

The files in this directory are `runtime-verifier-v3.py`, `cf12-verifier-v9.py`, `build_evidence.py`, their relative-source patches, `pins.json`, and this note. `pins.json` deliberately contains no future CLI SHA.
