# For-condition next-round private pins v2 review

This v2 metadata corrects one evidence-selection error in `next-round-luna-v1/pins.json`. The v1 script used the Python list offset `commands[74]`, which refers to receipt row index 74 (successful). The failed command is receipt row index 75, exit code 101, with no guard stop. V2 copies the original receipt’s row-75 argv and exact raw stdout/stderr paths, byte lengths, and SHA-256 values. It independently checked those raw files against the hashes and lengths in `execution.json`.

The v1 pins file is preserved unchanged and explicitly marked as not usable for acceptance evidence. The original failed receipt remains unchanged: `workspace-root-v3/execution.json`, schema `for-condition-full-workspace-batched-root-v1`, status `failed`; row 76 cleanup passed, target was absent afterward, and before/after 4,718-file source pin maps were equal. This correction does not convert the failed v3 workspace result into a pass.

The running workspace runner and verifier bytes were not modified. The verifier’s exact path is `/private/tmp/jarde-for-condition-next-round-luna-v1/jarde-for-condition-ci-verifier-luna-v4.py`; its recorded SHA-256 remains `cc82f638e0292196020bee8422e62dc033c371565a7afc38c56d2b971e0df673`. Final-build-v3 remains unrun in this preparation.
