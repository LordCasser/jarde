## Context

见 proposal.md。基线与四原件真实 IR 已封闭；trace SHA 为 cddd66ab28b5c4e16536bc5315bee53e32f7c43f47a91f115db16dfcc4888e74。Loop5 的自然集合 {14,23,39,88}，退出 14→106、39→46。唯一 named row ordinal0/[0,123)/handler124/typeCP54；block106 为 [106,124)，内部表达式到122，return123 在范围外。当前 region_at 先在 loop header14 返回 Straight0；try_level 的局部试探只接受 prefix+完整Loop+范围末尾goto，故真实结果只保留 Try(Straight0, catch124)，不是 guard/catches 根本识别不到原行。

## Goals / Non-Goals

**Goals:** 严格组合已有普通 named catch、完整循环和精确指令范围返回尾，复用既有 owner、Frame 和 writer。

**Non-Goals:** 不把 return 填入 normal_exit_bci，不改变 certified_try_exit 的单条 goto 契约，不普遍放宽 ordinary_try_list_scope 的块跨度检查，不引入部分块 owner 模型。这里只有单条物理命名行、一个完整循环、一个末尾 straight block；finally/retry/monitor/fragmented/nested try 不准入。

## Decisions

1. 在现有 prefix-loop 局部候选内复用 immutable named_catch_rows 与完整状态 checkpoint，作为旧 goto 证书旁的独立 return-tail 分支。仅先得到恰一个完整 Loop，再走唯一正常 exit 的单块 tail；返回 Try.body=Sequence(prefix,loop,tail)，normal_exit_bci=None。不把 break-side DAG 当作 tail，不容许未恢复 loop 发布组合。
2. 复用 recover-proved-loop-break-branch-bodies 的普通图闭合与 join/owner 证明，只另加当前普通命名行的精确证据。进入该证明前要求 parent frame.named_catch_rows/own_try 和 physical/canonical row 逐项匹配；全部外侧 owners 的每条非 normal 边均须由此行精确解释；Call/Return 子程序边拒绝。对实际不可抛异常的边，也不能用“死边”掩盖原行漂移或 foreign row。禁止无保护上下文通过此入口借用别处 catch。
   recover 的 excluded_edge_nodes 包含所有异常边和子程序调用边的两端，实际包括60/81；它不是未解释异常边的精确集合。外侧体只能在当前 named row 已逐边解释该 owner、且完整物理范围及双向边证明最终成立时越过此排除门；无当前行或存在 foreign/Call/Return 边仍拒绝。直接 Reader 测试须按生产入口构造此集合，不能用空集合代替真实状态。
   Loop5 的 latch88 同时含 append/charAt 和计数更新，正常入边来自23/39；既有 for_header_candidate 因效果块的多入口而拒绝提取，这个约束保持不变。此例恢复 While，continue_target仍为header14；另证明stay88是唯一真实latch、正常后继恰为header14，将其完整效果保留在body中。join只绑定同一break-body证书的branch39，以及后继恰为{branch39,stay88}且branch39独占前驱为自身的直接自然guard23；两者的局部join为88。不引入全局祖先join或另一个DAG证明，不借此放宽任意while/for提取。
3. 返回尾仅一个未占用、无 normal 后继的 canonical block，入口只来自完整 child loop owners。枚举每条物理/SSA 指令，核对 opcode/width/操作、单一路径与精确行：前缀及 loop owners 在 [start,end) 内（lead 单独校验）；tail 中全部非末尾指令在范围内，唯一末尾指令为 bci=end 的 Return，且 block_end=end+width，不能借任意重叠块准入范围外效果。物理/规范化行 ordinal/start/end/type/handler/handler参数绑定，禁止额外行。catch handler 仍独立拥有。
4. 对所有 body owners 核对唯一性、精确 visited 增量、双向正常边闭合及 takeable 异常边归属，入口、尾、handler 不得共享占用；拒绝无解释 owner。复用当前 candidate_blocks_have_named_catch_scope 只作为必要条件，另补指令范围、外侧体和返回边界证明。最终 recovered 全图仍须无 overlap/uncovered；无法通过则原样回滚。
   saved_visited已含prefix，saved与body owners交集必须恰为prefix；新增visited增量只等于完整loop及tail。canonical row.protected是异常边源集合（实际0/14/23/60/81/88/106），不是全部物理保护范围块；两者分别核验，不能要求不抛异常的39/46/53/67/74也具有异常边。
