## Context

动机及前置产品见 proposal。当前 Builder SHA256 为 `e3f201df7bd01b9b0691d5f7229cbfd5efd82ce5b70fdd39f0f003170fe51b39`。固定完整原始类 debug/none 的 raw 方法及类投影均有一条 `Iterator local5;`；来源为 init store@33，实际内层 foreach 主锚点是 test@42，非 next@47。前片完整对照与 [固定分析](../../evidence/java-syntax-2026-10-18/cf11-foreach-iterator-declaration-analysis-root-v1/README.md) 是基线，不冒称本片执行了新 JDK。

`declare_at` 提前发射祖先空声明。嵌套 loop、arm、switch 和 synchronized 构建时把祖先 `Vec<Stmt>` 暂存在调用栈；候选只有当前列表的删除索引。`iterator_declaration_for_loop` 已验证 `reuse` 身份和唯一 Elevated owner/anchor，并明确区分真实同列表条目与严格祖先；不能用 owner 路径长度推定声明位置。已有数组 cache 声明清理只处理其同列表证明，不是可复用的跨列表机制。

重要边界：`LocalVariable(slot,index)` 不是 `ValueId`。既有 iterator SSA/phi 使用证明闭合一个值，受保护循环的写槽扫描仅覆盖其 loop blocks，不能据此删掉仍为循环外其他值服务的变量声明。这个区别来自当前代码，具体合法控制是否触发旧 foreach 仍待实际基线分类。

## Goals / Non-Goals

**Goals:** 完整证明变量只供一个已提交 foreach 使用时删除祖先空声明，保持 source recovery 的指令来源、语句计数与真实 Stop。保留所有旧 foreach 准入与缺证行为。

**Non-Goals:** 不重建 SSA/reuse/类型，不改声明规划，不扩大数组或 iterator 候选，不删除带值声明、元素绑定或其他无用局部，不创建公共 proof API、通用 AST 编辑/死变量 pass 或新的回滚框架。这里改变 Builder 的 raw 方法内容，不能套用前片“raw 方法逐字不变”的跨版本验收；typed 类投影仍消费它自己的 raw 方法与 map。

## Decisions

1. **只补成功消费者与祖先声明的关联。** 在现有候选完成全部准入、成功发布其 `ForEach` 到本 Builder 后，记录必要的私有 `(LocalVariable, loop_test_bci)` 关系。方法、SSA、reuse、名字和 `Declarations` 已在 Builder 内，不复制成新模型。只记录 `declaration_index` 实际缺席且有精确 Elevated owner 条目的情形；named-catch 特例虽也返回 ancestor 标志，却没有该条目，不能借此清理。重复关系必须可检测，不能用覆盖或集合去重掩盖两个候选。简单保留重数的私有向量即可；无需通用证书存储。
2. **全变量闭合是可选清理的额外门。** 用现有已解码 Operations/SSA 指令和 `reuse.variable_at` 扫描整个物理方法的局部访问。该精确变量只能有候选 init store 与两条已证明 iterator load；额外读写、其他 SSA 版本、未知/未映射的相关槽访问均保留声明。映射为另一个已区分 `LocalVariable` 的复用不误当本变量。此门不改变原候选的接纳，缺证仍生成原 foreach；真实 Stop 必须传播。不得只遍历循环 blocks、只看写值 uses 或只比较槽号/打印名。`iinc`、缺失 Operation 等无法完整说明的相关访问也不能被略过。
3. **关联随既有试构建回滚。** 新私有状态只在构建中 append，初始化与测试 Builder 构造保持一致；`FinallyCheckpoint` 记录向量长度，恢复时 truncate 原前缀，避免复制整份关系；CompoundIf 已嵌入此 checkpoint。不清空 `declared` 或借用 `settled`：前者还决定后续赋值形态，后者不是声明删除身份。子候选成功但父 body 最终被 quote/丢弃时，不得留下可应用的幽灵关系。
4. **只在最终实际 AST 上消费一次。** 所有现有整方法 fallback/closure 与构造器顺序处理结束后、`Program` 返回前，核对每个唯一关系：既有计划仍有唯一 Iterator 空声明身份/anchor，最终树只有一个主锚点为该 test BCI 的实际 `ForEach`，恰有一个同名同类型、`value: None`、直接锚点相符的声明，且声明所在真实祖先语句列表在循环之前可见。不能把 RegionPath 当 AST 列表索引，不能用 derived BCI 或同 BCI 的其他节点替代。循环来源须已完整承接该声明锚点。缺失、重复、兄弟分支、错形态或父级回滚后的记录只保留该声明，不影响其他受证项。
5. **沿现有所有权和预算完成提交。** 只处理本次未发布的 Builder AST。访问指令/槽事实、保存关系、唯一性/祖先检索及实际删除遍历都明确 charge/poll，禁止未计费预扫/整树克隆或构造期成本退款。删除一次仅把最终 statement 计数减一。先完成证据核验再删，应用遇 Stop 仍返回已有 `Err(StopReason)`，Builder 不会返回半个 `Program`；不需另外建立公共事务。拒绝可选清理不是停止，不能吞 Stop 当成普通缺证。
6. **直接使用当前能力，不引依赖。** std 集合、已有 AST/Declarations/reuse/SSA/OriginSet 足够。JADX `LoopRegionVisitor` 在变量生成前以 `DONT_GENERATE` 抑制已消费 iterator，说明应由成功循环控制临时量生命周期；本仓声明已发射，不能照搬 flags、泛型修复或注册一个全局优化 pass。参考固定源码算法，不复制代码或引入其依赖；无维护/许可负担新增。

## Risks / Trade-offs

- 同一源变量有循环外另一 SSA 写值 → 全方法精确变量访问闭合，实际 protected/catch 控制与内部事实变形分开验证；不能把不可达新门的 while 反例冒称“清理门已执行”。
- 同 BCI 两个 AST 节点或同名兄弟声明 → 唯一性、节点形态、直接 origin、真实祖先及已提交循环联合核对，不用名称替代身份。
- 父试构建回滚或 whole-body fallback → 既有 checkpoint 还原新状态，最终实际 foreach 必须仍存在。
- 删除唯一 map 段导致漏指令 → 所有被删声明锚点仍在对应 foreach 来源中；新 raw text/map 按实际文本重新生成，逐物理 BCI、UTF8 区间和精确差分验收。
- 新扫描扩大预算成本 → 只在确有祖先候选/待处理关系时工作；按实际新增输入测量成本，不放宽资源阈值。旧 AST 未计费预扫债务独立，不复制它。

## Migration Plan

先重核基线并实际构造/编译控制家族，再由 Luna 私有实现、root 对抗审查和受控应用。root 独占共享写、Git、Cargo/JDK/JADX/CLI/网络，Luna 不并行跑工具链。固定默认栈、机器至少 5 GiB 空间、target 至多 1 GiB、一秒进程组守卫和 finally exact clean。最终新 CLI 完整类三方双 JDK/双 VM、raw/map 差分、广门、两 fresh native、精确产品自身 CI、strict/main spec 同步、归档与干净 main 闭合后才交付。所有实际失败、缺证和未执行准备保留。
