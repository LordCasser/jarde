# Final CLI v2 NullThenBuilder replay adapter

This private adapter collects 19 guarded commands: default/all class-source, full-class JADX, then compile and runtime for original/default/all/JADX under javac8 and javac23 (eight runtime legs). The original Java source is unavailable; the original profile uses the canonical class fixture directly. Acceptance requires the sole source change to fold the nested `if` into `if (arg0 && local2 == null)`, unchanged whole-class membership/maps between default and all, and `true=7` / `false=6` across all eight runtimes.

The collector calls the v9 `run_command` guard (one-second polling, at least 5 GiB free, target at most 1 GiB). The independent verifier rechecks every argv, raw stream pin, exit status, resource measurement, the frozen v5 build/source pins, and the CLI SHA provided on the command line. It does not run a toolchain; run it only after the root has produced the final candidate CLI v2 build evidence.

Expected root invocation after the final build record exists:

```sh
CLI_SHA256="$(python3 -c 'import json; print(json.load(open("/private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json"))["cli"]["sha256"])')"
python3 /private/tmp/jarde-null-builder-final-cli-v2-replay-luna-v1/collect.py \
  --cli /private/tmp/jarde-compound-condition-candidate-cli-root-v2 \
  --cli-sha256 "$CLI_SHA256" \
  --out /private/tmp/jarde-null-builder-final-cli-v2-replay
python3 /private/tmp/jarde-null-builder-final-cli-v2-replay-luna-v1/verify.py \
  --execution /private/tmp/jarde-null-builder-final-cli-v2-replay/execution.json \
  --candidate-build /private/tmp/jarde-compound-condition-focused-and-cli-root-v2/execution.json \
  --cli-sha256 "$CLI_SHA256" \
  --out /private/tmp/jarde-null-builder-final-cli-v2-replay/verification.json
```

The CLI hash is intentionally supplied from the actual build evidence at execution time; no future CLI hash or replay result is asserted here. No Git, Cargo, JDK, JADX, or CLI command was run while preparing these files.
