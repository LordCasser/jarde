# EM-22 integer mask condition fixture

This is a minimal complete-class adaptation of the active `TestRedundantBrackets.TestCls.method3` JavaInput shape: first the additive guard `a + b < 10`, then the integer bitwise condition `(a & b) != 0`, with returns from each arm and a trailing return. The method remains an instance method with its two `int` parameters.

The common Runner takes five paths through one instance: `(3,4)` selects the first guard; `(14,6)` passes the first guard and has a nonzero mask; `(8,7)` passes the first guard with a zero mask; `(-6,12)` exercises a negative operand and first guard; `(Integer.MAX_VALUE,1)` exercises Java `int` overflow in the first guard. Expected stdout is manually derived as `3\n84\n7\n-6\n2147483647\n`; this is a hypothesis for the original compiled class to confirm, not executed evidence.

Prepared inputs are `MaskCondition.java` SHA-256 `507a40ffd0e2f67e7c085e38599a7afd57bad53175ad90c075374a70bd39dc20` and `Runner.java` SHA-256 `1d69b66ac5b23a85dd2d7d0ecf1a66fe43b6e73dec8ef02b2b67927f117f28fd`. The collector is `../results/prepare-baseline-luna-v1.py`. Root supplies the corrected frozen candidate CLI and exact metadata file/hash explicitly, for example via its required `--cli`, `--cli-sha256`, `--metadata`, and `--metadata-sha256` options.

The prepared matrix is 2 original complete class runs, 4 JADX (default/none × two JDKs), and 4 Jarde runs (default/all × two JDKs). It uses the fixed Corretto 8 and OpenJDK 23 manifest, the pinned JADX 1.5.6 launcher, isolated empty classpath/sourcepath, fresh class directories, Java 8 source/target compilation, original javap instruction facts, complete source-map BCI checks, and `-Xverify:all` raw-output comparisons. Only a Runner package prefix may be adapted for JADX's `defpackage` output; generated target sources are compiled byte-for-byte as archived.

No toolchain was run. This fixture is not a Jarde bug claim and does not pull cast, `instanceof`, or array statements from the same JADX test into this slice.
