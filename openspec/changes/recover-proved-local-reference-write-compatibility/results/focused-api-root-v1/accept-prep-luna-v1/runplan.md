# Focused local-reference-write acceptance run plan

This is an independent, read-only acceptance of the already completed focused-v3 capture. It does not run Cargo or change the repository. Run only after reviewing the frozen pins below.

```sh
python3 /private/tmp/jarde-local-reference-write-focused-accept-prep-luna-v1/accept-focus.py \
  --receipt /private/tmp/jarde-local-reference-write-focused-root-v3/execution.json \
  --receipt-sha256 f322798aff08a9e510410b1167da847f3072b53f9f7a575e8ab033ab0c47fd30 \
  --candidate-sha256 56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d \
  --out /private/tmp/jarde-local-reference-write-focused-accept-prep-luna-v1/acceptance-v1.json
```

Frozen inputs:

- Acceptor SHA-256: `2f320721043653b5cf30ba18ef47b4803e1c802835669aee0ddce4594867b469`
- Receipt SHA-256: `f322798aff08a9e510410b1167da847f3072b53f9f7a575e8ab033ab0c47fd30`
- Runner SHA-256: `41d52db53da0462f3e2dc65c410f62ed20ed0aa57c28e1fe4b9b9da498adb327`
- Candidate `build.rs` SHA-256: `56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d`

The acceptor validates the pinned helper digests, receipt schema/status, all three command argv/index/cwd/exit/environment/raw-stream hashes, default stack, 5 GiB free-space and 1 GiB target guards on fmt/test, the explicitly recorded fixed `cargo clean` exception, the 4,784-path source closure before/after formatting/after cleanup against current live files, candidate builder pin, exact 28-class fixture matrix, stdout's three unique named tests and 3/0/0 summary, and target absence. It also confirms the test source contains four positive fixture iterations, 36 control iterations, and 28 pre-cancel fixture iterations; these are static loop-derived cases, not additional named test results.

Acceptance status is deliberately limited to `accepted_focused_api_only`. It does not establish late-stop behavior, full-workspace acceptance, compiler/runtime behavior, or all task-2.2 requirements.
