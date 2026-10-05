# Design：后缀自增旧值快照的后缀形呈现

## Context（root 已实测，四形判别）

| 形 | 字节码特征 | 现状 |
| --- | --- | --- |
| `int j = i++;` | `iload i(旧); iinc i,1; istore j` | ✗ |
| `return i++ + 10;` | `iload i(旧); iinc i,1; ldc 10; iadd` | ✗ |
| `++i`（前缀） | `iinc i,1; iload i(新)` | ✓ |
| `i++; j = i;`（拆） | `iinc i,1; iload i; istore j` | ✓ |

jadx 解法：`i++` 语句形/表达式直接呈现，或旧值算术等价（`i2 = (i - 1) - 1`）。时间引注表明 jarde 已**知道**旧值的存在（"is the value local 0 held at BCI M"）——缺的是归因。

## 决策 1：识别为后缀表达式（单消费方）

当模式为 `iload x; iinc x; <消费 iload 的旧值>` 且旧值恰一个消费方时，呈现 `x++` 表达式于消费位（`int j = x++;` / `return x++ + 10;`）。多消费方/消费方不明确 → 保持现状拒绝。**快照值在 SSA 中已是 distinct 节点**（时间引注证明），本片是呈现层归因，非新分析机制。

## 决策 2：呈现与源同构

`i++`（非 `i = i + 1` 再减——jadx 的算术等价形也可，但直接后缀形最忠实且更短）。呈现选择后缀形时**测试钉死**。

## 决策 3：零回退与负例

- 前缀/拆语句/复合赋值捕获新值三健康形逐字节不变；
- 负例：`i = i++`（旧值与存储目标同 slot 的自赋值）仍拒；多消费方仍拒；
- corpus 双腿扫描：预期 diff 为空（如实记录）。

## 验证标准（可证伪）

1. 主锚：CM2.immUse 恢复（`int j = i++;` 形）、CM2.postfixExpr 恢复（`return i++ + 10;`）、CM.incDec 恢复（四连形）；三类 `javac --release 8` exit 0、行为逐行一致（旧值语义精确——immUse=5 而非 6）；
2. 零回退/负例如上；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. 时间引注发出处与 SSA 旧值节点的现有表示（task 1.1 插桩——注意与 DT-03 alloc-qualifier 片交付的 SSA 值级细节复用）；
2. 后缀形 vs jadx 算术等价形的取舍——按决策 2 后缀形，若实现发现复杂度过高可降级为等价形（报告说明）。
