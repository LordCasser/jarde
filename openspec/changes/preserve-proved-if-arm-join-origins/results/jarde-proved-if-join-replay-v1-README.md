# Private CF07 replay draft for the If-arm-join candidate

This folder contains an unexecuted collector and independent verifier for `preserve-proved-if-arm-join-origins`. Both are adapted from the root-reviewed For CF07 collector and verifier. The collector reuses the reviewed 29-command whole-class replay and physical-bytecode helpers through controlled loading; it does not copy their implementation. The verifier independently checks the resulting closed evidence bundle and does not invoke a compiler, decompiler, CLI, or build runner.

The replay uses the accepted For candidate at `openspec/changes/preserve-proved-for-latch-origins/results/cf07-candidate-root-v1` as the immediate source-map comparison. Its manifest, inventory, acceptance record, and all 119 inventory members are SHA-closed before use. The original frozen CF07 baseline remains the source/Runner and same-JDK raw-runtime oracle. Each run has the same 10 full-class legs: original 2, JADX default/none 4, and Jarde default/all 4, under JDK 8 and 23. Generated class source must equal the captured Jarde document; Runner adaptation is limited to preserving or adding the same package declaration. Compilation uses fresh empty classpath/sourcepath and output directories, then runtime stdout/stderr must match that JDK's original raw output.

The If check is intentionally narrow. It retains `andWhile@15` and `counted@30` as derived while origins; requires `counted@20` (`goto 27`) to be one derived origin on the exact entire emitted `if (local3 == 7) { ... } else { ... }` span, including its terminal newline; and requires every physical BCI to be mapped except `lastIndexOf([IIII)I@25` in each of the four Jarde profiles. It checks raw javap output and independently reparses it, so source spans cannot substitute for physical method/BCI ownership. The proof is the origin on physical BCI 20; it does not hardcode the canonical If join target. Default/all class text and every method source map must be identical, and every prior method text, presentation, and origin fact must remain present.

The new CLI path is fixed at `/private/tmp/jarde-proved-if-join-cli-v1`; metadata is fixed at `openspec/changes/preserve-proved-if-arm-join-origins/results/candidate-cli-v1.json`. The scripts intentionally contain no future CLI, metadata, source-base, or runner digest. Once root freezes those real artifacts, pass the actual SHA values and `source_commit_base` explicitly. `--build` must point inside this change to `validation-build-root-vN/execution.json`; the metadata/build schema must identify `uncommitted_if_arm_join_product`, and the matching root runner SHA must be supplied. No old CLI fallback is allowed.

Expected invocation after root has reviewed the scripts, produced the frozen CLI/metadata/build, and copied these exact wrapper bytes to the change's `results` directory:

```sh
python3 openspec/changes/preserve-proved-if-arm-join-origins/results/prepare-cf07-candidate-root-v1.py \
  --cli-sha256 ACTUAL_CLI_SHA256 \
  --metadata-sha256 ACTUAL_METADATA_SHA256 \
  --source-base ACTUAL_SOURCE_COMMIT_BASE \
  --runner-sha256 ACTUAL_IF_RUNNER_SHA256 \
  --build openspec/changes/preserve-proved-if-arm-join-origins/results/validation-build-root-vN/execution.json

python3 openspec/changes/preserve-proved-if-arm-join-origins/results/verify-cf07-candidate-root-v1.py \
  --cli-sha256 ACTUAL_CLI_SHA256 \
  --metadata-sha256 ACTUAL_METADATA_SHA256 \
  --source-base ACTUAL_SOURCE_COMMIT_BASE \
  --runner-sha256 ACTUAL_IF_RUNNER_SHA256 \
  --build openspec/changes/preserve-proved-if-arm-join-origins/results/validation-build-root-vN/execution.json
```

The collector's exclusive evidence destination is `results/cf07-candidate-root-v1`; the independent verifier writes `results/cf07-candidate-acceptance-root-v1.json` only after every check passes. This preparation did not run either script or any CLI/JDK/Cargo/Git command and makes no claim that the If candidate builds or passes replay. The accepted For/Postfix whole-class controls are not included in this CF07-only draft; a combined candidate delivery still needs those unchanged controls replayed against the final frozen If CLI.
