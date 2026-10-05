# 平台接口实参单边扩宽巡查（2026-10-06 root）——第 6 族簇最小机制面

## 发现

- **顶层具名类复刻仍拒**（[fixture/AH.java](fixture/AH.java)）：`Collections.sort(out, new AC())`，`AC implements Comparator<String>`——诊断与 CP.byAnon 逐字同形（"presents `AC` but requires `java.util.Comparator` … no safe reference conversion evidence"）。**排除 `$` 伴生读取变量**：失败面是"用户类→平台接口"这一单边关系本身；
- **既有机制明确不覆盖**：`recover-snapshot-hierarchy-widening` 的 design Non-Goals 写明"单边快照外（用户类→平台目标）"；实现（facade.rs `snapshot_header_chain_widens_with` ~17217）对链上**每个节点先取 header**——平台接口目标（Comparator/Runnable）不在快照中，取不到 header 即停，即使源类自己的 interfaces 数组已逐字写着该目标；
- 关联锚：CP.byAnon（critical，`[30,10,20]` vs `[10,20,30]`）、Thread 匿名 ctor/TL$3→Runnable（安全拒形同因）。

## 最小机制（无需新类型系统）

快照 header 链走到**目标名**时，先于目标自身 header 判定命中：源类（快照内，header 已证）的 super/interfaces 数组逐字列出了目标，就是单边证明；目标自身是否在快照无关。两-sided 既有行为不变；平台→平台（java.io 流）不在此片（另被丢弃分配掩蔽）。

## 处置（2026-10-06，change `recover-platform-interface-argument-widening`）

AH/AC 的失败与 CP.byAnon 同因（顶层具名类排除了 `$` 伴生读取）：`f6.jar` 的 `AC` 在自己的
class-file header 里逐字写着 `java/util/Comparator`，而旧实现的等价游走**先取目标 header**，
平台接口不在快照即停。命中判定前移后：

- `f6.jar` 重渲染：[`results/jarde-AH-after-platform-interface-widening.txt`](results/jarde-AH-after-platform-interface-widening.txt)
  ——`refusals = 0`，`AH.byTop` 写出 `java.util.Collections.sort((java.util.List) local1, (java.util.Comparator) new AC());`；
- 判据的负例（`AW$MyErr`→`java.lang.Throwable`：目标名不出现在任何快照 header 上）保持现行拒绝
  逐字不变；平台→平台（java.io 流 ctor）仍不在本片。
