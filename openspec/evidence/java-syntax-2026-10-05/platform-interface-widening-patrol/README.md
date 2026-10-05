# 平台接口实参单边扩宽巡查（2026-10-06 root）——第 6 族簇最小机制面

## 发现

- **顶层具名类复刻仍拒**（[fixture/AH.java](fixture/AH.java)）：`Collections.sort(out, new AC())`，`AC implements Comparator<String>`——诊断与 CP.byAnon 逐字同形（"presents `AC` but requires `java.util.Comparator` … no safe reference conversion evidence"）。**排除 `$` 伴生读取变量**：失败面是"用户类→平台接口"这一单边关系本身；
- **既有机制明确不覆盖**：`recover-snapshot-hierarchy-widening` 的 design Non-Goals 写明"单边快照外（用户类→平台目标）"；实现（facade.rs `snapshot_header_chain_widens_with` ~17217）对链上**每个节点先取 header**——平台接口目标（Comparator/Runnable）不在快照中，取不到 header 即停，即使源类自己的 interfaces 数组已逐字写着该目标；
- 关联锚：CP.byAnon（critical，`[30,10,20]` vs `[10,20,30]`）、Thread 匿名 ctor/TL$3→Runnable（安全拒形同因）。

## 最小机制（无需新类型系统）

快照 header 链走到**目标名**时，先于目标自身 header 判定命中：源类（快照内，header 已证）的 super/interfaces 数组逐字列出了目标，就是单边证明；目标自身是否在快照无关。两-sided 既有行为不变；平台→平台（java.io 流）不在此片（另被丢弃分配掩蔽）。
