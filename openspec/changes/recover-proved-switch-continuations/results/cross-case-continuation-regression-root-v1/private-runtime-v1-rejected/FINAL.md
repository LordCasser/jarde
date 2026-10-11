# CandidateContinuation controlled full-class replay (prepared only)

No repository files were changed and no toolchain, CLI, Git, JDK, or JADX command was run while preparing this handoff.

The collector accepts explicit --fixture, --fixture-sha256, and --class-name arguments. Defaults are pinned to the reviewed 255-byte two-member CandidateContinuation fixture. The oracle defaults to the complete 726-byte Join class. Runtime compares that full Join.class against CandidateContinuation class-source default, class-source all, and full-class JADX output under both frozen JDK legs (javac 8 and javac 23 --release 8). Each runner constructs its class exactly once and invokes mixedExits over MIN,-3,-1,0,1,2,3,4,MAX. Expected output is pinned in pins.json: 1 returns 30, 2 returns 20, every other input returns 30.

The separate complete Join report is pinned as a preserved boundary: it has all seven methods and retains fallback outcomes for deepCatch and pick. The independent candidate control does not substitute for claiming the larger Join class succeeded. The candidate generation record binds the one goto mutation, unchanged 42-byte method Code, and the exact 13 target BCIs. The verifier checks those BCIs against the physical javap record, then requires all 13 to appear in the mixedExits source map under the exact candidate class/method owner. It also checks method and constant-pool UTF8 reference closure. Candidate default/all must each recover exactly the two class members, with complete structured method reports.

The run has exactly 19 guarded commands: two whole-class APIs, one full-class JADX decompile, then eight javac/runtime pairs. Both frozen Java runtime legs use -Xverify:all. The accepted guard/build-evidence helpers are pinned in pins.json; the runner keeps the existing one-second poll, 5 GiB free-space floor, and 1 GiB target-size ceiling. CLI SHA-256 and candidate build-evidence path are explicit invocation inputs so the run binds the actual final v2 CLI and its build.

After root supplies the final CLI artifact and build record, run:

    python3 /private/tmp/jarde-candidate-continuation-replay-luna-v1/collect.py \
      --fixture /Users/lordcasser/workspace/projects/jarde/openspec/changes/recover-proved-switch-continuations/results/cross-case-continuation-regression-root-v1/two-member-control/cross-case/CandidateContinuation.class \
      --fixture-sha256 9aa523e64295cdf5738d1240ea2ba4b30b136cfec4a5b90cd362a61bfdb6c238 \
      --class-name CandidateContinuation \
      --cli <actual-v2-cli> --cli-sha256 <actual-cli-sha256> \
      --candidate-build <actual-build-evidence-json> \
      --out /private/tmp/jarde-candidate-continuation-replay-root-v1

Then run the independent verifier against that execution record, using the same build-evidence path and CLI SHA:

    python3 /private/tmp/jarde-candidate-continuation-replay-luna-v1/verify.py \
      --execution /private/tmp/jarde-candidate-continuation-replay-root-v1/execution.json \
      --candidate-build <actual-build-evidence-json> \
      --cli-sha256 <actual-cli-sha256> \
      --out /private/tmp/jarde-candidate-continuation-replay-root-v1/verification.json

No replay result is claimed. The verifier is a prepared independent acceptance gate; root should review its source and run it with the actual final CLI/build evidence.
