# Lambda descriptor adaptation fixture evidence

`original.txt` is the complete `java -Xverify:all` output from the frozen
`LambdaAdaptationProbe.class`; `original-javap.txt` and `runner-javap.txt` preserve every `Code`
and `BootstrapMethods` entry.  `run_audit.py` compiles all source-only inputs with
`javac --release 8 -g:none`, checks the source bytes against the permanent class, and runs the
unchanged original, JADX, and current jarde complete-class paths.  No generated source is edited.

The runner resets each call's state and records nulls, wrong-type failures, evaluation order, and
the dynamic implementation call count.  `genericRawInteger` returns the actual Integer through a
raw Supplier; `genericTypedInteger` reports only the caller-side String cast failure.  The
bound-null, boxing, unsupported return-narrowing, and unknown-reference cases remain outside the
positive class and are represented by the planning evidence instead.

`summary.json` and `cases.json` record class hash/size, method and Code counts, Bootstrap count,
CLI hash at both audit boundaries, complete outputs, and per-case failure stage.  Only
`v8/LambdaAdaptationProbe.class` is permanent; helper and runner classes remain source-only.
