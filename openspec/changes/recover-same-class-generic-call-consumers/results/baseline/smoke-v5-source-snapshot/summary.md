# Frozen baseline replay summary

Frozen inputs: 3 selected input cases across 3 source families and 1 JDK/debug modes. Original full-class javac succeeded for 3/3, JADX for 3/3, and CLI flavor `baseline` for 1/3. Counts are whole-class compilation only; probe exits, behavior assertions, reflection identity and all diagnostics remain separately recorded in `manifest.json`.

Jarde CLI label: `baseline`; SHA-256: `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`. Probe SHA-256: `15311367877b47f1c09c950aaaa572a0ec64ad5bc497c987d842f19a3b4b807e`. Executed runner snapshot: `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-same-class-generic-call-consumers/results/baseline/smoke-v5-source-snapshot/replay-executed.py`; SHA-256: `ae41bed5c37c3f3bd3e69e4865e67776447ea601990066ec620f5536c94b51c7`. All build legs use the input-specific release/debug flags, explicit empty classpath/sourcepath, fresh classes directories, and runtime `-Xverify:all` with only those classes. Existing CallHold/ExceptionHold/BoundOverload jars are byte-for-byte reused and indexed by their source paths and hashes.

This is a frozen-input replay. No class is omitted after a compile failure. The per-case source, jar, JADX tree, Jarde source, Probe.java, javac/probe stdout/stderr and hashes are retained.