5. 每次扫描/复制/扩域前 charge，遍历和发布前 poll；复制保存 visited/unclosed_tail/catch_joins/loop_catch_terminal_owners/fragmented/own_monitor/depth，失败和 Stop 都恢复，预算不退款。证明结果尽量局部不可变，不引入额外 walker 状态。
6. root 已检查 build::try_region：普通 Try 的 arm 递归渲染 Sequence，不设置 shared-finally 的 body_span，返回表达式可以保持在 Try 内；不能把 physical row 扩大作为实现手段。ordinary_try_list_scope 是独立完整 List census，继续对 block_end>end 拒绝；本 StringBuffer 场景不因此获得 List census 能力。若 writer 存在真实局部障碍，只按当前证明补最小组合，不放宽全局范围语义。
7. Java return 操作本身不能抛异常；仅当其之前的全部求值仍在原行内时，Java try 中的 return expression 才与原件等价。区分 Reader parse、方言、Runtime、verifier 和源码恢复，既不改原异常表，也不把重编成功当作所有语义已验证。使用 Rust 标准库与现有 runtime，无新依赖；JADX 只作为算法与对照参考，不复制其可变 CFG 实现或源码，故无新增许可证义务。
8. 完整公开 API 证明区域成功之后仍存在 Builder 类型缺口：物理槽2先写double，再在handler124绑定Exception。已冻结IR中astore124读取Value10，后者是handler124自身Stack0 phi；guard::handler_binding已明确接受这种绑定，但decide_types仅跳过直接Caught，遂误报primitive/reference冲突并连带拒绝返回表达式类型。
   复用Declarations已经构造的RegionPaths.catch_parameters，把精确catch路径、handler ID、参数槽及入口Store与同一SSA指令的读写绑定起来；仅对该handler自身Stack0入口phi承载的catch头部声明补足既有排除规则。普通局部写入、其它块/槽的phi、foreign路径或无法精确关联者仍参与原兼容性检查。原directCaught契约、whole槽命名、resource/catch声明、catch_parameter_stays_in_clause逃逸检查保持不变。不得增加Double/Reference的一般重类型拆分，不以LVT或强制转换代替证据；所有新增查找有前置计费及停止传播。

## Risks / Trade-offs

- 保护范围按块误扩 → 检查每条指令、只准入恰边界的唯一 return，负例把 end 移到调用前须拒绝。
- 未完成 loop 被包进 Try → 精确单 Loop、owner 多重集和 visited delta 必须先成立。
- 行类型/handler 漂移仍看似可运行 → 静态精确行证据与行为对照分开验收。
- StringBuffer 声明或构造未正确跨 loop 表达 → 完整公开 API、整类重编和逐项 VM，若出现真实额外缺口及时记录并修正本片设计，不裁剪方法或抹掉诊断。
- 将有前置效果的latch提为for更新，或把guard23写为空臂 → 保留既有for拒绝，精确证书join与append次数由完整源码及12例追加oracle验证。

## Migration Plan

Luna 可并行准备本片私有 patch；root 集成顺序为 Loop4 闭合证明后再本片精确命名行扩展。本机Rust 构建串行，余量至少5 GiB、target上限1 GiB、每秒守卫且最终精确清理；运行期间禁止其它共享修改/Git mutation/网络/人工工具链。冻结最终 CLI 后为两原件各重做完整三方矩阵，独立 raw 审核、全工作区及新产品自身 CI 通过才能更新完成状态；CF09/71/612 分母不变。
