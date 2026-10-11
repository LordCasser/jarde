# Hoisted iterator same-IR verifier v2

This is a new private verifier version; the delivered v1 directory is unchanged. The earlier v1 defect was in the command-line wrapper: `SystemExit(0)` was caught by `except BaseException` and turned into exit status 1. V2 catches `Exception` at the top-level while retaining cleanup-on-`BaseException` inside the archive transaction.

V2 supports the actual v1 and v3 records with explicit binding. Defaults select the v1 execution SHA/schema and candidate schema/status. For v3, supply its execution SHA, execution schema, candidate schema, and candidate status. The verifier enforces the exact candidate schema/status pair associated with the selected execution version. It reads the same final-build receipt and applies the same 4,727-file independent enumeration, live/restored source SHA, two-temporary-file, tool/runner pin, raw stream SHA, exact test/clean, default-stack, fixed-seed, and resource-guard checks for both versions.

For both runs it verifies inner BCI 42 / path `[1, 1]`, matching init/test/next `LocalVariable { slot: 5, index: 0 }`, elevated owner `[1]`, emitted owner `at_region` declaration, and the first refusal `iterator_value_and_phi_use_ownership`. It verifies outer BCI 13 still begins with `Assign` and refuses at `body_first_not_initialized_declaration`, plus the V37 three `None` uses at block 97 and header Phi V12 replacement by V37. For v3 it also binds the actual `phi_use_expansion` record: block 97 / Local(5) Phi V22 has exactly three V37 inputs, no uses, and replacement V12; block 35 / Local(5) header Phi V12 has `[V37, Itself]`, no uses, and replacement V37.

Those records establish only the observed SSA and owner facts and the recorded first refusal. They do not establish that later `next`, effect, or Builder gates passed, or that a foreach rewrite was accepted. The script has not been run and contains no prewritten acceptance result. Python AST parsing, `symtable` unresolved-global inspection, and a read-only comparison of the v3 Phi fragments against raw evidence are the only checks performed. No Git, Cargo, JDK, JADX, CLI, network, or verifier execution was performed. The verifier checks the live worktree and independently enumerates its source closure without invoking Git or making Git-blob claims.

V3 invocation values from the pinned actual record (not executed):

```text
--execution /private/tmp/jarde-hoisted-iterator-same-ir-diagnostic-root-v3/execution.json
--runner /private/tmp/jarde-hoisted-iterator-same-ir-diagnostic-root-v3.py
--out /private/tmp/jarde-hoisted-iterator-diagnostic-accepted-root-v3
--expected-execution-sha256 da1c03997a49c91ee0df2efe985da9f1d38f11376a6c2ce2ad53cb83fe6c9b87
--expected-schema hoisted-iterator-same-ir-diagnostic-root-v3
--expected-candidate-schema jarde-hoisted-iterator-same-ir-diagnostic-candidate-v3
--expected-candidate-status "private static diagnostic candidate; unapplied, uncompiled, and unrun"
```
