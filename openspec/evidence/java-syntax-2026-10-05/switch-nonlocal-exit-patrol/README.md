# 循环内 switch 非局部出口巡查（2026-10-05 root）——新窄缺口

## 判别矩阵（决定性）

| 形 | 结果 |
|---|---|
| case 内 `break`（±循环尾语句） | 恢复 |
| case 内 `continue` | 恢复 |
| break+continue 混用（无尾） | 恢复 |
| 空 break 无尾（与 continue 不可区分形） | 恢复 |
| **同构 if 版** return+尾（对照） | 恢复 |
| **case 内 `return`（±尾）** | **拒**（"local 1 crosses a quoted fallback region"） |
| **case 内 `break outer`** | **拒**（同文本） |

边界锤定：**switch case 内的非局部出口（return / labeled break-outer）**；局部出口（break→循环尾、continue→迭代）与同构 if+return 全恢复。

## jadx 三方

- `retInSwitchNoTail`：jadx **完整正确恢复**（return in case 直排）——有解证据；
- `labBreak`：jadx 把 `break outer` 降级为 `break` + 循环内补 `return i`——**循环恒单迭代**；判别输入 `{9,1,5}` 原版 10、jadx 版 **9**（行为错码实证，[Drv 驱动]）；jarde 拒绝正确，但源级标签重建（jarde 在纯循环位已支持 break-outer 保留）应当可证。

## 诊断归位

"local 1 crosses a quoted fallback region" 与 local-scope 片同文本，但本族**无任何异常结构**——触发源不同（可能是同一呈现门的另一入口或另一门）；实现者插桩确认与 local-scope 片的落点关系（同门则合并实现，异门则独立）。

## 处置

登记 + 立项 `recover-switch-case-nonlocal-exits`（MVP：return-in-case 先行——jadx 已证有解；labeled-break-outer 第二步——需标签重建等价证明）。
