# 循环体内 try-catch 巡查（2026-10-05 root）——大颗粒缺口

## 变体矩阵（决定性）

| 形 | 结果 |
|---|---|
| 纯 for + try-catch | **拒**（"graph is not reducible over 4 blocks [4,10,26,39]: a loop is entered at more than its header or two loops cross"） |
| while + try-catch | **拒**（同 irreducible） |
| for-each + try-catch（无条件 throw 简化形） | **拒**（诊断不同——另一引注族） |
| 循环无 try（对照） | 恢复 |
| 循环内 try-**finally**（loopFinally，continue 交互） | 恢复（if/else 等价分布，行为精确） |

**结论：具名 catch 在循环体内 = 全形拒绝**——解析循环/重试循环/逐项容错处理是业务代码最高频模式之一。

## jadx 对比：行为错码（jarde 拒绝正确）

[results/jadx-DF.java](results/jadx-DF.java)：jadx 把 `if(x<0) throw` **提出 try 之外**（永不被捕获，异常直接逃逸方法）+ 尾随多余 `i--`——对 `{1,-2,3}` jadx 重编版直接抛出（原版打印 3）。jarde 的响亮拒绝是对的，但该形状应当可证。

## 已恢复面（负结果）

- **default-前置 switch**：恢复为 default 移末（各 case 带 return——语义等价规范化）；
- **default 中位+贯穿**（`default: case 2:` 源序）：恢复为 `case 2: default:` **同目标块标签重排序**——两标签指向同一块，顺序语义无关，等价；行为 `10/0/100/200/200` 精确；
- 循环内 try-finally + continue：等价分布恢复（`+= 1000` 双路径），行为 `3/2004` 精确。

## 既有先例（扩展对象，勿平行造机制）

- `recover-fragmented-loop-catches`（f8be28ae 已合入）——CF-18 认证形（分段同 handler）；
- `proved_loop_catch_joins`（region.rs:1639）——判据：handler 单后继 join 且 join 在循环内、handler 无普通前驱、全行具名+保护区非空、保护区在循环体内且被 header 支配。
- **本缺口即"该两机制均不覆盖的简单形"**——落点应为其中之一的扩展。

## 处置

登记 summary + 立项 `recover-loop-body-try-catch`（大颗粒：region/异常域机制扩展，插桩先行）。
