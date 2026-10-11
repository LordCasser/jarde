# CandidateContinuation controlled replay v2

This private harness compares the complete pinned 255-byte `CandidateContinuation.class` as the canonical original profile against API default, API all, and JADX whole-class source. It does not use the seven-member Join class as a runtime oracle. The Join report is pinned only as a read-only boundary record: its `deepCatch` and `pick` methods remain fallback.

The fixture SHA-256 is `9aa523e64295cdf5738d1240ea2ba4b30b136cfec4a5b90cd362a61bfdb6c238`; the independently observed frozen BLAKE3 is `06d3b07c87615ef02a521748823e792acd7df229773a2cccb90cb517e61f76e2`. Root-provided generation evidence binds the 42-byte target code and two-member class. The verifier checks the complete 13-BCI target map and UTF-8 byte offsets for source-map spans; no separate constant-pool parser is present.

The collector has a 19-command plan: two API calls, one whole-class JADX call, and eight compile/runtime legs (JDK 8 and JDK 23; original/default/all/JADX). Each Runner constructs the class once and calls `mixedExits` with nine values. The runtime uses `-Xverify:all`. The collector loads the existing resource guard with its one-second poll, 5 GiB minimum-free, and 1 GiB target-size limits. The candidate CLI path and SHA are supplied at collection time and checked through the existing build-evidence helper.

No collector or verifier run has been claimed here. No repository, Git, Cargo, JDK, JADX, or CLI operation was performed for v2. Source inspection confirmed there is no new constant-pool parser and the prior literal escaped-newline statement corruption is absent. Python AST parsing was not performed because this task forbade CLI/toolchain execution; root can perform that check during acceptance.

Invocation shape (root supplies actual CLI/build paths and CLI SHA):
```
python3 collect.py --fixture <CandidateContinuation.class> --fixture-sha256 9aa523e64295cdf5738d1240ea2ba4b30b136cfec4a5b90cd362a61bfdb6c238 --class-name CandidateContinuation --cli <actual-v2-cli> --cli-sha256 <actual-cli-sha256> --candidate-build <actual-build-dir> --out <new-output-dir>
python3 verify.py --execution <output-dir>/execution.json --candidate-build <actual-build-dir> --cli-sha256 <actual-cli-sha256> --out <new-accepted.json>
```

The exact source and evidence pins are in `pins.json`.
