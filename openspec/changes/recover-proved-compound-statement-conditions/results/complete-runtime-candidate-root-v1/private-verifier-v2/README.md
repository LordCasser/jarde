# Complete-class runtime verifier v2

`verify.py` is an independent, read-only verifier for the frozen full-class `execution.json`. It does not launch JDK, JADX, Jarde, or the candidate CLI. It otherwise retains v1's check scope and command/runtime/source-map evidence checks.

v2 fixes two verifier defects found when root checked v1 against the actual run:

- The fixture manifest file SHA-256 is `2104a1ef6f5b84381379dede5aa55030508f45dd4aad31fda0cfca8d3ca34775`. The separate README SHA-256 is `2fccd2bd82ed8fd0bc43e8050c9d9bcde92eaada4626349528121aff30b0fc15`; v2 checks that value in the manifest's `readme_sha256` field and independently hashes the fixture `README.md` bytes.
- JDK tool paths in the plan and execution record may use different symlink spellings. v2 compares strict resolved paths while retaining each pin's bytes and SHA-256, checks the execution record's mode against the live file, and verifies both declared and recorded pins against the live target. JDK pins are not accepted merely because their paths were normalized.

The original v1 verification report had 26 of 28 checks consistent. Its two inconsistent checks were the incorrect manifest digest comparison and literal JDK path comparison. v2 was prepared against the frozen execution record and that report; it has not been run. Root should run it with the fresh CLI SHA and a new output path, then review all checks before relying on its report.

Frozen input locations remain the same: the plan at `/Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/replay-planning-root-v1/plan-luna-v1.json`, the fixture directory `/private/tmp/jarde-compound-condition-runtime-fixtures-luna-v2`, and candidate CLI `/private/tmp/jarde-compound-condition-candidate-cli-root-v1`. Expected CLI SHA is supplied to the verifier at invocation time. v2 does not add checks outside the prior verifier's scope.
