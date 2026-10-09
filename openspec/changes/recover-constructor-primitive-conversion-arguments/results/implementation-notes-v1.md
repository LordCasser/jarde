# 实现候选记录

当前候选只接通普通 `new` 构造参数中的既有 primitive conversion 事实和这条生产路径的共享预算。

`init::verify_metered` 在已有构造参数依赖闭包扫描中，仅把属于实际实参依赖的 `PrimitiveConversion` 记为本 Site 的 conversion marker。无关 conversion 仍落入原 `StatementFree` 拒绝；Builder 的 Cast、源类别和构造 descriptor 验证没有复制到 `init`。本 marker 只为复用已有 handler ordinal 覆盖闭区间检查，不扩大普通 invoke 的 handler 规则。

report 调用的 `sites_after_array_composition` 接收同一个 `Budget` 并返回 `Result<Sites, StopReason>`。数组 pending Site 仍在进入 census 时单次 move。生产 census 每实际访问一条 SSA 指令计一次 `AnalysisSteps`，随后直接进入 `verify_metered`；内部已有扫描、SSA 读取和 handler 逐项比较仍由同一个 meter 收费。Refusal 继续成为普通 `Sites` refusal，Stop 直接返回并由 report 转成 stopped report，局部 Sites 不会传到 Region。仅供单元测试的便利入口保留显式 unmetered meter，预算断言使用生产入口。

Focused 用例覆盖两种 javac 生成的 wrapper conversion 与普通 return-new、stored-local reuse（核对 BCI 3 的唯一 producer 及两个 local-load/conversion 链）、无 conversion 的 Integer 对照、普通 census 的 budget/cancel、无关 conversion、从真实 Pair(JJ) 类派生的 `dup2` SSA 一读两不同输出拒绝，以及 handler 原覆盖/不同覆盖控制。结构断言本身不把 producer 塞进 Site.expression；一次运行由 root 的原始双流验收确认。Boolean 源类别与 cast 呈现继续由现有 Builder primitive-conversion 测试覆盖；本候选没有增加第二套 Java 类型规则。

这些测试和本候选尚未由 root 执行 Cargo、fmt、CLI 或完整双 JDK 门禁；此记录不代表 task 2.1/2.2 已验收。handler、dup2 和无关 conversion 的 Code 派生变体只在测试内分析，不执行变义字节码。完整生成源集的重编与双流结果仍由 root 验收。
