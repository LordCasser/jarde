# CF12 Labels private replay v3

Status: prepared for root execution; no CLI, JDK, Cargo, Git, or repository mutation was performed. v2 is preserved. The runner passed static AST parsing and its `field_and_derived` helper was imported and exercised against the four accepted historical Bafb raw JSONs (root/Inner × default/all).

Runner: `/private/tmp/jarde-cf12-labels-full-replay-luna-v3/labels_replay.py`
Patch from v2: `/private/tmp/jarde-cf12-labels-full-replay-luna-v3/v2-to-v3.patch`
Input/artifact pins: `/private/tmp/jarde-cf12-labels-full-replay-luna-v3/pins.json`

Changes in v3 are limited to the reviewed raw-schema fixes: source-map `origin.derived` is parsed as a list of physical facts; comparison takes new projections against old own field names and physical BCIs (not the empty old projection set) and requires the normalized physical source-map anchors to remain equal; post-run source closure uses the same dynamic grouped-pin decoder as initial build validation; and the final build receipt must have completed/accepted status. Historical raw helper check passed for all four representatives, yielding 8 source-map segments each and zero prior projections. That historical check does not claim a new replay result.

The final CLI binding remains `/private/tmp/jarde-cf12-constant-candidate-cli-root-v2` SHA-256 `8e70234797af43e114e94cd08ba0a8f120d59d0e9c638a20a8cf64e9876fd64b`, 60,820,032 bytes, with final receipt `/private/tmp/jarde-cf12-constant-final-cli-build-root-v1/execution.json` SHA-256 `4105325b3cea9cae9b011c112950907861597ef848f8c0380dc93c3f2de4a872`. The pinned old Bafb CLI remains `/private/tmp/jarde-arithmetic-stack-candidate-cli-root-v2` SHA-256 `b081ca53080d0a41f00bf8f5b924499a2c781f413746fc9facb7862ef8b9cfe0`.

Root execution command (output directory must not already exist):

```sh
python3 /private/tmp/jarde-cf12-labels-full-replay-luna-v3/labels_replay.py --cli /private/tmp/jarde-cf12-constant-candidate-cli-root-v2 --cli-sha256 8e70234797af43e114e94cd08ba0a8f120d59d0e9c638a20a8cf64e9876fd64b --build-record /private/tmp/jarde-cf12-constant-final-cli-build-root-v1/execution.json --out /private/tmp/jarde-cf12-labels-full-replay-root-v3
```

The runner uses the already documented dual-JDK, whole-class, original/JADX/Jarde, `-Xverify:all`, raw/argv/tool/source/class pin, and 5 GiB / 1 GiB / one-second guard plan. No future replay outcomes are asserted here.
