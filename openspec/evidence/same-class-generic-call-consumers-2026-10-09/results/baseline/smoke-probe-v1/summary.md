# Baseline replay summary

Frozen inputs: 2 selected input cases across 2 source families and 1 JDK legs. Original full-class javac succeeded for 2/2, JADX for 2/2, and accepted Jarde baseline for 2/2. Counts are whole-class compilation only; probe exits, behavior assertions, reflection identity and all diagnostics remain separately recorded in `manifest.json`.

Jarde CLI SHA-256: `3f75dab495c9753ec2c3f1bd0ac91665ed42f8b877eea65cb7c69a6879448f70`. Probe SHA-256: `15311367877b47f1c09c950aaaa572a0ec64ad5bc497c987d842f19a3b4b807e`. All build legs use the input-specific release/debug flags, explicit empty classpath/sourcepath, fresh classes directories, and runtime `-Xverify:all` with only those classes. Existing CallHold/ExceptionHold/BoundOverload jars are byte-for-byte reused and indexed by their source paths and hashes.

This is baseline evidence, not a candidate acceptance result. No class is omitted after a compile failure. The per-case source, jar, JADX tree, Jarde source, Probe.java, javac/probe stdout/stderr and hashes are retained.
