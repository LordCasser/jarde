## Context

动机见 proposal.md。上一来源片的新CF07独立接受119 inventory members/29命令/10完整类腿；counted(II)I四profile仅缺物理goto20→27，lastIndexOf25→5独立缺口保持。准确字节、javap与Runner原raw都已冻结，不改变原oracle。

现有Region::If保存join和独立两arm。Builder::region构造完整StmtKind::If的OriginSet，但只处理空arm单指令Transfer；普通非空arm末尾goto没有相应Java语句。Emitter直接记录If节点完整范围，不需要新增映射服务。Builder::arm在传播Stop前恢复外层stmts/declared，最终成功push If才提交该语句；无需通用事务或回滚实体。

## Goals / Non-Goals

**Goals:** 精确补齐现有已恢复If的非空arm末尾汇合来源；保持正文、已有物理OriginSet和外循环来源。证明只使用已存在canonical、SSA、operations、Region与预算。

**Non-Goals:** 不修改If join选择算法，不恢复新结构、不改变conditional-value folded/refused路径、不扩ForHeader，不修lastIndexOf嵌套else latch，不新增Region字段/IR/Frame/pass/依赖/CLI。不把覆盖集合的差集直接补到任意source span。

## Decisions

1. 实施前诊断实际嵌套Region与canonical边。all-evidence现只列顶层loop的blocks(6/11/17/23/27)，不能证明其内部If join=27与then尾部形状。动态前提未满足时保留未完成任务，而非建立新机制去强行匹配。
2. 在既有普通If的origin构造处读取join，复用OriginSet.plus_derived。只检查本arm末尾Straight（如有Sequence，只看最后直接child），末尾SSA必须真实goto/goto_w且Operation::Transfer；完整canonical outgoing准确唯一Normal边到该If join，block由该arm唯一持有，与另一arm/外层owner不重叠。错误target、缺join、return/throw/conditional、多边/异常/非末尾均拒绝该来源。保留现有空arm行为，不顺便扩展其它边类型。
3. 来源归属完整If语句，不能使用Loop.gateway_origins；后者派生到完整LoopStmt，会错认@20为外while回跳。已有Switch尾部goto到join的局部位置可参考，不能直接复制any Normal edge作为新充分证明。JADX IfRegionMaker.findOutBlock的dom frontier/path-cross提供join选择参考，当前没有替换本仓join算法的需要。
4. 新证明扫描按现有budget先charge再poll，Stop原样传播且不发布部分正文/来源。root独占Git/工具链与生产应用，Luna私有最小patch，root读审、真实反例和新CLI完整类验收。无需外部库或SDK：标准Rust集合、既有Runtime和Emitter已满足本目标。
5. 机器free≥5368709120 bytes、本仓target≤1073741824 bytes，1秒守卫中止进程组，完成cargo clean本仓并保留冻结CLI/源码/class/raw。前片d714a6bcc精确自身CI接受之前不应用本片生产补丁；准备规划与诊断不替代自身产品CI。

## Risks / Trade-offs

- [把内层transfer归到外loop或then赋值] → 精确If join、全canonical边和唯一arm owner，永久测试核完整If范围及外while@30原有来源。
- [反向搜到非末尾或nested区域内部goto] → 仅最终直接Straight，拒绝跨nested If/loop/terminal扫描。
- [只比manifest而不验证真实字节] → 重解析固定双JDK raw javap、物理owner/descriptor/全部BCI，原样完整源码重编运行同原raw。
- [预算/取消后仅少个origin却照常发布] → Result Stop传播，实际预算停止测试确认NotProduced和零正文/来源。

## Migration Plan

先接受前片确切CI与本片基线、完成有界Region诊断，再应用已审私有patch并独立构建新CLI。双JDK/default-all完整对照只允许Runner包名适配，来源验收要求counted所有BCI完整，同时lastIndexOf25缺口保持单列。提交推送新产品并验收其精确自身CI后，更新账本/handoff及主线清理审计。71/612及CF07整单元计数不自动增加。
