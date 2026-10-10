# Array literal boundary fixtures

This is a focused, standalone adaptation of three JADX `TestCls` fixtures:

- `TestArrayFill4` → `LongArrayLimits`, retaining the private static final `ARRAY_SIZE = 4` field and the exact long-array return method.
- `TestArrayFillConstReplace` → `ConstantIntArray`, retaining `CONST_INT = 0xffff` and the exact array return method.
- `TestArrayFillNegative` → `DependentArrayStores`, retaining the three dependent element stores and removing only the JUnit `check` harness.

The complete upstream test files are frozen under `upstream/`. The isolated sources preserve each selected field/method body token-for-token after leading indentation is removed. The standalone runner checks exact values, array identity across repeated calls, and prints deterministic output. The runner's assertions are a runtime sanity check; the baseline script still uses the fresh original complete-class run as the oracle and compares every exit/stdout/stderr triple byte-for-byte.

`prepare-baseline-luna-v1.py` prepares original, JADX default, JADX none, and Jarde full-class source sets on javac 8 and 23. All three product sources and the runner compile together with empty classpath/sourcepath, and each runtime uses only its fresh classes with `-Xverify:all`. The JADX input jar is built from exactly the three target classes emitted by the javac 23 original leg. Every CLI render, command, raw stream, class file, javap record, source, and closed inventory is retained. Failures remain evidence and are not repaired in the harness.

This records three narrow array-literal cases only. It does not claim broader array recovery coverage or completion of an EM18 suite.

Root actually executed `prepare-baseline-luna-v2.py` (v1 was prepared but unused). The v2 raw execution is preserved in `root-execution-v2`: exit 0, 39 commands, 8 full class-set legs; original 2/2, fresh JADX 4/4, Jarde 2/2 compile and raw-runtime success. It fixes the repository root, accepts an unrenamed default package, verifies the exact 10 product pins, and captures all 12 default/all CLI responses with identical class text. Independent verification of the raw evidence is pending.

The semantic result does not remove the presentation gap: JADX uses `CONST_INT` and `Long.MAX_VALUE` in the recovered arrays; current Jarde emits equivalent complete numeric literals. These inlined class-file values do not prove which source symbol was originally written. Symbol selection and ambiguity require separate architecture analysis before proposing implementation.

Root then independently reviewed and actually executed `results/verify-baseline-luna-v2.py`, exit 0; see `results/root-verifier-execution-v2` and `results/baseline-verification-luna-v2.json`. V1 was prepared but unused; v2 corrects upstream snapshot paths and packaged class census, then closes the actual preparer execution, all source copies, member identities, physical BCI maps and raw outcomes. All eight legs are accepted for this narrow baseline. No instance-initializer implementation is claimed.
