# 立项前提证伪与关闭（root，2026-10-06）

**本 change 关闭，零生产改动。** 实现片门控实验（`evidence/gating-experiment.md`，双腿决定性探针）证伪了 proposal 的两个前提：

1. **"计数门把已提交局部与栈携带值混同"——假**。`prepare_deferred_bindings` 的计数门自有前置（`instruction.writes()` 须含写该值的 `Slot::Stack(_)`），已提交局部的 store 写 `Slot::Local(_)`，**根本不在门的输入集**。
2. **"NI 形=已提交局部多读被拒"——假**。NI 的拒绝来自**拼接链内联分支**：`(… == n)` 的引用等值以控制流物化 0/1，`+` 链因此跨块，`concat@1` 的 `jre_concat_split`（toString 终端在另一块）拒链 → deferred-binding 回退 → "3 consumers" 是**级联症状**。判别探针：NMA（去分支的四读）与 NI2（NI 仅去比较）都完整恢复（`local1` 渲染四次、行为与源一致）；NMB（NMA+一分支）逐字复现 NI 诊断。

## census 修正（重要，覆盖第 4 族主行归因）

- 全部 **114 处** "has N consumers" 存档出现**均为 "has 3 consumers"** = 1 真指令读 + 2 平凡 phi 操作数记录（**phi 膨胀**），语料中**不存在**真多指令消费者形；
- 第 4 诊断族（多消费者）的主行是**级联面**：真因分布在其上游（本例=拼接跨块；BI 例=增强 for 协议隐式多读）。后续巡查见到该诊断应先做 phi 记录剥离再归因；
- 多消费者族的**可恢复性真缺口**重定位为：**内联条件值作 `+` 链操作数**（链跨 join）——已另立 `recover-inline-conditional-concat-operands`（判别探针 NI/NI2/NMA/NMB/CMP 已冻结于本目录 `evidence/`）。

## 处置

tasks 1.1 完成（即证伪记录）；2.x/3.x 永久关闭（前提不存在）。负例资产（真多指令消费者合成探针的需求）转移至新片的门控实验要求。
