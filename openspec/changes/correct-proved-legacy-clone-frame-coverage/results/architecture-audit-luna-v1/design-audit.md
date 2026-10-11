# Legacy clone coverage 架构审计

## 结论

失败来自 canonical graph postcondition 对每个 clone node 单独查找直接入边：必须由匹配的 Call 直接进入，或由匹配的嵌套 Return 直接回到 continuation。它没有验证 clone node 是否可从所属 frame 的入口沿同路径控制流到达。因此合法 clone (14, [0]) 被误报 orphan。应验证 frame 感知的可达覆盖闭包，不能只因某个路径有 Call 就放宽。

## 代码与失败证据

- CloneState::drain 位于 crates/jarde-jvm/src/canonical.rs。Normal 和 Exception 转移都复制当前 id.path；Exception 保留 handler ordinal，不会 pop call path。Call 将 call_site push 到 path；Return 校验 payload 中与当前末尾 call_site 匹配的 row/target，再将 path pop 一层到该 continuation。
- CanonicalCfg::postcondition 当前逐 clone 检查直接 Call 或直接 Return 入边。真实失败诊断为 clone CanonicalBlockId { bci: 14, path: [0] } is orphaned。真实 raw 的大诊断未在报告中展开。
- build 先从方法入口 (first_bci, []) drain。之后为尚未由 Call edge 进入的 context 建死代码 seed，但仅当 call-site block 在 cfg.unreachable 中；raw-reachable 而 walk 未进入的 callsite 会被拒绝，避免错误地挂到 top-level path。
- assemble 已建立 edge 邻接并从真正方法 root 计算 unreachable。此 root reachability 和 clone frame coverage 是两种不同事实：死 callsite seed 打开的区域保留在 graph，但仍属于 unreachable。

## 最小可靠修正

在 assemble 增加 clone coverage 遍历，覆盖结果用于验证，不改变既有 root reachability/unreachable 计算。覆盖根取所有现存 path=[] IDs，以保留入口和 build 已验证的显式死 callsite seeds。遍历只能走满足 frame 关系的实际 canonical edges：

- Normal/Exception：source.path 与 target.path 相同；
- Call(site)：target.path 等于 source.path 加 site；
- Return(site)：source.path 等于 target.path 加 site。

要求每个 clone ID 都进入覆盖闭包。嵌套 Return source 必须先被覆盖，Return 才能覆盖外层 continuation；之后普通边可继续覆盖 continuation 后的 clone blocks。纯自循环或断开的 clone 分量没有覆盖根，仍被拒绝。错 path 的普通/异常入边不能支持覆盖；错误嵌套层级或 call-site 的 Return 也不符合精确路径关系。

对给定方法，Call 0 进入的 [0] frame 可以沿其内部 Normal/Exception edges 到达 (14,[0])。Call、比较分支以及唯一 ret 35 的实际 return target 仍由已有 traversal/payload 负责；覆盖 walk 不重新推导 return points。jsr 0/3→8、比较位于 11/15/22/26、ret 35 的真实 major 49 class 已由用户确认通过双 JDK -Xverify:all。

全部 path=[] 节点作为覆盖起点之所以安全，是因为构造器仅从方法入口、受 cfg.unreachable 约束的 dead callsite seed，或已 drain 的 ret continuation 创建 path=[] 节点；后两种语义都不应让覆盖验证改写原有 unreachable truth。顶层 Call 入口可以打开 clone；嵌套 Call source 则必须先由覆盖 walk 覆盖。若 postcondition 也需要独立验证任意手工构造图，这条构造器前提应显式传入 seed IDs；但当前 postcondition 是 build 的内部发布后置条件，最小修复可在 assemble 消费构造结果验证。

## 预算

不要把新遍历藏进当前没有 Budget 参数的 postcondition。把 coverage walk 放在 assemble 的带预算流程内；复用现有 edge adjacency，邻接项改存 edge refs，或复用已有分组，避免复制 CanonicalBlockId.path。对覆盖节点的 worklist pop 和检查的 edge 收 AnalysisSteps。新增的 seen/worklist/index 槽位在分配/插入前按节点或边数量预扣；优先复用既有计费项，避免再创建独立预算维度。raw_transfers 对每个中间 successor 项已有 IrItems 计费，clone node 与 origin 在 create 前也已计费。当前 assemble 的 root walk 已按 pop 和 edge 检查收 AnalysisSteps。覆盖遍历不应参与 unreachable 的计算。

## 历史测试约束

canonical.rs 的 historical finally 测试覆盖 classfile major 45–48，要求共享 subroutine 的 [5] 和 [12] clone 分开，并让 ret 各回自己的 continuation；handler-path call 的 dead seed clone 仍保留且属于 unreachable。nested_calls_keep_their_call_path 与 shared-routine nested-call 测试要求 [0,5]、[0,8]、[3,8] 路径分离，Return 仅回匹配 continuation。a_context_the_walk_never_entered_inside_a_reachable_block_is_refused 固定了 reachable-but-unentered callsite 不得猜 frame。

以上是只读架构审计；没有执行 Git/Cargo/JDK/CLI，也没有修改仓库。
