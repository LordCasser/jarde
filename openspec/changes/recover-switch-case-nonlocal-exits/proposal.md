## Why

判别矩阵证明：循环内 switch 的**局部出口**（case 内 `break` → 循环尾、`continue` → 下一迭代，含混用形）与同构 if+return 全部恢复；**非局部出口**全拒——case 内 `return`（±循环尾）与 case 内 `break outer` 同文本拒（"local 1 crosses a quoted fallback region"，@bytecode 整方法引注）。该形（循环内状态机 switch 的早退）是业务代码常见模式。jadx 对 return 形**完整正确恢复**（有解证据）；对 labeled-break 形产出**行为错码**（降级 break + 循环内补 return → 恒单迭代；`{9,1,5}` 原版 10、jadx 重编 9）。

取证：[switch-nonlocal-exit-patrol](../../evidence/java-syntax-2026-10-05/switch-nonlocal-exit-patrol/README.md)（判别矩阵 + jadx 双形对照 + 错码实证）。

## What Changes

- MVP 先行：**case 内 return**——呈现层归因（局部值跨引注区）复用既有出口/汇合证明；插桩先行（实现者改动前回答，落盘 change 目录）：
  - **Q-i**：`retInSwitchNoTail` 的引注触发链——为何 if 版同构恢复而 switch 版拒（tableswitch 的哪条结构事实使局部 1 的 def-use 切片无法词法绑定）；
  - **Q-ii**：与 local-scope 片（preserve-local-scope-across-exception-regions）的落点关系——同一呈现门的不同入口（合并实现）或独立门；本形无任何异常结构。
- 第二步：**case 内 break outer**——需要标签重建等价证明（jarde 在纯循环位已支持 break-outer 标签保留，patrol-labeled-block 实证）；jadx 自己在此形行为错码，验收以行为逐行一致为准（含 `{9,1,5}`→10 判别输入）。
- 拒绝边界保持：出口结构不可证明时整方法响亮拒绝。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：扩充可证明的循环内 switch case 非局部出口恢复，保持出口归属、效果顺序与拒绝边界。

## Impact

主要涉及 `jarde-java` 呈现层（局部 def-use 词法绑定）与 switch/循环出口汇合证明；验收 fixture：[switch-nonlocal-exit-patrol](../../evidence/java-syntax-2026-10-05/switch-nonlocal-exit-patrol/) 的 SN.java（四锚）+ SM.java 五对照零回退 + SL.java mixed 终极形。
