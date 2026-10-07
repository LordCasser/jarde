# Root 独立验收（2026-10-07，合并）

## 判据逐项

1. **HEAD 复验裁定**：实现片澄清巡查记载——AS 在 HEAD 是 `produced` 非 `refused`（0 诊断、`compile_status=not_attempted`），不可编译是**静默**面（仅 javac 报）——比巡查的"SAFE 响亮"更准确，本片价值因此更高（从静默不可编译到可编译）。
2. **门控复核**：准入单独翻转两形（`local0[0]=…` → `((java.lang.Object[]) local0)[0]=…`，双腿 exit 1→0）；同型/int[]/无证明组件存入逐字节。
3. **diff 审查**：`widen_covariant_store_receiver` +59/−2（单规则置于数组通道读者旁）——规则=呈现组件为非 Object 引用 ∧ 值呈现类型为异型引用（初始化器已证集的**否定**）；cast 携带接收者自身 origin、应用一次、宽化引用转换不可失败、`aastore` 运行时检查保留。读取侧/初始化器规则/窄存臂/发射器零触碰。
4. **root 实测**：AS 渲染 0 引注 + 2 处宽化接收者；定向 4+4 ignored 回放绿（ASE1/ASE2 三方文本一致）；oracle 3/3。
5. **门禁（权威口径）**：全量 exit 0、**339 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **312/312**（实现片轮有 d3 计时 flake，复跑 ×2 绿）。
6. **corpus**：6/6/1 delta 全为本片 AS/SD/SC+巡查 as.jar；指纹 +12 纯增；census `(899,3867,334,2445,8)→(907,3911,344,2453,8)`。
7. **CI**：合并推送后 run 为准（监控在案）。

## 边界裁定

- **未证兼容形也宽化**（SC：`String` 入 `CharSequence[]`）——**接受**：封闭层的"零子型判断"设计后果，行为与可编译性不变仅文本变化，SC 钉为可测边界；收窄需层级事实另片；
- 无组件事实接收者（UB.merged）保持现状含其不可编译声明——不猜，钉住。
