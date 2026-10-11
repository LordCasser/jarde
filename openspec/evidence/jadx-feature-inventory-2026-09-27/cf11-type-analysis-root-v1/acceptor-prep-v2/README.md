# Private CF11 fixed-run diagnostic audit prep v2

This private v2 copies reviewed prep v1 and fixes only the `FIXTURE_SHA` name error in the final diagnostic-facts manifest field, using the already pinned `FIXTURE_SHA256` constant. V1 remains unchanged.

This verifier is frozen to the current root-v2 failed run and invocation receipts in `pins.json`. It is a diagnostic audit of that fixed failed run only; it is not a general acceptor and cannot accept a future rerun. Its inputs require the exact runner invocation to exit 1 with the recorded `AssertionError`, the run receipt to remain `failed`, and the guard stop to remain below the 5 GiB minimum. The named test itself exited 0, but the resource guard stopped after command exit. Cleanup passed and restored source pins; neither condition converts the run to `observed_same_ir`.

When root chooses to execute it, the verifier should write the complete raw diagnostic facts alongside `status: failed` and return exit 1. The output does not claim a successful same-IR observation or product fix. No verifier, Git command, or toolchain was run during this v2 repair; only static AST parsing was performed.

Verifier SHA-256: `8c14fc622f8d9fe984fe0a998c8536bb2b61e8b1ae5bd80b41668e012e72daae`.
