# CF-13 巡查：switch 内 continue 的出口所有权（2026-10-01）

[CF-13 账本](../../jadx-feature-inventory-2026-09-27/control-flow.md)登记"已证差距，限于 switch-local join 与 loop-update 同时出边"的展开取证（主线 `4ac1bac7`）。固定转录 [fixture](fixture/)（W1/W2，SHA 见 [results/fixture-sha256.txt](results/fixture-sha256.txt)）；基线 JSON 在 [results](results/)。

## 形态与结果矩阵

字节码（两形共通）：循环头 4–6（`i<n`）；selector 9–12（`i%3` + lookupswitch）；case 0/1 体后 `goto 63`（join）；default 含 `if_icmple 60 / goto <LATCH>`（continue 绕过或等于 join）；join 63 后（W1 有 `total+=10`）到 latch `iinc 2; goto 4`。

| 场景 | 主线 Jarde |
| --- | --- |
| W2.noCont：无 continue（所有 arm 汇 join） | 完整恢复 `while { switch {…} total+=10 }` 形态，行为一致（`70`） |
| W2.contNoJoin：continue 直跳 latch（join==latch） | 整方法 quote：`jre_region_ownership_overlap`@BCI 9 |
| W1.mix：continue + join 后语句 | 同上 quote（行为基准 `70`/`36`） |

## 根因（scratch worktree 插桩取证）

1. **主缺口（出口所有权）**：loop 体内 switch 的 arm 存在直跳 loop latch 的边（continue lowering）时，主 walk 未能构造 `Loop { Switch { arm 内含 continue } }`——退化为一组 Fallback。判别变量精确：同一形状去掉 continue 即恢复。仓库已有 `Region::LoopContinue`/`LoopBreak` 呈现（labeled-loop 切片刚验收 `continue loop;`），缺的是**switch arm 走查接受"arm 出边 = 外层 loop latch"并产 arm 内 `continue` 语句**的构造路径。
2. **次生缺陷（诊断误导）**：退化的 Fallback 生成三个块集重叠的 region（`[4,9]`、`[9]`、`[69,…]`），最终报 `ownership_overlap`——把"构造失败"误报为"所有权重叠"。fallback 路径应划分而非双 claim 块集（即便最终整体 quote，诊断也应指向真实首个失败）。

## 处置方向

`recover-switch-arm-loop-exits`：switch 构造处识别 arm 出边落在外层 loop 的 latch/test 集内 → 该 arm 以 `continue`（无 label，loop 直接外层）呈现并从 switch join 候选中排除该 arm；join 取其余 arm 的公共汇合。W1 形态（join 存在 + continue 绕过）与 W2 形态（join==latch，全部 arm 以 continue/自然落出）都要闭合。次生的 fallback 重叠双 claim 一并修（fallback 划分互斥或统一为单一 quote region），使诊断指回真实原因。

原 class 为行为基准；账本登记的 JADX 反例（continue 后留不可达 break、不可编译）不作参照。
