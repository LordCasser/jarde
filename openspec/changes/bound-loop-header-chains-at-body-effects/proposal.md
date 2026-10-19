## Why

固定JADX原件TestBreakInComplexIf的完整三方基线表明，Jarde八个API组合均拒绝目标循环，JDK23完整源码重编缺return。实际同轮IR确认，AND表头链把带字符串构造和Map.get的body入口纳入条件候选，随后在StatementFree处拒绝整条循环；现有普通If/body表示本可保留这些效果的位置。

## What Changes

- 仅在已证明的AND表头测试之后，把精确StatementFree拒绝的后续候选作为普通body边界，保留已有纯测试前缀及完整边闭合检查。
- 复用现有单测试/多测试循环、普通If、break和来源表示；保留null短路、每轮一次lookup、break前后的更新顺序。
- 增加真实IR与公开API正反例，并完成原件/JADX/Jarde完整源码及隔离运行对照。
- 不放宽真正表头的纯度、OR链、异常边或ownership限制，不新增Region、pass、CFG改写、全局solver或依赖；不处理其它未证缺口。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 已证明的循环条件前缀后允许保留带效果的普通body条件，保持执行顺序与完整来源。

## Impact

主要影响jarde-java的Region表头链识别和对应测试。公开facade、IR/AST种类及Builder接口不变。依赖现有预算、物理身份和来源契约；完整原件基线已接受，恢复实现及最终验收尚未完成。Reader库存仅按实际新增fixture更新。CF09/71单元/612文件仍为partial。
