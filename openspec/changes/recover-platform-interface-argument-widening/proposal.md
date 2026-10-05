## Why

[platform-interface-widening 巡查](../../evidence/java-syntax-2026-10-05/platform-interface-widening-patrol/README.md) + [comparator-anon 巡查](../comparator-anon-patrol/README.md)：用户/匿名类实现 JDK 接口（`Comparator`/`Runnable`）作方法实参时，"no safe reference conversion evidence" 拒绝整条调用——CP.byAnon 是 critical（幸存 `[30,10,20]` vs 原 `[10,20,30]`）。顶层具名类复刻同败，排除 `$` 伴生变量。

## Root 预审计

- 快照 hierarchy 扩宽已存在（facade.rs `prove_snapshot_hierarchy_widenings` / `snapshot_header_chain_widens_with`，change `recover-snapshot-hierarchy-widening`），但 design Non-Goals 明确排除"用户类→平台目标"；实现对链上每个节点（含目标）先取 header，平台接口目标不在快照即停；
- 源类的 class-file header（快照内、已按字节读取）**逐字列出**其 implements/extends——目标名出现在该数组即单边证明，无需目标的 header。

## What Changes

- `snapshot_header_chain_widens_with` 在走到目标名时**先判命中、后取目标 header**（源侧 header 链仍逐级完整证明）；
- 仅此单边放宽：目标必须是快照内某类 header 的 super/interfaces 名；两-sided 快照证明、平台闭集表、数组/Object 回答全部不变。

## 硬不变量

1. 快照内两类（两-sided）既有扩宽渲染逐字节不变；
2. 目标未出现在任何快照内类 header 上时保持现行拒绝（不查 classpath、不猜）；
3. 平台→平台（如 java.io 流 ctor）不在本片。

## 验收

- CP.byAnon：sort 调用完整呈现，剥离编译 exit 0、运行 `[10,20,30]`；
- AH.byTop（顶层具名对照）：`[a, bb, ccc]` 一致；
- AN own-interface 四位（既有两-sided 正例）零回退；无接口关系的负例仍拒；全门禁。
