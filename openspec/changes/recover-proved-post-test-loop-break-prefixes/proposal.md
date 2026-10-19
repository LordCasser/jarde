## Why

完整原始 TestDoWhileBreak 已实际复现缺口：八API均拒绝、16次Jarde整类重编均失败；原件与可编译的JADX结果运行一致。同轮IR确认单块赋值出口被既有纯goto桥门排除，随后header-tested路径在latch分支拒绝，并非缺少ForHeader或新的循环表示。

## What Changes

- 为普通后测循环证明一个独占、单块、含语句且最终跳向同一break出口的前缀，保留其语句执行位置和次数。
- 复用现有latch-tested loop、If、scope、transfer gateway和LoopBreak，不改变纯goto桥的原承诺，不新增CFG/Region/pass或外部依赖。
- 以完整上游原件、真实IR边界及实际预算取消验证；只有最终完整源码重编/运行通过才能关闭任务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：受证后测循环的单块break前缀恢复及其所有权、来源和停止契约。

## Impact

主要落在region.rs现有循环形状证明；Builder继续处理原本的普通赋值，不新增值变换。公共协议与canonical图不变。原始源码、两profile的class/plainJAR、JADX许可、真实失败和同轮IR分开归档；原件/JADX仅在输入与CLI绑定闭合时精确复用。主线050产品的continue汇合及数组减法已完成自身CI独立验收，本片是其后的独立增量，不将新计划当成已实现。
