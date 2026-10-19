# Remaining fixed BreakInLoop source scope

[Current v2 plan](current-v2/README.md) is static source analysis only; no compiler/decompiler/runtime or recovery proof was run. Root reopened the source copies and reviewed the original Java 3/4/5 files. Prioritize complete original baselines for Java positive candidates 4/5; check the actual JADX result for disabled/NYI 3; treat Smali/Dex 6 as a separate input/bytecode-lowering boundary. No syntax unit is closed.

Historical v1 and its correction diff are retained. The suggestion that 4 might share Loop2’s exceptional retry shape is rejected: 4 has no try/catch/handler. This was an erroneous root task hint, not evidence; source facts override the hint. Original check cases remain intact.

The original patch diff is stored losslessly as `v1-v2.diff.gz` under `current-v2`. Its context-only blank lines carry format-significant spaces; decompression restores the exact original bytes and the recorded uncompressed diff SHA. The frozen preparation pins remain references to those original uncompressed bytes.
