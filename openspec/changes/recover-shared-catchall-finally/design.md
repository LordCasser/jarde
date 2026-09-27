## Context

`FinallyOnce.handled` 的异常表为 `[4,21)->31 IAE`、`[4,21)->65 any`、`[31,55)->65 any`。正常副本 21–26、具名 catch 正常副本 55–60、异常副本 66–71；正常返回先在 BCI 20 保存 slot 1，再从 29 读取/30 返回，catch 返回先在 54 保存 slot 2，再从 63 读取/64 返回，handler 在 65 保存原异常 slot 3，74 读取/75 重抛。同一个 catch-all handler 65 同时是 try 和具名 catch 的异常后继。slot 1 还被具名 catch 参数复用，其 SSA 定义不同；不能放宽词法局部范围来掩盖当前 `local 1 crosses a quoted fallback region`。

现有 `guard.rs::prove_finally_copy` 在 1 条 catch-all、1 个正常返回和 1 个异常副本上证明 SSA 归一化、半开异常范围、保存值/同对象重抛；`Plan` 只有完整证明后才 Claim。结构化 `finally` 正文经受限子 Region 和已有 `Try.finally_body` 输出，暂存所有权再提交。当前多行竞争被明确拒绝。JADX `MarkFinallyVisitor` 的“先找 catch-all/scope，再比较每个出口的副本，最后抑制重复指令”可参考调查顺序；其 `DONT_GENERATE` 可变块和本例正常路径重复清理不能作为 Jarde 的证明或运行 oracle。

## Goals / Non-Goals

**Goals:** 仅在上述三行、三份清理和两条返回完成流的一一对应被完整证明时，输出一份 `try { … } catch (IAE e) { … } finally { cleanup(); }`；保留两条返回值快照、同一异常对象的重抛和每条路径一次清理。完整源码可用 Java 8 重编，Jarde 与原 class 正常/catch 计数一致。

**Non-Goals:** `escaping()` 的只有异常出口形态、原 `FinallyOnce.main()` 的资源误判、多个具名 catch/handler、循环或 switch 内 finally、任意覆盖/返回替代、跨 quoted fallback 的一般局部范围修复，以及沿用 JADX 的错误副本去重。

## Decisions

1. **先行证明三条异常行的优先级与半开边界。** 以物理 row ordinal、`start_pc/end_pc/handler_pc/catch_type` 匹配唯一具名 IAE 行及两条共享同一 catch-all handler 的范围；前者必须先于 try catch-all 覆盖 `[4,21)`，后者再精确覆盖具名 catch 正文 `[31,55)`。正常清理的任何可抛指令不得落入其对应 catch-all 的保护范围，避免清理再次被 handler 捕获。JVM 有效的扩围 `[0,23)->25` 反例必须继续拒绝，不能将真实两次清理折成一次。
2. **对每个完成出口逐份比效果和值。** 用既有 SSA 归一化/Operation 身份、相同调用或字段目标、实参稳定性与顺序检查 21–26、55–60、66–71 清理等价；每个副本有唯一入口和完成后继，没有额外消费者/分支。正常返回的 slot 1 和 catch 返回的 slot 2 都在清理前保存，返回各自的同一 SSA 值；handler 保留并重抛进入它的同一异常对象。尤其区分复用的 slot 1 在具名 catch 的新 SSA 定义。任何额外行/边、值变化、不同目标或清理替代完成都拒绝。
3. **只扩现有私有 Guard 证书和所有权提交。** 成功后将一个具名 catch、两个正常出口与共用 catch-all 作为同一 `Plan` 的完整 owned/facts 闭包；受保护 try/catch 正文用现有受限子 Region 恢复，不把异常清理复制进任一正常正文。Region 只能在证书通过、三个副本和所有 BCI 均被解释后一次性提交 visited；失败恢复 checkpoint，引用完整候选。现有 `Try` AST 同时容纳 catches 与 `finally_body`，无需新异常节点或独立后处理器。
4. **优先用最小类族验收语义。** 独立 `handled`+计数器与外部 Runner 可让 catch 返回字面量，以免字符串拼接掩盖本项；应覆盖正常返回、具名 catch 返回、未匹配异常重抛和清理自身抛错覆盖原完成（后两者仅在相同有界形态可表达时纳入）。前两者必须三方 Java 8 全类重编验证运行，并单列固定 JADX 的正常路径重复清理差异。再回放原 `FinallyOnce`：若 `escaping/main` 仍引用，只声明 `handled` 恢复，不把整类误标已追平。证书须记录三条异常行，source map 覆盖三份副本和两个 saved-return 的物理 BCI；预算、取消原子停止。

## Risks / Trade-offs

- [把 catch 正文副本当成 try 正文或丢失具名 catch 优先级] → 按 row ordinal 与半开范围证明每条真实异常边，两个保护区和 handler owner 必须闭合。
- [在清理后重新计算返回或丢失原异常] → SSA 证明两条 saved value/return 和 handler 同对象 rethrow，并用副作用计数运行对照。
- [JADX 可编译但清理重复] → 以原 class 为语义 oracle，固定 JADX 保留为有记录的差异，不作为等价断言。
- [多出口子 Region 部分认领] → 先暂存完整 Guard/Region/Builder 状态，未闭合时回退为原物理引用。
