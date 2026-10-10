# Discarded-call product CI capture and verifier (private draft)

This directory contains an unexecuted, reviewable capture script and independent verifier for `preserve-proved-discarded-call-origins`. No Git, Cargo, JDK, GitHub CLI, or product CLI command was run while preparing these files. The product CI has not been captured or accepted by this draft.

The capture accepts an explicit product commit and run ID. It calls the three established `gh run view` forms, saves the API stdout/stderr and the stable/supply job logs, uses deterministic gzip (`mtime=0`), and records command argv, environment override hashes, UTC start, duration, exit status, byte counts, and SHA-256 values. It refuses to overwrite its evidence directory. It first pins the accepted return-arm CI artifact (`d04ac86e…fea47`) and requires its exact accepted identity, 4 jobs/52 steps, and both 3393/0/97 workspace runs with 354 records. The new capture's expected counts are fixed at lib 337, gateways 15, and 3398 workspace passes; its expected workspace summary count is 355 because the new integration binary adds one summary record to the accepted predecessor's 354.

After capture, run the verifier with explicit, independently obtained values:

```text
python3 /private/tmp/jarde-discarded-call-ci-luna-v1/verify-ci-product-luna-v1.py \
  --product-commit <40-hex-CI-head> --run-id <decimal-run-id> \
  --source-base <40-hex-validation-source-base> \
  --metadata-sha256 <candidate-cli-metadata-sha256> \
  --cli-sha256 <frozen-cli-sha256> --build-sha256 <validation-execution-sha256> \
  --runner-sha256 <results-runner-sha256>
```

The verifier requires the product commit and live working tree to match every pin: 17 product files, 8 test/workflow files, and 47 canonical inputs. It checks the frozen CLI, metadata, build execution, validation runner, fixed v9 guard template, all 12 actual argv rows, all 24 raw build streams, disk-guard records, and per-command summaries. It loads only the SHA-pinned static CI and integer CI helpers and calls their verification functions; it does not run their `main` functions.

For each fixed CI seed, the verifier directly reads the stable job log and requires the `jarde_java` library target to report 337 tests, the gateway binary to report all 15 accepted test names, and `proved_discarded_call_origins` to report exactly the five source-pinned test names, each `ok`, with a 5/0/0 summary. It independently recomputes 355 workspace records totaling 3398/0/97 and invokes the pinned integer-test verifier for the prior eleven regressions. The final acceptance file is created exclusively and only after all checks pass.

The capture assumes GitHub job names remain `stable / test and specification` and `supply chain`, matching the accepted capture script format. The new test names are copied from the current integration test source and are checked against the committed product source by the verifier; if root changes names before the product commit, update this private draft before use.
