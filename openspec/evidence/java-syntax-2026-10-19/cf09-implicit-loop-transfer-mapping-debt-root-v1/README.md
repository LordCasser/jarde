# Separate implicit-loop-transfer provenance debt

Static architecture analysis only; no probe or implementation accepted. [Current analysis](analysis-v2.md) directly reads product region SHA5da4d9f4 and the existing latch/gateway/OriginSet/replay path. Root independently checked helper/caller source. Historical v1 is retained: its description of goto as a two-byte instruction and its stale Atlas Region coordinates are corrected in v2; the actual opcode is three bytes with a signed 16-bit offset.

Accepted primary comparison lacks only BCI69 (32/33); ComplexIf2 lacks only BCI78 (33/34). The latter instruction’s exact kind still requires direct Code inspection; matching gap shape is not proof of a common cause. Root’s next implementation must first establish the actual bounded tail Region and the helper’s refusal, then reuse existing origins if sufficient. No whole-CFG mapper, speculative attribution, or current-product code change is authorized by this note.

Current memo SHA256 `25e0708b0ebf2e0d1b7567c2f2b2d9204464d02581238df6f23b60acc84359a3`; v1→v2 diff SHA256 `4bdf8f5c40985920ac8a527e730726554a1f126a9dde49f1a6f2525041c1502e`.

The original patch diff is stored losslessly as `v1-v2.diff.gz`. Its context-only blank lines carry format-significant spaces; decompression restores the exact original bytes and the recorded uncompressed diff SHA. The frozen preparation pins remain references to those original uncompressed bytes.

## Loop5新增实际缺口（2026-10-19）

最终产品412245d51的完整TestBreakInLoop5八API均为59/60，缺BCI103；debug/no-debug物理Code均为`a7 ff a7`，即三字节goto、相对偏移-89、目标14。完整源码重编和双JDK运行/12例扩展已通过，已有映射均有效；这不等于完整来源覆盖。物理记录与独立raw审计见[Loop4/5最终对照](../../../changes/recover-named-catch-loop-return-tails/results/focused-and-full-comparison-root-v1/README.md)归档中`jarde-loop5-bci103-physical-root-v1-20261019/`。尚未建立恢复路径的来源丢失证明，不能仅凭形状断言与69/78同根因；不混入当前语法恢复改动。
