# 嵌套三元与条件链极限巡查（2026-10-05 root）

## 健康面（负结果）

[fixture/TC.java](fixture/TC.java)/[TC2.java](fixture/TC2.java)/[TC3.java](fixture/TC3.java)（`--release 8`）：
- **右结合链**（`x<0?-1:x==0?0:x>9?9:x`）、**嵌套括号**（`a ? (b ? "ab":"a") : …`）、**三元作实参**、**短路+if**（`if(a&&b)` 嵌套展开形）、**纯右链**全部恢复；
- 深链 15/-3 值语义精确。

## 发现：短路复合条件作三元条件（三元片判别 +1 维度，已立项域）

`(a && b) ? … : …`、`(a||b) ? …`、`((a||b)&&c) ? …` **全部拒绝**——判别矩阵：

| 条件形 | if 消费 | 三元消费 |
| --- | --- | --- |
| 简单条件 | ✓ | ✓ |
| `&&`/`||` 短路复合 | ✓（嵌套 if 展开） | **✗** |

**jadx 直接还原** `(z && z2) ? "both" : "one"`（[results/jadx-TC2.java](results/jadx-TC2.java)）。归 [#1 三元拆分片](../../../changes/recover-divergent-ternary-statement-split/)——但注意该形**分支同型**（String/String），与该片的异型汇合前提不同：这是**短路复合条件进三元**的独立维度（同 if/三元消费差）。补锚到该片 + 诊断差异（jsr/多 owner vs two-values-joined）已在 [polymorphic 残留巡查](../polymorphic-remainder-patrol/README.md) 记录的"变体不同落点"警示内。

## 处置

三元片锚家族 +1 维度（短路复合条件）；不新立（同片实现时插桩确认三变体落点关系）。
