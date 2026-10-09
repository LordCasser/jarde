# Frozen candidate replay summary

Frozen inputs: 140 selected input cases across 35 source families and 4 JDK/debug modes. Original full-class javac succeeded for 140/140, JADX for 130/140, and CLI flavor `candidate` for 140/140. Counts are whole-class compilation only; probe exits, behavior assertions, reflection identity and all diagnostics remain separately recorded in `manifest.json`.

Jarde CLI label: `candidate`; SHA-256: `5b58a8816479c63fbcae8c0d9dcb0c7c7be59e80203551a0c72977585353a1b7`. Probe SHA-256: `15311367877b47f1c09c950aaaa572a0ec64ad5bc497c987d842f19a3b4b807e`. Executed runner snapshot: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/candidate/candidate-v3/gc01-08-140/replay-executed.py`; SHA-256: `6b102b7c16e0bc318d42d3fbd21a32d51bd3bbeeeb72c8f66722808c09f28960`. All build legs use the input-specific release/debug flags, explicit empty classpath/sourcepath, fresh classes directories, and runtime `-Xverify:all` with only those classes. Existing CallHold/ExceptionHold/BoundOverload jars are byte-for-byte reused and indexed by their source paths and hashes.

This is a frozen-input replay. No class is omitted after a compile failure. The per-case source, jar, JADX tree, Jarde source, Probe.java, javac/probe stdout/stderr and hashes are retained.
