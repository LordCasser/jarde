# Current-product own-CI acceptance adapter v5

v5 is a one-point fix to v4: the expected per-target summaries now use tuples, matching `local_command_observation()` which returns `counts` as a tuple. The root v4 preflight failed only at this list/tuple comparison for `jarde-jvm`; the raw error is preserved at `/private/tmp/jarde-compound-own-ci-local-git-preflight-root-v1/error.raw`, with its execution record beside it. No expected numeric count changed and no acceptance outcome is claimed.

All v4 behavior remains unchanged. Re-run root's local/Git preflight and then the full verifier after reviewing this exact diff. No Git, Cargo, JDK, JADX, or CLI command was run while preparing v5.
