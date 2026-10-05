# Design：初始化 dance 值的任意消费方位

## Context（root 已实测，六位判别）

| 位 | 形 | 现状 |
| --- | --- | --- |
| 局部赋值 | `int[] x = new int[]{1,2}` | ✓ |
| 字段赋值 | `f = new int[]{3}` | ✓ |
| 实参 | `sum(new int[]{5})` | ✓ |
| 外层初始化器元素 | `new int[][]{ new int[]{1}, … }` | ✓ |
| 元素存储 RHS | `partial[0] = new int[]{7}` | ✗ |
| 立即下标 | `new int[]{9}[0]` | ✗ |

jadx 两失败位皆恢复；`new int[2].length`（裸立即消费）合格证明非"立即消费"机制。

## 决策 1：合格条件 = dance 单写者 + dup 单读者（既有读者不变量的新值形）

d`ance` 的 dup 引用是 SSA 意义上的一个值：**恰一个**后续消费方（aastore 值位或 arrayload 接收者等）时呈现于该位；零读者（丢弃形）归丢弃分配片；多读者保持拒绝。即本片把"具名目标"判据泛化为"单读者消费方"，与 init.rs 读者门的三分支语义对齐，但落在数组的 dance 值上（build.rs 侧）。

## 决策 2：呈现保持显式形

`a[0] = new int[]{7};` / `return new int[]{9}[0];`——与四位合格位同形（全仓一致），不做简写美化。

## 决策 3：零回退与负例

- 四位合格位 + 裸立即消费（`new int[2].length`）+ 锯齿整族逐字节不变；
- 负例：dance 值被两处消费（手工构造）仍拒；
- corpus 双腿扫描：预期 diff 为空（如实记录）。

## 验证标准（可证伪）

1. 主锚：MD.partSet 恢复（`MD.partial[0] = new int[]{7};` 呈现）、MD3.bareIdx2 恢复（`return new int[]{9}[0];`）、MD/MD2/MD3 三类 `javac --release 8` exit 0、行为逐行一致；
2. 零回退/负例如上；corpus 空 diff；
3. 门禁全量（基线以合并态为准）+ fmt + CI-exact clippy + openspec strict + `git diff --check` + fingerprint 再生。

## Open Questions

1. "copy … has no proved local assignment" 的发出处与 dup 值的现有 SSA 表示（task 1.1 插桩）；
2. 实现层次（读者门泛化 vs dance 消费方枚举）——按 1.1 的实际落点选最小改动，报告说明。
