# Compound condition clone/path fixture

This private fixture starts from the frozen 2,848-byte `CompoundStatementConditions.class` at `/private/tmp/jarde-compound-condition-old-candidate-runtime-root-v2/original-bytecode/CompoundStatementConditions.class` (SHA-256 `2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa`). The patcher rewrites only `test(ZZZZLjava/lang/StringBuilder;)Ljava/lang/String;`'s `Code` attribute and lowers the class major version from 52 to 49. That method's nested Code-attribute count is zero, so the old `StackMapTable` is discarded. The candidate is a 2,658-byte class (SHA-256 `ededf0b0ac57ca6259a31d83e5a30309d175cb8cdc2bfd2c839566df94407376`).

The 33-byte body is:

- `0: jsr 8`, returning to BCI 3; `3: jsr 8`, returning to BCI 6.
- `6: aconst_null; 7: areturn`, so the caller returns null after both calls.
- `8: astore 5` saves the return address. The subroutine compares argument slots 0–3 using real `ifeq` branches at BCIs 11, 15, 22 and 26. Its structure is `(a && b) || (c && d)`. Both outcomes end in `ret 5` at BCIs 29 and 31.

The reader should normalize two clones at the same subroutine BCIs with paths `[0]` and `[3]`. For the clone/path boundary check, use the actual comparison block `CanonicalBlockId { bci: 11, path: [0] }`, its decoded `ifeq` target at BCI 21, and the actual caller continuation `CanonicalBlockId { bci: 3, path: [] }` as the join. The existing `compound_statement_condition` should refuse at its root/join path check and leave `visited` unchanged. The caller continuation is the first `ret`'s actual Return-edge destination and also the second `jsr` call site.

The patcher was syntax-parsed and called in memory to confirm it deterministically reproduces the checked-in private candidate. No JVM verifier, reader, Cargo, or project CLI was run; root should verify those. The constant pool has no tags newer than the legacy classfile set used here, and the rewritten Code has `max_stack=1`, `max_locals=6`, no exception entries and no nested Code attributes.

SHA-256:

- patcher: `58ac104070b2b423eb8b19999fb3bc238402ab62afffbbb68959c1872c5d7b7b`
- original class copy: `2fc584d5ba9d35940516fcf9001479a72851ebb0216c22a62f3ec5500dbc35aa`
- candidate class: `ededf0b0ac57ca6259a31d83e5a30309d175cb8cdc2bfd2c839566df94407376`
- fixture metadata: `fixture.json` records code bytes, method, version and expected BCIs.
