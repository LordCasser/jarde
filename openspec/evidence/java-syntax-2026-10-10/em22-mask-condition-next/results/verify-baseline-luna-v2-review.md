# EM-22 independent verifier v2 review

This is a new verifier; v1 and the collected baseline are unchanged. The v2 delta fixes the statically identified review issues: it reuses the `method3_observation` captured while matching the exact physical method identity, removes an obsolete javap text-format assertion, checks each physical method JSON `access_flags == 1`, and binds the report method indices to the original javap declaration order (`<init>()V` at 0, `method3(II)I` at 1). The parser also explicitly preserves that javap member order.

The normalized `ACC_PUBLIC` flags check in the javap parser remains in place, including JDK 23's `(0x0001) ACC_PUBLIC` formatting. All other v1 manifest, inventory, raw-stream, tool-pin, compile/runtime, physical owner, source-map, and semantic checks are retained.

Only Python syntax compilation was performed for v2. The verifier itself and Java/JADX/Jarde tools were not run.
