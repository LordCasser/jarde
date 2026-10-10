## Context

动机见 proposal。root 已实际运行并独立接受 `openspec/evidence/java-syntax-2026-10-11/cf12-real-ir-diagnostics-root-v1/`：完整 1563B class，SHA5347758d128465aa1889769ac2a18448f060682a7f6567b27f8d98e3499fc034，test(IZZ)Ljava/lang/String;。canonical 为11块/17条 Normal 边，无不可达、Return/Exception/Call，path 均空。dispatch0→32/117/146/149；case1 为32→59/117、59→63/117、63→67/92、67/92→171；case2 为117→121/171、121→171；146/149→171。当前真实输出为117 Loop、SwitchArmsOverlap 和 UncoveredBlocks。

现有 switch_region 已证明 join171、验证 decoded targets、共享 target 分组并按已证 fallthrough 排序；switch_fallthroughs 只接受单 successor。非空 map 才让 Frame::switch_arm 将其他 case 作为 stop 边界。NormalFlowView 保留 Normal 和 canonical Return，隐去 Exception/Call，故仅看 view 不能证明边闭合。

## Goals / Non-Goals

**Goals:** 扩展既有局部 fallthrough 证书，证明32的结果为{117,171}、其余三个入口只到171，复用现有 case boundary 和 break 呈现。

**Non-Goals:** 不扩 JVM IR/SSA/frame 类型域、不加通用图服务或 source body 副本，不改变 natural loop/异常恢复，不改类型或常量名。此片不追求非相邻标签的重新排列，也不代表整 CF12 通过。

## Decisions

1. **有限图探测先于正文 walk。** 在既有 switch_fallthroughs 内用标准容器和有限 DAG 分类；显式工作栈或已有受控遍历避免新无界递归。已证 join 与其他 case 是叶边界，不能占有它们的正文；return/throw 仅在 terminal_bci/Operation 与完整零 outgoing 证据都成立时作为独立叶。合并全部路径结果，至多一个 case target，且必须是现有 BCI 排序下相邻 group。循环、未知叶、多目标或不相邻拒绝；全部仅到 join/准确终止则 no-fallthrough。保持 Some(empty) 与 None 的含义。
2. **完整边而非投影作证。** 用 canonical identity（BCI/path）核 incoming/outgoing；任何触及候选正文的非 Normal 边拒绝，包括 canonical Return，不能只用 excluded_edge_nodes。按多重集合核完整 Normal 行与 view 邻接，重复边被折叠不能当成唯一边。正文 interior 所有 incoming 必须来自本 arm；case entry incoming 只允许 dispatch、本 case 或已证进入它的前 case。全组集合核证后发布 map，不能以任意 case predecessor 代替已证路径。case/join 为边界，边界后的继续代码不属于前 arm 的证明。
3. **必要的 switch-break 证据叶。** root/Luna消费审查确认：Frame::switch_arm 可停止117的claim，但 group-level fall_through 只能控制整组末尾自动break；Region::If不承载每支的终止目标。单发32→117 map会让到171的路径错误执行case2。Builder重新扫描CFG并修生成后的AST会重复证明，易丢失嵌套作用域；因此增加一个必要的 Region::SwitchBreak 叶，以准确 source_bci、所属 switch branch BCI 与 canonical join 身份承载路径退出。复用现有 Frame switch boundary、If树和AST StmtKind::Break，Emitter不新增语法。它不是LoopBreak；不能伪装loop目标。

   在region已有完整边证明且走到该switch join的准确分支末尾加入该叶，记录实际transfer/branch来源；到其他case保持fallthrough边界，join不归属case正文。SwitchBreak不占有目标join块；完整正文原块仍只归属一次。私有Frame owner上下文与Builder当前switch目标栈只承载该既有控制作用域，必须传播/恢复准确，checkpoint同样还原。Builder仅当叶目标是当前最近breakable switch且没有跨越内层loop/switch时消费为无label Break；跨作用域需要label的情形保守拒绝，不给AST Switch加label机制。全组 graph certificate 和逐路径退出证据一起准入，不能用reaches(join)替代准确路径出口。

   既有group排序与fall_through仍决定末尾能否自然进入下一标签；新Break使部分路径明确退出。所有Region exhaustive访问的blocks/structure/fallback/rule/report/type-use必须核完，叶必须保留source_bci而不能当空代码抹除。真实永久测试核117恰在case2一次、171不在arm、无假Loop、67/92到171的路径有准确break来源，不能只测map便声称恢复成功。
4. **实际工作计费。** 每次实际节点/边遍历 charge AnalysisSteps 并 poll，使用原 Budget/Stop；准备邻接扫描同样收费。不能以缓存名义隐藏首次全图工作，不能用无计费的逐节点全图搜索。证明不写 visited，Stop 通过原 recovery/Report 原子发布路径传播。
5. **已有实现与依赖。** 本地 JADX SwitchRegionMaker 的 dominance-frontier 与独立排序检查提供算法参考，其源历史 SHA741133377c6d9ec0f60c40d234f01e3dbd79f2bd80e4a2ed15ca105f0a408bd1/revision2fb1b16386941660fda07e9017285aec40fcb37f 已记录于基线；这里在现有 Rust identity/ownership 上独立实现，不复制数据模型或引入库。标准库、现有 Operations/CFG/Budget 提供证明事实；上述单一语义叶补齐了现有结构缺失的路径退出信息，不引入其他实体。
6. **验收与交付。** 原五fixture/六方法 default/all 保留 complete class/check/Inner/SDK，新的条件锚完整编译运行同原及JADX、全物理BCI来源；其他fixture按所选源基线保留正文/map/失败分类，不能借类型片或旧CLI修复计数。新增真实来源正例、非 Normal/clone/external incoming/cycle/multiple/nonadjacent/unknown-terminal 边界、budget/cancel；拒绝测试须明确哪条检查实际触发。

## Risks / Trade-offs

- [看似闭合的 view 隐藏边] → full canonical incident edge 与 incoming 证明；实际样本无非 Normal，必须另设真实边界。
- [case2 被前 case 消费] → 在probe与walk两层核边界和exactly-once，整类原样运行与map验收。
- [过宽 terminal 或 BCI 排序] → 精确 decoded terminal/零 outgoing，保留相邻限制。
- [额外遍历和栈溢出] → 所有实际工作计费、poll，有限工作栈；不新增猜测深度阈值。

## Migration Plan

先核冻结基线、真实诊断和周边架构；Luna只准备 private patch，root审查后在前片干净关闭后顺序应用。root独占工具链，用5GiB free/target1GiB一秒进程组守卫，完整对照（包含混合break/fallthrough和最近作用域反例）、冻结CLI/source pins、产品提交与自身CI独立验收后清理并更新账本/handoff。保留失败原稿/raw，不接受后续文档CI冒充产品CI。
