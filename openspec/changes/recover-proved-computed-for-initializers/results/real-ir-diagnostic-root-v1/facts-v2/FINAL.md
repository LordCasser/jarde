# CF07 real-IR diagnostic facts v2

This is a corrected private extraction of root v3's captured diagnostic. `ValueId(9)` is defined by the BCI 0 `iload_3` instruction; `ValueId(3)` is the entry value for local 3. `facts.json` distinguishes those values and stores definitions/uses in structured form.

`verify_diagnostic.py` now checks each listed init read/write, SSA value definition/use, local-4 access, header phi input/use, loop ID/BCI, return-leaf fact, and canonical edge against captured `0.stderr.raw`, in addition to recorded command argv/exit, raw byte hashes, temporary source-pin delta/restoration, marker counts, and cleanup evidence. It is read-only and does not invoke tools.

The verifier was syntax-parsed only and was not executed. Root can run exactly:

```sh
python3 /private/tmp/jarde-computed-for-real-ir-diagnostic-root-v3-facts-v2/verify_diagnostic.py
```

The task 1.1 evidence remains SSA/CFG-only: Builder-produced initialization statement and statement materialization/order have not been diagnosed.
