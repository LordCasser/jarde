## Why

[sif 取证](../../evidence/java-syntax-2026-10-03/single-interface-fold-patrol/sif/README.md)**证伪了本片原前提**：单接口子不折叠的判别量不是"接口子"而是"根文本带 lambda 伴生投影"（2×2 矩阵：单接口/单类子无 lambda 均 FOLD；带 lambda 的类子同样 no-fold）——原巡查归因变量混淆，已更正。真缺口为两处可分离小项，本片取其一：**token 锚定匹配器漏 `CpEntryKind::InterfaceMethodRef` 的 owner 命中**（`WCallI` interface 调用形 no-fold vs `WCallC` 虚调用形 FOLD——接口方法引用的 owner 类型不参与锚定匹配）。agent 最小补丁实测：多折叠 4 个 corpus 类、既有家族逐字不变、`F1` 折叠产物重编运行与原 class 一致。

## What Changes

- 锚定匹配器接受 `InterfaceMethodRef` 条目的 owner 类名作为折叠成员名的匹配源（与 Fieldref/Methodref owner 同一判据位）；不改变锚定健全性要求（覆盖段/CP 索引既有校验）。
- WCallI 形恢复折叠；WCallC/既有全部家族 diff 逐字不变；Y1 折叠仍受**根重投影门**限制（独立大切片 `recover-fold-context-projection-preservation` 处置——**该切片已落地，Y1 双呈现已由其达成**；本片为其互补的锚定判据位）。实测 corpus 变化为 **5 类**（F1/SDAbstract/SDIndirect/pkg.SDPacked/SDDiamond——后者为 owner 扩展与 fcp 生产者锚的合取，实现者消融实验证实；与 fcp 的 8 类无交集）。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：接口方法引用的 owner 参与折叠成员名锚定，接口调用形家族可折叠。

## Impact

`crates/jarde-java`/`src` 锚定匹配器单判据位及测试；无新机制。既有折叠/锚定零回退。
