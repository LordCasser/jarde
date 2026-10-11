## Context

见 proposal 的动机。冻结 CF07 类 SHA 为 `004e99baeb062861f95a4dac13e3a78399a66efd31f2589b0028cfe50432ae33`。bafb CLI 当前 lastIndexOf 仍输出 `local4 = arg3 - 1; while (...)`；双 JDK 原 class 与完整源码运行一致。既有 return-arm 来源片已经保留 goto@25，不应以来源完整冒称 for 写法完成。

当前 `region.rs::for_header_candidate` 用 SSA 证明唯一更新、preheader、header phi 和局部不变量，但要求最后两指令为 Push(Int)+Store，并拒绝自然环/preheader 外所有 induction-slot 访问。`header_tested_loop` 之后才计算 `loop_terminal_returns`。`implicit_tail_latch_origin` 的末尾无 join return/latch If 仅允许 for_header=None。Builder 已按 init_bci 抽取原有 Declare/Assign，要求其后只有空声明，独立构造唯一更新并核对同一局部名。

## Goals / Non-Goals

**Goals:** 通过既有事实与语句消费扩展安全计算初值；让同一返回叶证书同时服务局部读取准入和正文 scope；保留 ForHeader 下的完整回跳来源。

**Non-Goals:** 不增加公开 IR/Frame/pass，不建立通用表达式 walker 或新的 purity 表，不改变自然环集合/所有权，不允许任意循环外读写，不修复其他异常或循环形态，不为对照算法导入外部依赖。

## Decisions

1. **表达式由原 Builder 恢复。** preheader 仍为唯一外部正常前驱、正常后继仅为 header。末尾仍是准确 Store(slot)，其唯一 stack input 必须由同一 preheader 的 Instruction 定义，其全部 uses 仅为该 Store 的准确 block/BCI instruction use，且 Store 的 stack reads 只有此值；此栈值与 Store 写入的 init_value 是不同值。另保留 init_value/update_value 的唯一 header phi 消费、header 输入、update 和 invariant 检查，不将 phi operand 误当栈输入的额外消费。不限制为 arg-1 模板，不在 region 层重建表达式。Builder 继续复用已产生的原初始化语句，不能移动其他语句、重复 producer 或改名字；须由真实 IR/API 诊断确认这一输入通过现有消费路径。
2. **先算一次已有返回集合。** 在准确 frame/exit_node 下提前调用 loop_terminal_returns，传给 for proof 的现有 SSA 访问扫描，并复用同一集合建立 body frame。只允许这些 leaf 中的 induction-slot 读取；外部写和其他读取仍拒绝。不得将 leaf 加入自然环，或另造并行证书/重新扫描集合。保持已有计费、poll 与 Stop 传播；实际费用和停止位置如有变化，以真实测试核验记录。
3. **来源仍证明准确末尾形状。** 对最终无 join If 的两条单块 Straight arm，允许同一 terminal_returns 集合中的确切 return leaf 与准确 proof.update_block 配对；由当前 caller 传递该已算集合，不能用语法 return 或更宽 frame 重新推断。复用现有自然环、唯一回边、完整 canonical 出边与 return 约束。ForHeader 更新与最后 transfer 必须准确匹配既有 proof，只有该 goto 可加入 gateway_origins，并映射完整 For span。更新只能在 header 出现一次；settled 后的空 arm 是否保留由原呈现机制决定，不为删除空 else 新增改写。原 Straight/while 路径不放宽，不递归搜索其他尾部，不复制任何 Region。
4. **效果和词法安全由既有语句机制负责。** 初始化保持同一个语句位置和上下文，其后只能为空声明。prepare_deferred_bindings、consumer-count、独立边界和 render_value 负责在原 producer 位置物化或拒绝。`f()I; g()V; store` 是必要的对抗例，不能变成先 g 再 f；共享 effect 不能重复。真实 Guard/handler 或声明作用域无法保持时，保留可靠旧呈现而不强制新的 header。若现有证明不能保证这一点，先记录独立债务并修订设计，不顺手增加新机制。
5. **对照与许可。** 参考本地 JADX TestLoopCondition5 和 LoopRegionVisitor.checkForIndexedLoop 的 init/phi/update 使用判据；本仓已有 SSA、AST、effect 物化和来源证书足够承担当前设计，不移植 Java 代码。若将来实际复制外部实现，另审许可证与归属。原 class 是行为真值，JADX 的活动 for/return 断言是呈现目标，解析、恢复、来源和运行验收分别记录。

## Risks / Trade-offs

- 仅放宽初值不能覆盖返回叶 → 三个约束同一窄片闭合，先诊断精确 leaf/frame，再实施。
- for 消费 update 后空 else 或来源变化 → 不改变返回/更新顺序；核完整正文、所有物理 BCI 和 UTF8 span，不以片段 contains 代替整类验收。
- 初值潜藏独立副作用/共享值/handler 边界 → 用真实可验证 class 核 materialization 或准确拒绝，不预设通过。
- 对现有 array-for-each 等共享消费者产生影响 → 复跑真实 ForBoundaries、loop transfers、array foreach/arraylength 与类型/声明边界，保持既有拒绝语义。
- 预算顺序变化 → 同一证明只计算一次，新增工作不得无计费；记录真实 Stop dimension/location/usage 与原子产物。

首次效果验收限定三个实际控制：pending-stack 独立 g 调用的 f/g 顺序及各自抛出、共享 f 值的一次求值、try/Guard 初始化与循环上下文边界。未证明的新 for 形式不要求扩大准入，但必须与原字节行为一致或准确拒绝；observedAfter/externalWrite 必须保持拒绝新的 for。task1.2 先冻结实际 class/方法/预期结果，再实现2.x；预测结果不当运行结果。

## 真实基线补充（2026-10-11）

同一次冻结 lastIndexOf 的真实 reader/SSA/CFG/Builder 诊断已实际通过。Store@3 的原语句是空 Declare(local4) 后紧邻 Assign(local4, Subtract(arg3@0,1@1)@2)@3；当前 Builder 用 rposition 只选有值 Declare/Assign，所以会选准确 Assign，末尾没有其它有效语句，现有消费机制可复用。该纯初值的 deferred/plans/bindings/rejected/refused 均为空；此结果不代表效果 producer 已获新 for 证明。

新控制的完整 JADX 1.5.6 实际对照发现，普通 javac 的 f()-1;store;g();while 被呈现成 g();for(f()-1;...)，四条正常/异常行在双 JDK 均与原字节不同；pending-stack 调整后的 class 及 shared 控制则须分别记录实际结果。Jarde 当前复杂控制整类因 find/shared/scoped fallback 编译失败，无运行成功声明；无 handler 的 limit()-1 正例仍为 structured while，完整 Jarde 重编运行四条结果同原。新增 for 必须保持现有初始化语句后仅空声明的门，不能照搬这个 JADX 提升位置。JADX 默认包改为 defpackage 导致第一轮 Runner 包绑定失败，是 harness 失败；第二轮仅给同一 Runner 正文前置准确 package 声明，JADX 正文原样保留。

## Migration Plan

生产应用等待当前 stack 产品自身 CI 与 clean 主线关闭，并在排队 CF12 gate 的 root 串行集成之后。Luna 可先在私有目录准备诊断和实现草案，root 独占工具链/Git。使用用户批准的 5 GiB free/target 1 GiB、一秒进程组守卫，保留冻结 CLI/raw，完成后 clean。不新增 worktree/功能分支；仅实际产品、对照、strict 和自身 CI 验收后关闭任务。任何失败保留，不借先前版本结果。
