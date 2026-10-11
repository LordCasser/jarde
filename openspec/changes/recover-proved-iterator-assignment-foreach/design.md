## Context

动机见 proposal。基底 Builder SHA256 `b3b9f75205add71234fd6f1854312e4b74ffafb5bda9e44a0c8fd3399259692c`；固定 class 为 `cf11/UpstreamNestedLoops.test(Ljava/util/List;Ljava/util/List;)V`，1213B，SHA256 `8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231`。

root 受控同 IR 诊断 v2 实际执行 test 和 clean 两命令，公开 API 三请求均通过原表示断言。外层 test BCI13/path[1] 的首句 AST 是 `s1 = (String) local3.next()`：next17 的 Value32 进入 CheckCast22，Value33 进入 Store25/local4、写 Value34。三组均到 `proof_complete_diagnostic_only`，在名称分配、改写和删除前强制返回 None。日志没有发布元素 LocalVariable 身份；slot4/name s1 不用来推定身份。全部 4732 输入恢复，target 峰值327621731B，已 clean。receipt SHA256 `581ee2a06b144ef9ba3cc622bc213344d58b25312aa8d898fcf83411488ef811`；独立 accept-v3 实际 exit0，证据见 results/outer-assignment-diagnostic-root-v1。

## Goals / Non-Goals

只扩大既有 iterable candidate 的绑定 AST 准入和最终 RHS 可变访问；原 Assign 节点留在原 body。无 cast 的 Assign 和 separate Cast(Local) 的 Assign 不纳入本片。原 Declare 的两种路径保持。不得顺带改变 iterator helpers、SSA、reuse、声明计划、类型决策或 literal gap 策略。

## Decisions

1. 复用已构建的 Assign。`write_target`/`write_statement`/`assignment` 已按真实 LocalVariable、声明与类型决定构造该节点，本次不移动或删去目标声明，不重新选择赋值对象，因此不必新增元素身份服务。新增 Object 合成元素仍用现有名称表与 collision/budget 逻辑。
2. 首句 Declare 沿旧路；首句 Assign 仅接受 `Cast(Call)`，之后仍核精确 Iterator.next 接口 descriptor、receiver、next/cast/store BCI 与 SSA 唯一消费者。不得借打印名称绕过这些门。只把该 next 叶子替换为合成 Object local，保留 cast/Assign 的 origin、type、目标和全部其他语句。最终 RHS 访问与入口保持相同准入范围，避免完整证明成功后却不能写回。
3. iterator 完整证书原样消费：初始化/hasNext/next LocalVariable、可见声明 owner、准确列表删除索引、header Phi、use occurrence、gap、receiver、effect/handler 均不放宽。沿用 Object foreach，cast 在 body 原位置执行，原 target 的读写生命周期保留。
4. 新形状检查与 RHS 访问是常数工作，沿现有 candidate 预算/poll和Stop传播；若新增扫描或证书工作必须明确计费。拒绝时不分配名称、不编辑 body 或列表；成功只修改克隆 body，caller 仍在证明后倒序删除准确 iterator 行。不增加通用回滚机制。真实 late Stop 和来源验收由公开 API 重新测量，旧片的 Stop 用量不能直接当新片证据。
5. 参考本地 JADX `LoopRegionVisitor.checkIterableForEach` wrapped CHECK_CAST 的识别和唯一消费思路；其 `fixIterableType` 会修改寄存器泛型，不能作为本仓 source proof。现有 AST/NameTable/SSA/effects 已足够；无新依赖、代码复制或许可负担。raw List 和泛型方法证明继续独立。

## Risks / Trade-offs

- 改成 typed header 会删 cast 或缩小 target 作用域 → 保留 Object header 和原 Assign；完整类测空列表、最后值及错误元素时写入状态。
- 额外 next/consumer、body 前置副作用或不同异常范围误通过 → 保留全部原门，必要真实反例分别定位实际拒绝，不用编译失败冒充运行通过。
- 来源或 optional evidence 改变决策 → 固定公开 API 三请求逐物理 BCI 核验，next17/cast22/store25及外层回边保留；完整类源码不得手工修补。
- 修复引发旧输出/预算 golden 变化 → 先逐类确认真实行为，再只修准确预期；原始失败保留，不混入泛型或其它语法改动。

## Migration Plan

先接受诊断和控制基线，Luna 私有最小 candidate/root 对抗审阅；前片自身 CI 与干净 main 关闭后才应用。root 串行构建，5GiB最低余量/target1GiB一秒进程组守卫、默认 Rust 栈、finally clean。完成永久回归、完整类双 JDK 原/JADX/Jarde、fresh工作区、lint/strict、确切产品自身 CI 后交付。只关闭该窄片，不关闭 CF-11。
