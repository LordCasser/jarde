## Context

动机见 proposal。当前85ca CLI对完整原始类呈现 `String s1; for (String iteratorElement17 : l1) { s1 = iteratorElement17; ... }`，内层首句为 `String s2 = iteratorElement47`；无调试版使用已有 local4/local6。原始/控制最终完整矩阵与20API重绑已接受，不能复用10月12日的旧 CFG 首拒。

现有 typed proof 已闭合直接 List formal、Signature E、next/checkcast/store 和首句 AST。`SameClassBodyValueSite` 已有全部 local reads/writes 及同次 value/definition/uses/replaced_by；opaque AST 完整保留声明、实际子块和表达式。缺少的是“这些物理读取所对应的源码引用全部属于该 loop binding”的关系，不是新循环机制。旧 scope_anchors 把 If 两臂及 Try 各臂放在同一容器锚点下，不能用它单独证明声明可见性。BCI25的空声明和赋值也不是同一 AST 节点。

## Goals / Non-Goals

**Goals:** 同时完成固定 outer Assign、inner Declare，以及已有 declared-only 正例；证明之后直接复用原目标名字，除精确绑定/空声明删除及头名外，其余 AST 不变。保留强类型证明和原子停止；initialized、observable-before/after 等旧控制不被折叠。

**Non-Goals:** 不建立通用名字解析、scope path 框架或活跃性分析，不扩充 SSA/phi 公有载荷，不改 Reader、区域识别、泛型元素推断或无关调用包装。仅有 Definition::Phi 而没有 incoming 闭合事实时保留可靠表示；本片不为它新增机制。

## Decisions

1. **窄绑定聚合事实，复用同次 AST。** 在 report opaque helper 内对一个已有 foreach 候选直接遍历真实 statement 子 Vec/祖先可见环境，确认唯一首句 target、唯一 synthetic 引用、原目标声明与全部引用的归属。只输出本片需要的窄 BindingSite/计划事实：loop/首句身份、synthetic/target 已有名字、可删空声明的精确选择与来源、目标 Local 读取锚点。优先复用 OriginSet/ClassSourceAstAnchor 与现有 edit，不导出通用 local occurrence 或 branch discriminator 系统。新增 doc-hidden 记录仅用于跨 crate 证书与最终重验，不进入序列化模型。
2. **在实际 AST 中证明可见性。** inner 必须是该唯一 ForEach.body 的直接首 Declare；outer 必须是直接首 Assign，并有唯一、未初始化、在实际祖先 statement block 中先于当前路径可见的空 Declare。逐子块继承可见环境，兄弟 If/Try/Switch 分支不互相泄露。扫描所有目标 Local 引用，要求都在该唯一 loop.body；synthetic 只有首 cast child 这一次读取。按节点形态和唯一性区分同BCI25的空声明/赋值，不按 BCI 范围、打印文本或压扁 scope 推断。其它同名 formal、声明、foreach/catch/resource/lambda binder 或 LocalAssign/更新写入使可选候选拒绝；未覆盖/不完整节点不能视为没有使用。
3. **物理 slot/value 闭合仍在 facade。** 利用首句唯一 store 找到原 slot 和 stored value。核全方法该 slot 的唯一写入正是该 store；全部 local reads 与 AST 目标引用锚点、load opcode、同一 slot 和 stored SSA identity 一一闭合，拒绝额外/缺失/重复读取、替换值或无法闭合的 phi。保留现有 next→cast→store 唯一消费、类型和 handler 条件，不重新跑 Reader/SSA，不以 LVT 或同名替代物理证明。
4. **可选折叠不污染 typed 前提。** 在已有 typed edit 上组合一个可选的受证绑定提升计划。缺少折叠证据只令本次 fold 为 None，不撤销 E 头/原 body binding。report 投影在改动前按同一 opaque AST 重建并核具体绑定事实；可选计划不能匹配时保留 typed 编辑。实际 Stop 必须沿原事务传播。facade staged/final 重算完整 typed+fold 计划并计费比较，不能只比名字/类型。一次 clone/原有事务内验证后再改 AST，禁止独立文本重写或新的发布通道。
5. **精确编辑与来源。** 只将 ForEach.name 换成已有 target，移除其直接首 binding；Assign 形额外删除唯一受证空声明。其余 Local 仍用原目标名字，不进行全方法重命名。将被删 binding/空声明的全部同方法 primary/derived 来源合入 header，保留已吸收 cast 的来源；全层检查 foreign owner。多层计划应在同一原始树验证，应用时不让外层删首句导致内层节点选择错位。
6. **所有新增工作先收费。** 新遍历按节点/子项提前 charge/poll，复制名字、来源 owner/CP 与计划向量前收费；比较/排序工作纳入已有预算。不可先无预算扫描或 program_node_count 后补账，也不可使用不支持中途 Stop 的旧 FnMut 遍历硬拖到整树结束。复用原有 AST clone 和提交计费；其既有预扫债务仍独立，不扩成本片重构。默认 stack，不引入深表达式递归风险。
7. **主规范显式演进，输出平面分开。** 已交付 typed requirement先同步主规范；本片以完整 MODIFIED block保留全部六个场景并增加折叠条件。投影 method/class 是允许变化的平面；RecoveryReport.text、quality/fallback 与自身 raw map 不因美化改写。default/all/no-rule-details必须得到同一投影，Stop无半个声明或名字替换。
8. **参考现成算法，不复制另一套 IR。** JADX LoopRegionVisitor的 usedOnlyInLoop/assignOnlyInLoop、复用原 CodeVar 是用途闭合与原名选择的参考；Jarde以已有物理事实和保留树完成局部证明即可，无需 REGION_ARG、DONT_GENERATE 或另一套变量系统。使用标准库和现有 Runtime/AST，无新依赖或许可变化；这里只改 source recovery，不改解析、验证、dialect/runtime选择。wrapped路径静态差异不当作已确认新bug。

