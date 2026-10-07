# 诊断族普查第二轮（2026-10-07 root，收口后重测）

当前主线（spn 合并前二进制，本周 15+ 片后）重渲染 10-05 巡查冻结 jar（45 随机采样）并聚类 `// the …` 诊断行。

## 结果：**96 行 / 45 类 ≈ 2.1 行/类**（10-05 普查时四主族主行+级联即 55+10+9+6+40+… 远高于此）

| 残余模板 | 次数 | 归因 |
| --- | --- | --- |
| "saved producer … no bounded final expression consumer" | 26 | 第 4 族 phi 膨胀级联行（上游各异，见下） |
| "belongs to no shape this run verified" | 23 | 通用级联行（同上，随主因消失） |
| "the copy at BCI N has no proved local assignment" | 15 | copy 族**登记边界**残余（物化入局部 boolean 位形等） |
| "array instruction … nothing in this body reads" | 5 | 数组死值拒绝（独立小面） |
| phi 膨胀 "3 consumers" | 4 | 同 census 修正 |
| irreducible / conditional-type join / entry-state | 各 1-3 | 独立已知域 |

## 逐锚归因（top 残余载体）

- **OP2（22 行）**：`condAssignOld`/`main`——**已登记边界**（`recover-short-circuit-local-values` 的 boolean 位置判据：`proves_boolean_local_store` 只认 putstatic-Z/boolean-ireturn/append-Z，三元 `ifeq` 读不在内）。
- **RH 家族（13×6）**：array-element-receiver 巡查的 driver `main`——已验收片的**多形状混合 main 级联**（数组死值+copy 级联），非新族；其 critical 面（RG put）已由该切片关闭。
- **BT（9）**：bitset-ops 巡查的 copy 族升级面（已在 soundness 守卫下 SAFE）。

## 结论

**前沿已从"新族发现"转入"登记边界收口"**。残余全部有归属：copy 边界（15）、short-circuit-local 位置（OP2）、driver 级联（RH/BT）。下一优先级按防回归价值排列：fixture 行为守卫（在飞）→ capture-ctor-super-order（呈现缺陷）→ boxed-widening → local-scope 12/13 锚。

方法注：本轮采样 45/154 巡查 jar（shuf 固定源）；逐锚归因遍历全量 jar。
