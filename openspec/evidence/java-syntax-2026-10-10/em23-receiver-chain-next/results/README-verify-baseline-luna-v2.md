# EM23 receiver-chain baseline verifier v2

`verify-baseline-luna-v2.py` independently checks the closed `baseline-root-v2` evidence and preserves the four recorded Jarde compile failures as a product gap. It does not invoke Java, JADX, Jarde, Git, or the collector. The verifier refuses to overwrite `baseline-independent-acceptance-luna-v2.json`.

The v2 changes are limited to corrections for the actual frozen evidence schema: the complete Runner SHA-256; normalizing the two JDK legs from the pinned tool-control manifest's list while cross-checking both control and toolchain manifests against the baseline's `jdk_legs` map; recognizing qualified constructor names and both javap flag formats; validating the distinct field and method identity shapes; and checking `class_set_exact` for original/JADX cases while requiring `complete_source_class_set` only on original cases. The four Jarde failures, no-runtime boundary, inventory, source-map, whole-source and raw-output checks are unchanged.

Preparation includes read-only checks of the actual manifest, raw javap and class-source JSON schema, plus Python bytecode compilation only. No acceptance execution has occurred.

The pinned BLAKE3 dependency invocation is:

```sh
uv run --no-project --with blake3==1.0.11 python -B \
  openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/results/verify-baseline-luna-v2.py
```
