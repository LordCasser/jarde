## Context

动机见 proposal.md。`results/same-run-ir-root-v1/` 已真实执行 debug/no-debug 原件：normal0→34、handler9→17/19、17终止throw、19→0；normal view 无自然 loop，唯一异常行保护 [0,6)、handler9，Catches.join34。实际首拒绝为 `retry_loop_facts` 的单 handler successor 条件；普通 catch walker 随后沿19回0，再次构造 Try，重复0/9。

现有 `RetryLoopFacts` 已为这种“仅通过异常边闭环”的重试建立 Frame 并输出 Endless Loop。`try_level` 与 `retry_owned_regions_complete` 仍假定 catch 只有一个物理块，且在 catch 尾补一个 continue。必须共同修正，不能只放松首拒绝。

## Goals / Non-Goals

**Goals:** 保留一个物理保护块和唯一具名 handler 的已证明前提，扩展到一次条件分叉、一个单块 throw 叶、一个单块 retry 叶；复用当前 IR、loop Frame、Try/If/Throw 与转移节点。

**Non-Goals:** 不探索任意 DAG、嵌套 handler、多个物理行、finally、Smali/Dex 输入、反射执行、数据流驱动消除不可达 catch；不修改自然 loop 构建、最终 overlap gate、Builder 或来源映射债务。

## Decisions

1. **在现有 proof 中分辨旧单块与受限三块 handler。** 保持全部原有保护区/物理行/join检查。条件块必须是精确解码的两条 normal successor；一个叶以实际 Throw 终止且无 normal successor，另一个叶的实际 goto 唯一回到 header。三个块互异、非 clone、非额外保护区/共享 handler，子块无外部前驱；所有涉及的 canonical 边与 throw-site 行归属均需核验。循环 owner 集为保护块与三个 handler 块，唯一正常出口为已有 join。按精确角色证明，避免引入通用图框架。
2. **复用 Frame 和作用域。** 保护区 goto→join 发射原有 break；retry 叶 goto→header 发射原有 continue；throw 叶保留原语句。handler 内 clear/sleep 不移进保护区，原异常范围不扩大。分支形状的 continue 必须在 retry 分支中，不能无条件挂在已终结的整个 If 后。旧单块路径保持现有行为。
3. **证明结果和实际树双重校验。** 精确核 body owner、catch owner 的集合及出现次数，visited 增量恰好等于候选 owner；核 break/continue 的源指令、数量和目标以及 terminal throw 叶。不能用 BTreeSet 掩盖重复出现。形状拒绝或 Stop 时恢复现有 checkpoint 与 unclosed_tail。
4. **不引入依赖或复制 JADX 管线。** 标准集合与已有 Runtime/IR 已足够。JADX 的 endless-loop 与 handler exit 边界处理可作为参考；其先循环后 try 的全管线迁移成本与当前缺口不匹配。源码仅由既有恢复器消费，生产不运行目标。读取/结构恢复、Java 语法重编、执行对照与 JVM verification 分别报告。
5. **原件与可达控制例分开验收。** 原件完整保留 outer/check 与框架边界，公开方法均需固定最终 CLI 的 default/all、debug/no-debug、JDK8/23 编译尝试和隔离 VM；原件 catch 实际不可达，不借此宣称 catch 行为已测。独立控制例用可控失败次数/重抛条件和无真实睡眠的事件记录，覆盖正常完成、重试、重抛、retry 效果自身抛异常、条件极性反转；控制例不替代原件。

## Risks / Trade-offs

- [三块集合看似正确但重复拥有] → 检查实际 owner 次数、visited 增量与最终 overlap，保留对抗性错边/外部入口/共享行测试。
- [handler 尾 continue 误挂或吞掉 throw] → 精确 transfer 与两条路径的终结检查，完整 javac 及可达控制例事件序列对照。
- [Frame 早返回使未闭合候选泄漏] → 复用 checkpoint，覆盖 used-minus-one、预取消与组装后的拒绝。
- [原件语义覆盖不足] → 明示 catch 不可达，用独立控制例补足，不能修改原件或手修生成 Java。
- [新机制范围漂移] → 若需要任意图或 Builder 规则，先记录独立债务并重新审查，不混入本片。

## Migration Plan

root 先审 Luna 私有 patch，再串行受控运行定向门、原件完整对照、控制例与一次 fresh workspace/产品自身 CI。产品验收前不勾完成或提升 CF09。失败保留原流并恢复；无需公共 API 迁移。构建保持 5 GiB 余量、target 1 GiB 上限、默认栈、每秒守卫和 finally 精确清理。
