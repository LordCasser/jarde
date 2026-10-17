## Why

取证基线e575的完整EX仍将`finThrow(I)String`的try分支抛异常/返回与落穿finally输出为fallback。父任务明确保留这一正向欠账；本片先闭合该形，避免把固定FY/FX/FO已通过误写成finally家族全部完成。

## What Changes

- 固定EX原class与独立完整BranchThrowFinally的双编译器红基线，实际读取Facts/SSA、首个拒绝点和Region，不从相同诊断推断原因。
- 优先复用现有finally副本、SavedReturn和受限body walker，补齐分支throw/return的完整值、异常行、cleanup及所有权证明；若既有证明已充分，只修正消费接缝。
- 保留try真实抛异常、正常返回先保存、finally只执行一次以及cleanup抛错替代待完成结果的语义；加入真实class变体、预算/取消门和公库/CLI回归。
- 原样重编完整生成类并与原class/JADX对照，保留EX其它红方法；本片不关闭父整体任务或增加71/612计数。

## Capabilities

### New Capabilities

无新增能力目录。

### Modified Capabilities

- `java8-recovery`：增加单catch-all下分支throw/单saved return穿落穿副作用finally的有界恢复与完整对照要求。

## Impact

预期落点`guard.rs`、`region.rs`及已有Builder的finally消费。复用标准库、Facts/SSA、Finally/SavedReturn/Continuation，无新增依赖、公共AST/Region变体或通用等价solver。先完成取证后选最小实现；nestedFin并行分析，finally自身return/throw及exception-only tryThrowsFinallyRuns保持独立边界。
