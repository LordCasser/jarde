# Guarded integer-array build v2

`run-validation-build-root-v2.py` is a prepared replacement for the v1 guarded build/freeze runner. It retains the 20 GiB free-space and 1 GiB `target` limits, two Cargo jobs, disabled incremental compilation/debug info, the existing validation commands and the same 30 source/canonical pins. After Clippy, it adds three focused tests: the two report-level array-name projection cases, the single class-source body replay budget-stop case, and the eight facade integer-name tests. Their filters and expected summaries are recorded in the script; the facade filter matches the existing `integer_constant_name_tests` module and the scoped-v6 command evidence.

The runner writes only to `validation-build-root-v2`, freezes the resulting CLI at `/private/tmp/jarde-int-array-names-cli-v2`, and writes `candidate-cli-v2.json`. It is prepared for root review and has not been run.
