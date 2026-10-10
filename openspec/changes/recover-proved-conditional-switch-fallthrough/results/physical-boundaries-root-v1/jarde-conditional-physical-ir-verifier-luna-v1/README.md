# Conditional physical IR observation verifier

`verify.py` independently checks the frozen public-reader observation in
`/private/tmp/jarde-conditional-physical-ir-root-v1`. It validates the
execution record and raw stream hashes, guarded command and test summary,
live source pins, removed temporary observer, and the four class inputs.

For each `partialBreak(II)Ljava/lang/String;` report, it recomputes the class
BLAKE3 digest and checks the physical owner, snapshot, and method identity.
It parses physical and typed instructions, branch targets, canonical blocks,
the complete eight-edge normal-edge multiset, and SSA block membership from
the raw output. Case 0 exits are derived by traversing those parsed edges.
The recorded status is `accepted-public-ir-observations-no-product`; the
observation does not establish product acceptance or a conditional proof.

Run from this machine with:

```sh
uv run --offline --with blake3 /private/tmp/jarde-conditional-physical-ir-verifier-luna-v1/verify.py
```

This verifier prints its acceptance record to stdout and writes no result file.
