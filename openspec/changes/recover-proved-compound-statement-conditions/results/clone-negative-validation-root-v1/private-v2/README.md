# Compound clone/path fixture v2

This is a new candidate directory; failed v1 is retained unchanged. It starts from the controlled 2,848-byte `CompoundStatementConditions.class` (SHA-256 `2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa`). The patcher changes only `test(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;`'s Code attribute and the major version 52→49.

The body has `jsr 8` at BCI 0 and BCI 3; they return to BCI 3 and 6, after which the caller returns null. The shared subroutine begins at BCI 8 with `astore 5`, then uses real `ifeq` comparisons of boolean argument slots 0–3 at BCIs 11, 15, 22 and 26. The graph expresses `(a && b) || (c && d)`. The two outcome blocks at 29 and 32 each `goto` one shared `ret 5` at BCI 35. This leaves exactly one `ret` instruction for each `jsr` call context, addressing the v1 verifier rejection. The target Code attribute has no nested attributes; `max_stack=1`, `max_locals=6`, `code_length=37`, no exception-table rows.

The Rust test should discover, from reader-produced canonical/SSA blocks, the comparison instruction at BCI 11 inside the clone for call site 0 (likely canonical block ID BCI 8, path `[0]`), and the real caller continuation block BCI 3 with empty path. Invoke the existing compound proof with the decoded comparison operation/target and this actual join. The expected early refusal is the root/join path mismatch, with `visited` and other walker state unchanged. The exact root canonical block BCI must be asserted from reader output; do not fabricate it as BCI 11.

This candidate has only been checked by classfile patcher structure and deterministic in-memory rebuild. It has not been JDK-verified or run through the Reader; root will perform those checks before fixing the Rust test.

SHA-256: source `2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa`; candidate `74d632baa95485b7233476984a56598b9689cfa1552f965b5a1dd87704ad1525`; patcher `34ad1c8e87983b62bdef29f3b73ad0ab6c4d320e35cef402d80025b10a677bd0`. `fixture.json` records offsets and code bytes.
