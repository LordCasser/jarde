# EM23 receiver-chain baseline verifier v3

`verify-baseline-luna-v3.py` independently checks the closed `baseline-root-v2` evidence. It preserves the four recorded Jarde compile failures as a product gap, and refuses to overwrite `baseline-independent-acceptance-luna-v3.json`.

V3 narrows javap member parsing to lines inside the actual top-level class body, from its opening `{` through the matching top-level `}`. This prevents constant-pool entries and trailing attributes such as `InnerClasses` from being mistaken for fields or methods. The parser retains the qualified-constructor and JDK 8/JDK 23 flag handling from v2.

Preparation checked the exact parser against all four archived original javap reports: outer class 1 field / 3 methods and nested class 1 field / 1 constructor for each JDK, with exact descriptors, access flags, and instruction BCI sets. It also checked the actual case-census and class-source identity shapes and ran Python bytecode compilation only. The full acceptance verifier was not run.

```sh
uv run --no-project --with blake3==1.0.11 python -B \\
  openspec/evidence/java-syntax-2026-10-10/em23-receiver-chain-next/results/verify-baseline-luna-v3.py
```
