## Why

完整 `TestBreakInLoop3` 基线发现：上游测试虽保留NYI且未启用，但当前JADX实际能整类重编，并在双runtime得到与原件相同的builder与check结果；Jarde的8个API均因BCI11重复归属回退，16次整类重编失败。真实IR已定位为catch回跳递归重开同一Try，需要在现有异常重试架构内闭合归属。

## What Changes

- 对具有未受保护前缀的普通具名catch重试建立精确证书；handler的一臂重试，另一臂在内部条件汇合后到正常出口。
- 保留每次尝试前的前缀、精确保护区间、标志更新、重试边、catch内部效果与循环后效果的顺序和异常作用范围。
- 复用既有Try lead、重试循环证书和If/Sequence/LoopContinue；保留最终owner查验及失败原子回滚，不以递归展开重复执行路径。
- Luna私有实现，root架构审查、真实Reader与负例、公开API及完整源码三方运行、完整工作区和本产品自身CI通过后验收。
- 前提为完整原件基线及同轮IR探针。非目标：把NYI当成已启用JUnit、将异常边改成普通边、通用不可约图重写、任意多handler/保护块、Dex前端、删除重复owner检查、重做旧finally/Loop2/Loop4/5或独立工具债务。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：补充带未受保护前缀和正常完成catch分支的受证重试恢复。

## Impact

主要涉及 `crates/jarde-java/src/region.rs` 既有named catch/retry证明、Frame中的精确转移授权及其真实fixture/公开回归；无新增依赖或公共协议。`guard.rs`/Builder仅在实际证据证明需要时沿既有契约最小调整，禁止以宽松catch或局部类型猜测补洞。原始三方基线见 `openspec/evidence/java-syntax-2026-10-19/cf09-last-four-original-root-v1/`；CF09/71/612保持partial，原Smali/Dex6仍单列输入边界。
