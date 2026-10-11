# Compound negative-control observation verifier v4

This private verifier preserves the v3 checks and fixes two frozen-record schema details exposed by actual validation. The 74 input pins contain 74 distinct literal paths but 68 resolved paths: exactly six JDK executable pairs have `/Library/...` and `/Users/lordcasser/Library/...` aliases. v4 allows only those six hard-coded path pairs, still verifies every pin, requires equal bytes/SHA-256/mode in each pair, and requires the post-run pin list to equal the complete original list. Other duplicate resolved paths fail.

The fixture and Runner each have separate original and copied pins with different expected paths. v4 verifies both pins and requires their byte count, SHA-256, mode, and actual bytes to agree; it also requires each copy pin in the 74-pin inventory.

The verifier was only syntax-checked with Python AST parsing. A separate read-only inspection checked the frozen JSON's literal/resolved path counts, six alias groups, and copied-input metadata/bytes; the verifier itself has not been executed. No workspace, Git, Cargo, JDK, or product CLI was used.
