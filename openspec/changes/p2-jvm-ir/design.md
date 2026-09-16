## Context

以 P1 完成为进入条件。本 change 规划消费 PhysicalView/RuntimeView、结构 consumer 和 evidence/coverage，建立 Java 8 恢复前的 IR 底座；架构要求把 JVM 栈语义与 Query/XRef 分离，并允许按需升级到方法 Body，当前尚未实现。

## Goals / Non-Goals

**Goals:**

- 用 provider 和不可变 snapshot 实现 demand-bound resolution。
- 建立 raw CFG → legacy normalization → CanonicalCFG → Frame → SSA/effects 的显式契约。
- 在验证未完整实现时输出 verification=NotPerformed，并提供 Bytecode/Conservative fallback。

**Non-Goals:**

- 不实现完整 Java AST、lambda/accessor/concat/enum 等 P3 恢复。
- 不把静态 Frame 分析等同于 JVM verifier，也不执行 bootstrap 或目标代码。
- 不为了 IR 而修改 P1 的原始 X1 evidence、BCI 或 view identity。

## Decisions

1. **Resolver 使用显式 providers 和状态型结果。** 平台、依赖和 loader policy 通过 provider 注入；相比返回 `Option<Definition>`，状态型结果能表达 Missing/Ambiguous/Unsupported/BudgetExceeded。
2. **Legacy normalization 位于 raw CFG facts 之后。** jsr/ret 的 returnAddress 传播需要控制流和异常信息；相比线性替换，raw→clone/normalize 保留 origin 且能在膨胀时 fallback。
3. **IR 分层并传播不可变 OriginSet。** BytecodeIR 保留原始事实，CanonicalCFG/SSA/Region 只增加受控派生；每个 pass 使用 descriptor 校验依赖和 invalidation，避免 giant mutable IR。
4. **异常和副作用是独立模型。** 用 throwing instruction、handler order、保护区间和 effect sequence 约束 rewrite；相比只用 block edge，能避免 finally/monitor/初始化语义错置。
5. **输出先求诚实降级。** Structured Java 等待 P3；P2 以 Bytecode 表示为可靠基线，低级 Java 或混合输出必须明确语法/编译状态；quality 单独记录 Conservative/Fallback，不把不可证明的替代代码当成恢复成功。

## Risks / Trade-offs

- [Risk] returnAddress/共享子程序克隆导致 IR 膨胀 → context/clone/byte budget，超限 fallback。
- [Risk] 缺失平台 Header 使类型状态传播不完整 → 保留 phantom/unknown 标记，禁止唯一解析证明。
- [Risk] 异常 effect 模型增加实现成本 → 先覆盖指令级 throwing/effect contract，再扩展 rewrite。
- [Risk] Pass 依赖或 invalidation 漏记 → descriptor registry 启动校验和每 pass invariant test。

## Migration Plan

以 P0/P1 result 语义和 JSON 形状作为输入基线；新增 IR/Resolver 类型按本 change 规划接入，若需要改变契约则通过 OpenSpec 显式修订。先在历史 45–52 fixtures 上验证 A09/A10/A11/A13/A14/A16/A17，再允许 P3 消费 CanonicalCFG/SSA。
