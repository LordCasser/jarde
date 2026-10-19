# 真实原件 SSA 探针

实际 `build::tests::test_break_with_label_slot3_boolean_ssa_probe` 通过 1/0，508 项过滤；只运行该临时探针，没有实现新规则。debug/no-debug 同图：whole slot3 两写 `(1,V20)/(37,V52)`，两读 `(68,V11)/(78,V11)`。退出 block53 的 V11 仅合并 V20/V52；循环 block5/15 的 Phi 输入为 V20/Itself、无用户，实际回边为 41→15/47→5。所有具体输入对应真实同槽 store，use 和 reuse owner 均核验。

实际进入既有 BooleanConsumption：load68→append69 拒绝，load78→return79 通过。因此接下来补局部类型证明和精确消费准入，保持 concat gate。物理常量生产仍由同一已 pin 原件的 Code 解码证据绑定，不据本例的 BCI/ValueId 写生产特例。

`capture.tar.gz` SHA `9804d20c75a874d29a51639a3f30771e89fd883def206a5e33faab806356f526`，16 payload+内嵌库存全部逐字/mode复开。包含原始 class 精确 pin、Luna patch、root 插入/运行/恢复 wrapper、守卫 raw 和源码前后快照。峰值 172234200B，default stack、每秒 guard、finally exact clean；共享 build.rs 已精确恢复到 SHA f99304f609bb86c9c962db6240c24a8f4fee9dd58a197eda35ef6dd001ebf9eb，target absent。

这里只证明当前原件的 SSA 和旧消费边界。新准入、全部反例以及真实准入处取消/预算仍待，不以本探针替代生产证明或完整源码运行。
