# `verify-baseline` v1 → v2

The v1 script remains unchanged. V2 corrects the following frozen-evidence schema mismatches while retaining the other inventory, command/raw, source-map/BCI, member-family, whole-source, oracle, and failure-boundary checks.

1. Corrected `RUNNER_SHA256` from the truncated `...1359` to the actual pinned `...1359b`.
2. Reworked `verify_tool_pins`: it now reads `jdk_manifest.legs` as a list keyed by each row's `leg`, compares the expected leg set, and cross-checks every tool path and SHA-256 in three places: baseline manifest `jdk_legs`, control manifest `jdk_tools`, and source toolchain manifest `tools`. It returns the already-verified baseline `home/tools` map normalized as `legs[leg]`, which the javap/compile/runtime checks consume. It does not infer or relax any pin.
3. Constructor parsing now strips the package prefix before comparing a declaration with the simple class name, recognizing `em23.InputFieldIncrement2()` and `em23.InputFieldIncrement2$A()` as `<init>`. Flag parsing extracts `ACC_*` tokens, handling both JDK 8 plain flags and JDK 23 `(0x....)` prefixes; empty `flags:` remains access flags zero.
4. Method identity validation now matches the actual shape `{owner, name, descriptor}`. Field identity validation remains `{owner, member}` with its field member payload checked exactly.
5. Whole-source cases now require `class_set_exact` for original and JADX cases. Only original cases require `complete_source_class_set`; JADX is checked according to its actual schema and is not required to carry that original-only field.
6. The result filename/schema are versioned to v2 so a later run cannot overwrite a v1 result.

Hashes:

- v1 verifier: `aa6223b360a1743d2dfb023bdec7876c8f5b6dd2410134d9fff197a35aede2ca`
- v2 verifier: `32438382c1a2e06ad68eb4f907ed9a9265dfb981738c1115295b7ad5449d3b2c`
- v2 README: `98b7df91e76193ece507b16c0d0cbd5ed951b7ceb453bb2ab6e17f450f8715a0`

Preparation checks: read-only assertions over the actual JDK/toolchain leg and tool schemas, javap constructor/flag text, class-source identity shapes, and case census fields; `py_compile` only. The final verifier was not run.
