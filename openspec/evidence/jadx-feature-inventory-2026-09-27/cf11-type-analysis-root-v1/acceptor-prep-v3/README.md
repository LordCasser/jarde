# Private CF11 fixed-run diagnostic audit prep v3

V3 preserves v1 and v2 and fixes the event grouping bug found in the executed v2 audit. It parses each diagnostic event token once with a token-boundary regex and groups those exact parsed `(event, line)` pairs. This retains the final `event=end` line, which has no trailing space, and prevents substring collisions such as `local`/`lvt`. The expected event sequence remains exact; no event requirement or resource guard was relaxed. The default output is isolated in this v3 directory.

The prior root v2 acceptance is retained as historical failure evidence at `/private/tmp/jarde-cf11-local-type-debt-observation-acceptance-root-v2.json`, SHA-256 `ed4bedde6906100545ff7a41d6b6975daaebc1bb5daf1f68c88aa479b35cc52c`. It has status `failed`, failures `real resource guard stopped/failed the test command` and `IndexError: list index out of range`, and `facts: {}`. The `IndexError` occurred because the v2 dictionary comprehension created an empty list for the parsed `end` kind by searching for `event=end `, then indexed `[0]`. The root v2 artifact is unchanged and is not replaced or reinterpreted by this prep.

V3 is still a diagnostic audit of the fixed failed run only. The pinned invocation exited 1 with the runner's `AssertionError`; the runner receipt is `failed` because the resource guard stopped after command exit at 5,352,644,608 free bytes, below 5 GiB. V3 is expected to write a `failed` manifest with the full observed raw facts and return exit 1. It cannot claim `observed_same_ir` or a product fix.

No verifier, Git command, or toolchain was run. Static AST parse passed. V3 acceptor SHA-256: `86ba71a701b2c66c74a73c8565acef920b44ec1578af3008ee69b9dd0750461d`.
