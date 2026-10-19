## Why

完整 `TestDoWhileBreak2` 三方基线确认：Jarde 的8个API组合均回退，16次整类重编失败；JADX完整源码能重编并与原件运行一致。真实IR表明直接返回叶块已有证明，却在后测循环使用该证明前被goto前缀门拒绝，需要补齐现有恢复路径。

## What Changes

- 支持已获证的普通后测循环内提前返回，保留体首取值/转换/赋值、空值分支、体尾测试及循环外返回的次序。
- 复用既有终端返回叶块与后测循环证明，只让确已认证的返回叶块绕过goto专用前缀检查；未证明路径继续拒绝。
- 由Luna私有实现，root审查后执行真实Reader/公开API、完整三方、对抗性回归、完整工作区及本产品自身CI。
- 前提为已闭合原件基线及同轮IR证据。非目标：新增CFG/AST/全局恢复机制、泛化任意多出口图、修改旧预算测试、修复独立泛型字段/映射/runner债务或扩展Dex输入。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：补充后测循环直接返回的可观察行为及拒绝/停止边界。

## Impact

主要涉及 `crates/jarde-java/src/region.rs` 的候选判断和既有返回叶块证明传递、真实fixture与公开回归；不增加依赖或公共API。完整基线见 `openspec/evidence/java-syntax-2026-10-19/cf09-last-four-original-root-v1/`，后续验收不得借用旧产品CI，CF09/71/612保持partial。
