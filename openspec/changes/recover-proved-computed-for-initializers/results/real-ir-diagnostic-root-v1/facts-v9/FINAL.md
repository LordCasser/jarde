# CF07 real-IR diagnostic facts v9

The v9 read-only verifier passes against the captured root v3 stdout/stderr and execution record. Its typed use comparator checks the raw `{block_bci, bci}` coordinates exactly, permits only the known `as: phi input` annotation, verifies that annotation against the matching header-phi input, and rejects unknown keys. The phi input parser now slices the raw `inputs:` area through the last closing bracket before the `value=SsaValue` section. Slot-value lookup uses the actual raw field name (`slot=`) for `CF07_SLOT4_VALUE`; initializer markers continue to use `access_slot=`.

Facts record the corrected BCI 5 phi use in block 5. `v8-to-v9.patch` changes the verifier's facts directory to v9 so it reads the corrected facts; all earlier failed attempts remain in their own directories and are pinned.

The verifier was run with exactly:

```sh
python3 /private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3-facts-v9/verify_diagnostic.py
```

It exited 0. The run only reads the frozen record/raw and candidate artifacts. It did not invoke Git, Cargo, JDK, CLI, or modify repository files. Task 1.1 still has only SSA/CFG sub-evidence; Builder-produced initialization statement/materialization is outstanding.
