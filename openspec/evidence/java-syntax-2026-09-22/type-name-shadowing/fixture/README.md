# Type qualifier binding fixture evidence

`original.txt` is the complete null/non-null runner output for the one permanent
`TypeQualifierProbe.class`.  `original-javap.txt` and `runner-javap.txt` preserve the Java 8
class shape and source-only runner.  `run_audit.py` compiles all helper and runner sources,
checks the source-produced target bytes against `v8/TypeQualifierProbe.class`, and runs the
original, unchanged JADX, and current jarde complete-class paths with `java -Xverify:all`.

The positive class uses external default-package owners `arg0` and `arg0_2` in static calls and
field reads/writes, an unused `ShadowOther` parameter, and a same-class no-conflict static-call
control.  The runner resets both external static fields for each case and records null/non-null
outputs plus the instance field, so a changed qualifier is visible even when compilation succeeds.

`summary.json` and `cases.json` record hash, method/Code counts, all outputs, each failure stage,
and the `target/debug/jarde-cli` SHA-256 before and after the audit. Debug `java` package-prefix
and method-reference qualifier/real-field shadowing remain source-only boundary evidence under
the change notes and are not added to the permanent positive class. `run_boundaries.py` records
the source-only method-reference, debug-parameter, and real-field package-prefix probes: original
and JADX compile/run successfully, while the current Jarde output fails the expected javac checks
at those source-binding boundaries.
