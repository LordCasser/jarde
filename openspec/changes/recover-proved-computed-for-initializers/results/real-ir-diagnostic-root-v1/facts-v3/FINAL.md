# CF07 real-IR diagnostic verifier v3

This v3 preserves v2's facts and analysis and changes only `verify_diagnostic.py`. Root's v2 run failed at `value12 uses bound to raw`: raw typed coordinates were `{block_bci: 5, bci: null}`, while facts also carried the annotation `as: phi input`.

The new comparator first requires and compares the typed coordinates exactly. It allows only the known `as` key, requires its value to be exactly `phi input` with a null instruction BCI and the header phi block, and separately confirms that the matching `ValueId` is an input to that phi. Unknown annotation keys and annotation values fail. The same rule covers `ValueId(21)`; the value 12 and 21 phi annotations are both explicitly related to `header_phi.inputs`.

The v2 failure record is pinned as an input. The verifier was syntax-parsed only and was not executed. Root can run exactly:

```sh
python3 /private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3-facts-v3/verify_diagnostic.py
```
