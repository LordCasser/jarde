# Observation acceptor v3 review

This is a private correction of acceptor v2. Root's v2 run failed before acceptance with `fixed full collector argv differs from receipt config`; the preserved raw failure is `/private/tmp/jarde-iterator-assignment-observation-accept-invocation-root-v1`.

## Finding and minimal fix

The capture receipt serializes `collector_config` in argparse declaration order (`inputs_dir`, `inputs_manifest`, `out`, then the other options), while the launcher builds `collector_argv` from an explicit ordered key list (20 frozen tool/build fields, then those three paths). JSON dictionary equality correctly ignores key order, but the v2 verifier incorrectly reconstructed CLI argv by iterating the capture config dictionary. This rejected valid evidence even though all names and values matched.

v3 defines that 23-option order from the pinned root launcher source, requires the configuration key set to match exactly, and reconstructs the full argv using the launcher's order. It retains exact argv, config, executable identity, and collector/checker script pins. It does not relax any command, evidence, hash, source, runtime, or guard checks.

## Static review

- Python AST parse passes. The 23 argument names match the frozen launcher source order.
- Both fixed and controls launch collector argv exactly match the reconstructed contract. Launch/capture config values are equal for both.
- All 29 command argv vectors in each capture were reconstructed and matched exactly.
- Acceptor execution, toolchains, workspace mutation, and Git operations were not performed.

## Scope

This change only corrects argv serialization-order checking. Acceptance still requires the root to execute this verifier against the same pinned receipts after the active full-workspace run has cleaned `target`. The prior root failure remains preserved as evidence.
