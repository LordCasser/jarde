# B/C/S field-store core

This is the input-stage B/C/S subset of the sibling 380-case field-store audit. `NarrowFieldStores.java` and `NarrowFieldStoreEffects.java` remain the original javac inputs; `patch_field_stores.py` names only the six B/C/S field descriptors, leaving the two int-named boolean controls unpatched. `NarrowFieldStoresRunner.java` runs the B, C, S cases only. No decompiled output was edited to make a test pass.

使用 `python3 openspec/evidence/java-syntax-2026-09-22/numeric-conversions/narrow-field-stores/core-bcs/run_audit.py` 重放。原 class 为 3443 B，SHA-256 为 `ae0bfc789051a30dd6779d57cea2f484d7e5fe4822f18102eb74915075d04eda`；patched class 为 3467 B，SHA-256 为 `da7bbcebd61dda7221f5169200a68b11fbe9200b8a4bb1f9d970bccc559157a5`。49 个 Code 属性在补丁前后逐字节一致。当前 CLI 运行前后的 SHA-256 均为 `48edb9d2e3eec451983aabb4affcbcaf6d723c8a75b0284605f729743088cc76`。

patched class 通过 `java -Xverify:all` 并产生 285 行（B/C/S 各 95 项）。Jarde 完整源码同样
可编译并运行 285 行，字段写入处没有引用。JADX 完整源码仍无法通过 `javac`，因此这里不
建立 JADX 运行对照。完整 380 项仍是 Z 低位边界，其 root 独立重放位于 `../root-review/`。
