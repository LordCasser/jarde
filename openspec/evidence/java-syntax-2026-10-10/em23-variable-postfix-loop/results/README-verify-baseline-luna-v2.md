# EM-23 baseline evidence verifier v2

`verify-baseline-luna-v2.py` independently checks the already collected
`baseline-root-v1` records. It never runs the baseline collector or invokes
JDK, JADX, Jarde, Git, or Cargo. Run it only when the pinned `blake3` Python
module is available:

```sh
python3 openspec/evidence/java-syntax-2026-10-10/em23-variable-postfix-loop/results/verify-baseline-luna-v2.py \
  --manifest-sha256 e8aec7964455ab84ffb12d984140e18bffe94e5cc0cf7f1eb5de06c21cc23a2d \
  --inventory-sha256 89af5a986896541c5adb0dd8bed30c89d61038017f05d57634142e8416cce783
```

A verifier result of `verified: true` means the baseline’s evidence and its
observed outcomes were independently matched. It does **not** mean that Jarde
compiled or recovered this family successfully. The actual four Jarde reports
are complete class-source renderings, but `countEmpty(Ljava/util/List;)I` is an
`explanation_only` fallback with reason `jre_region_arms_do_not_meet`; each
unmodified full-source Jarde candidate then fails javac with the recorded
missing-return diagnostic. Those compile failures and absent runtimes are
explicitly checked as the negative result, not treated as successful runs.

The verifier also checks both original JDK class files and their `javap`
instruction inventories, complete successful original/JADX class sets, exact
source copies, empty class/source paths, `-Xverify:all`, and same-JDK raw
runtime equality. It checks all source-map `primary` and `derived` origins
against the original physical method owner and BCI set, including the
explanation-only method; it does not infer successful source semantics from
that map.
