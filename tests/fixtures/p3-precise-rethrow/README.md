# Precise rethrow fixture

The sources in this directory are compiled as Java 8 with `javac --release 8 -g -Xlint:-options`.
The frozen class files are in `v8/`. `PreciseRethrowProbe.precise(int)` throws either of two
checked exceptions, logs `catch (Exception e)`, then rethrows `e`; its `Exceptions` attribute keeps
the two narrow types. `PreciseRethrowBoundaryProbe` holds the catch-all `finally`, two-row
multi-catch, and changed-value throw controls.

The two probe class SHA-256 values are:

- `PreciseRethrowProbe.class`: `1290464fe261baec374a5de3a347dada062256ef0042853e91c330da4f9fc454`
- `PreciseRethrowBoundaryProbe.class`: `2daac6f9eaa5c115d4cefb2246cc031e9b8985b88cd95d7d5f3fd75cb268603b`

The complete original/JADX/Jarde source, verification, compile and execution comparison is in
[`openspec/evidence/java-syntax-2026-09-26/precise-rethrow/`](../../../openspec/evidence/java-syntax-2026-09-26/precise-rethrow/analysis.md).
