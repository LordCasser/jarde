# Focused local-reference-write acceptance run plan v2

Prepared only; this acceptor has not been run. It validates the completed focused-v3 receipt and current read-only source closure.

```sh
python3 /private/tmp/jarde-local-reference-write-focused-accept-prep-luna-v2/accept-focus.py \
  --receipt /private/tmp/jarde-local-reference-write-focused-root-v3/execution.json \
  --receipt-sha256 f322798aff08a9e510410b1167da847f3072b53f9f7a575e8ab033ab0c47fd30 \
  --candidate-sha256 56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d \
  --out /private/tmp/jarde-local-reference-write-focused-accept-prep-luna-v2/acceptance-v1.json
```

Frozen pins:

- Acceptor SHA-256: `718bf1444575ed2183d802fd73773223825dca052be2f2febb141b49a67a1b92`
- Receipt SHA-256: `f322798aff08a9e510410b1167da847f3072b53f9f7a575e8ab033ab0c47fd30`
- Runner SHA-256: `41d52db53da0462f3e2dc65c410f62ed20ed0aa57c28e1fe4b9b9da498adb327`
- Candidate `build.rs` SHA-256: `56664a850c8b8f47d27c66072cfb3552a492a6901e1d29a528e19f809780ce5d`

The source-pin check uses the receipt's exact `source_pins_before_format`, `source_pins_after_format`, and `source_pins_after` fields. It recomputes the first-to-second diff and requires it to equal `formatter_changed_paths`; the observed diff is empty, and the post-format and final maps are identical. Each free-space observation is independently checked against its relevant floor. In particular, the cleanup command sample (`14349611008`) and receipt final sample (`14349213696`) are retained as separate observations and are not required to be byte-equal.

The acceptance scope remains `accepted_focused_api_only`. Static test-source counts (4 positive fixture iterations, 36 control iterations, 28 pre-cancel iterations) are not additional named test outcomes. This does not establish late-stop behavior, full-workspace acceptance, compiler/runtime behavior, or all task-2.2 requirements.
