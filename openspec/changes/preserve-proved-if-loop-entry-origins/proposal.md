## Why

真实JADX `TestLoopRestore2.raung` 装配输入的四个API均恢复为结构化Java，完整源码八次编译成功，但来源映射只覆盖十条物理指令中的九条：if臂进入嵌套循环的BCI8跳转丢失。已有分支/循环结构足够；应补来源归属，而不是增加语法恢复机制或模仿JADX不可编译的while文本。

## What Changes

- 为已证明的if非空臂前导、唯一transfer-only块到紧邻嵌套循环入口的跳转，在真实已发出的if语句上保留派生物理来源。
- Java文本、物理块owner、原始事实和既有拒绝保持不变；缺证不猜来源，真实停止继续走既有事务。
- 冻结原始Raung class及完整三方基线，补四API完整BCI映射和边界回归；本片已通过root定向验收，完整workspace/本产品自身CI仍待收口。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 受证if臂进入嵌套循环的隐式跳转保留物理来源，且不改变语法、所有权或停止契约。

## Impact

可能仅涉及jarde-java Builder的if来源收集及其小范围测试、永久原始fixture与Reader/指纹库存。复用既有Region/OriginSet/预算/发布协议，不改canonical/Loop.gateway_origins语义、公共API、依赖或CLI。前置为已接受真实基线及当前双入口产品CI/源码绑定收口；此前Luna只能私有实现。非目标：通用goto映射、任意结构来源完备保证、常量折叠/while风格、DEX前端、运行等价或CF08/71/612整项关闭。

证据入口：`openspec/evidence/java-syntax-2026-10-19/cf08-looprestore2-original-root-v1/README.md`，原class SHA4fd1503499a049efa8e736a98de4339b6504973f80208987ed0774073df9c447；完整20实际命令（装配4/基线16），JADX两次完整源码编译失败均保留，目标方法从未调用。
