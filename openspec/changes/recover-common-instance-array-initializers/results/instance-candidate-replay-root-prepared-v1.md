# Instance-array candidate complete-class replay collector

Preparation only. The runner is [prepare-instance-candidate-replay-root-v1.py](./prepare-instance-candidate-replay-root-v1.py); it has not been executed. It does not rebuild original inputs, run JADX, inspect Git, or alter product files and accepted baseline evidence.

The runner requires explicit `--cli`, `--cli-sha256`, `--metadata`, and `--metadata-sha256` arguments. It checks the CLI and metadata pins, the exact 10 product / 4 test / 16 canonical metadata path sets, and each declared source hash. For the currently frozen inputs, the invocation is:

```sh
python3 -B openspec/changes/recover-common-instance-array-initializers/results/prepare-instance-candidate-replay-root-v1.py \
  --cli /private/tmp/jarde-instance-array-cli-v1 \
  --cli-sha256 5abb4bc82a50a8d427eff425ad80bc2fb845253b4216a81b60229e2337711663 \
  --metadata openspec/changes/recover-common-instance-array-initializers/results/candidate-cli-v1.json \
  --metadata-sha256 b86961182a352dfa663215c47bdd3e64cf7f8a67c5e36951f2c98d020f044cfa
```

It reuses the accepted controls baseline (10 target class files and two original runners per JDK) and the no-clinit/super-argument baseline (two target class files and one original runner per JDK). It checks the controls acceptance-v5 record and requires the no-clinit independent verification-v2 result (`success`, 1053 passed, zero failed); the original no-clinit preparation status remains `baseline-with-failures` because its JDK 8 `javap` header parser was too strict.

For every target class file on both JDK baselines it records fresh default and `--evidence all` class-source JSON/stdout/stderr. It compares assembled default/all text, exact field and method identities, physical method report text, source maps, and owner class bytes' BLAKE3 digest against the accepted baseline facts. The emitted physical-facts records include observed source-map BCIs only; they make no claim that all bytecode offsets have source anchors. The assembled class text is checked for the intended positive field initializers in `CommonDirectSuperByteArray`, `FinalLiteralTwoArrays`, and `CommonNoClinitArrayInit`.

It then compiles each complete generated Java source set with the original runner source copied from its hash-checked archived input. A package prefix is the sole permitted runner adaptation when the generated class source declares a package. Each set uses empty classpath/sourcepath directories, Java source/target 8, and fresh output classes. Runtime uses `-Xverify:all`; each runner's exit/stdout/stderr is compared byte-for-byte to its same-JDK original raw run. Compile failures are retained and do not suppress later JDK or family legs. A complete run records 48 render commands, four compiles, six runner executions, and 52 case records.

The output directory is `results/instance-candidate-replay-root-v1/`. Its manifest records every command and raw stream, all candidate JSON/source/class artifacts, input snapshots and physical-facts records; the file inventory closes over the output, including the manifest and excluding itself. Failure outcomes are preserved as `baseline-with-failures` rather than represented as successful replay.
