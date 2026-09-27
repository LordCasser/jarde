# Test14 conditional finally replay

Run `replay.sh JARDE_CLI OUTPUT_DIR` with a fresh CLI. It first reruns the frozen original/JADX fixture script, then generates all three minimal class sources with Jarde and recompiles them with `javac --release 8 -g`. The runner uses the generated top-level binary `$` type spelling; its seven-path behavior is compared byte-for-byte with the original and pinned JADX under `java -Xverify:all`.

`Test14-minimal.class` is the frozen same-layout target (`356108017a0d24be26105ea227e93e3089a8832411ba938d3e3ece4193ed35e0`). `Test14Neighbor.java` generates three additional verifier-valid near misses from it:

- `Test14-slot0.class` (`afc2d590ca4511ce7154116b43a696b1c18a9a7cbe777f4cd5d20ce932311953`) adds `aload_0; astore_0` inside the protected range. All seven original outputs remain, but cleanup local-0 reads now have an instruction definition rather than method-entry `this`.
- `Test14-second-field.class` (`ed2e277d8ae162163d71045fbf28bd4b5f29de55fc4d4a2970cb1a85203793df`) changes only the second normal cleanup read (BCI 22) to a new field with the same descriptor.
- `Test14-handler-call.class` (`96094337cfea465b5e85d0249539e6e05342471075b4c0b7372ee79f5a2eee89`) changes only the handler cleanup call (BCI 43) to the existing same-signature `doFinallyAlt()`.

The replay checks JVM verification and execution for each, and Jarde refuses all three. The six frozen verifier-valid neighbors also remain refused.
