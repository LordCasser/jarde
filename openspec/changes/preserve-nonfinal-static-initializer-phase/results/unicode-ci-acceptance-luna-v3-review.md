# Unicode CI replay verifier v3 delta

V3 leaves v1 and v2 unchanged. In v2, the metadata loop required each current working-tree file to equal the frozen metadata hash. That is no longer appropriate after the instance-initializer worktree changes.

V3 instead reads each of the 10 production, 4 test, and 16 canonical blobs from the pinned commit `6fd51a18dd83d980f2f060246c209fb6fb0afba1` with read-only `git show --no-textconv`, and requires each blob hash to match the frozen metadata. It records each current file hash and equality as informational output; drift in the current worktree does not fail this pin check. The Unicode test source and fixture hashes remain explicit checks. All collector, CLI, jar, JDK, report, source-map, compilation, and runtime checks are otherwise inherited from v2.

The script was syntax-checked only. It was not executed, and makes no claim about local Rust tests or CI beyond the already captured root evidence.
