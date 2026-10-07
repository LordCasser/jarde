# Root 独立验收（2026-10-08，closing round，合并）

## 判据逐项

1. **1.2/1.3 回填复核**：planning 片的 5 pin 在两腿仍绿；行为锚 12/13 由 guard 族恢复且**现时 CLI 渲染与两片记录摘要一致**（锚 12 `79b61c4d…`、锚 13 `27d2b672…`）——不勾选理由（锚未闭）确已消失，回填成立。
2. **零生产码复核**：`crates/src` diff 仅 classfile.rs 的 census 断言重测（+16/−1，测试断言+登记注释，仓库惯例）——无恢复代码、guard 证书零触碰。
3. **2.x 交付抽查**：2.1 拒绝闭包断言（quote 块集==切片块集、逐指令落在被引区间、origin 同块集、无越界名）；2.2 独立 sibling 保持+依赖切片块级引注；2.3 固定 SHA 三方（10 行 trace 三方逐字含逃逸 AssertionError、拒绝形不可编译明示不计数）；2.4 停止点=成员首个局部访问 BCI（9/7/5/19/8）+ Stopped/NotProduced/空文本契约 + 两腿无半个成员；2.5 审计两裁决（变异 p3_try_local 的 fallback **属**同一局部切片→保留完整拒绝；typed-catch fallback **属**独立身份→块级引注已是缩窄）——裁决均可从块区间归因复核。
4. **root 实测**：新套件 5+1 ignored（25.35s 含 JDK 腿）+ planning 4+1 全绿；oracle 3/3。
5. **门禁（权威口径）**：全量 exit 0、**343 targets ok、0 FAILED**；fmt OK；CI 逐字 clippy `Finished` 0；openspec **314/314**。census 945→950/+20/+18/+21 逐项归因；指纹恰 9 新文件。
6. **CI**：合并推送后 run 为准（监控在案）。

## 账本

**`preserve-local-scope-across-exception-regions` 全部任务关闭**——长驻 change 完结（规划测试面 + guard 族恢复锚 + 原子拒绝/行为验收 + 2.5 审计）。残余位点移交：`IO.readAll`/变异 p3_try_local close（guard/copy 家族）、`ScopeRefusalsEscape` 未变异形（该家族后续）。