## Risks / Trade-offs

- 同BCI节点或兄弟分支误配 → 实际子块可见性、节点形态/全方法唯一性，加内部 AST 变形负控；不将变形称为合法 JVM 样例。
- slot复用、额外写入/外部读取或 capture被遗漏 → 原始物理读写全量集合与目标引用闭合，不完整只跳过可选 fold；旧 initialized/observable 控制必须保持原绑定。
- optional fold拒绝误伤已有 typed 或泛型结果 → 类型证书与折叠资格分离；公共控制比较原可靠 typed/raw/mismatch，而非以退回 Object 头算成功。
- 删首句丢 origin 或部分发布 → 所有来源合入原 header、default/all/raw-map 分平面审核，真实 late预算/取消/期限及 final计划重验。
- 仅修内层或只美化一份 debug 输出 → 原始完整 debug/none、两真实编译器/双VM、outer/inner黄金片段和独立完整控制一起验收，不裁剪源码。

## Migration Plan

先冻结前片85ca完整基线与本次窄关系分析，按 tasks派 Luna 私有实现及独立公开/拒绝测试准备。root审 patch后应用，串行构建/完整原始和控制三方对照，之后 fmt/CI Clippy/MSRV、Reader/指纹/P5、两 fresh native 和最终实际产品自身CI。只调整明确受新契约影响的正向呈现断言；旧计量表/阈值/拒绝与事件oracle不改，失败原样保留。

root独占共享文件、Git、Cargo/JDK/JADX/CLI/网络，Luna只私有准备/只读。Rust至少5GiB余量、target1GiB每秒守卫、默认stack、finally exact clean。用现有 runner/collector/parser/driver复用测试能力，不能把仅工具准备当实际验收。完成后更新账本/handoff并留干净main，原名头之外的黄金差距及71/612总目标继续保留。
