# 内部类跨类私有访问 + long 位掩码巡查（2026-10-05 root）

## ⚠️ 结论作废声明（同日 root 复核后更正）

初版曾登记"`access$002` 跨类形 = EM-15 残留位点 not recovered"——**证据无效**：判定所用 `/tmp/jarde-cli-spared` 构建于 11:03，早于 EM-15 合入 af6f65c4（11:51），**为 EM-15 前旧二进制**。真 javac 8 腿（corretto）同探针同拒复测确认是二进制陈旧而非新缺口；[账本 compound-accessor 行](../../jadx-feature-inventory-2026-09-27/summary.md)记载的"合入后真 8 腿 access$000+access$002 双恢复"为权威。**跨类 access$002 形维持已恢复结论，无新缺口。**

## root 自纠错（本轮第 3 次提取级教训）

**patrol 基础设施纪律新增**：spared 二进制在每次主线代码合入后即过时——巡查前必须比对 spared 构建时间与最近代码合入时间；涉受影响路径的结论必须用现二进制复测。近几轮巡查面（iterator/format/map-cache/equals/assign-chain/bool/array-sort 等）不受 EM-15 影响的论证：EM-15 变更全部集中于写访问器 expected_descriptor 表（build.rs 该路径），root 验收全门禁 2985 通过=既有行为零漂移，故仅涉 access$ 写访问器的结论需复测（仅本巡查）。**待办：DT-26 验收后重建 spared**（现主 target 已删，重建成本已评估）。

## 有效结论（不受二进制陈旧影响）

- **Probe 端全域恢复**（桥调用+this$0 物理事实）；**静态 long 位掩码完美**：`lflags |= 1L << b` → `IA.lflags = IA.lflags | 1L << arg0`（>32 位惯用法）；`testBit` `(lflags & 1L << b) != 0L`；行为 `56/50` + `true/false/1099511627778` 一致；
- access$000（跨类读）/access$100（跨类方法转发）恢复；CA 探针 javac23 腿 access$012 拒=账本已知独立缺口（读-改-写形）。
