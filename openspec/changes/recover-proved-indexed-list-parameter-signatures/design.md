## Context

动机与前置交付见proposal。当前final56对固定CF11输入已实际32API重放：NL2的test(List)Z结构化、raw/map闭合，但ordinary_generic_source_unproved仍保留raw List。当前source pins及只读分析保存在前片results/current-fixed-api-and-next-architecture-root-v1；本片新的完整原始/控制基线待root执行，不把旧等价leaf输入冒充新完整上游运行。

Atlas本轮定位report.rs::generic_return_candidate；incoming Focus完成并只在当前closure找到recover_inner。现有候选只为末尾return;生成VoidBody，ordinary_parameterized_declaration和另一ordinary泛型入口亦要求void；facade组件选择、staging/final只识别该variant。List闭集合同没有get(I)Object。SameClassInvokeUse已带result_value_fact，opaque AST保留同次完整Program，不缺IR。

## Goals / Non-Goals

**Goals:** 复用完整方法参数读写census与既有泛型事务，恢复与精确primitive/void结果正交的参数声明；getter读侧必须连接同次物理SSA和保留AST。E只从合法Signature选择，get/cast用于验证已有E，不创建新推断来源。

**Non-Goals:** 改写循环/返回表达式、擦除真实转换、任意List方法或通用赋值性求解。保留引用/T/array结果的既有专门证明，不能借新完整体候选扩大它们；不新增IR、pass、RecoveryRequest字段、第二AST、跨方法证书或依赖。

## Decisions

1. 将现有VoidBody的参数用途数据与结果类别在原候选中明确区分，优先把同一variant改为语义准确的完整body形式，附已存在的精确primitive/void表示，不并列复制一个PrimitiveBody及其census。结果来自本次物理descriptor，primitive的Signature结果与descriptor必须精确一致；实际return opcode按descriptor类别核准，Z/B/C/S/I共用ireturn不能替代精确源类型。完整Program/non-ragged/Code/SSA/effects/operations、参数所有写入与逐读multiplicity沿用现有核验；不从单个末尾return推断整个返回值类型。新primitive分支至少要求一个已完全闭合的精确List<ConcreteReference> formal；方法类型变量专用的原void路径仍明确保持void-only。所有消费者（包括类变量和方法泛型分支、候选转发、组件入口、stage/final）逐个审计，原有特殊返回候选保持其原准入。
2. 在既有List合同中只加入精确java/util/List、invokeinterface、get:(I)Object这一用途。每个参数读取仍必须唯一对应直接接收者加载；index实参保持原int上下文，不重写。要求get结果未替换、定义为该调用且仅流向唯一物理CHECKCAST；cast输入与结果SSA相同，opcode为c0、目标精确等于Signature选择的可拼写reference E。get结果未消费/多消费者、cast前Phi、不同cast或未知目标不接纳本条用途；remove(int)E不因返回相同擦除类型而被纳入。
3. 物理关系仍须与opaque AST的唯一Cast(E)直接子Call、method/BCI/opcode/target闭合。现有class_source_invoke_ast_sites没有父节点，invoke_result_uses遇cast只给Other(consumer_bci)，不能单靠Other或presented_type当cast证书。优先复用既有局部赋值、调用库存和遍历；如需跨opaque边界，新增最小doc-hidden布尔查询精确匹配该直接cast-child关系，而不改变所有Other消费者的分类、不新增公开记录。它只回答源形状事实，不证明E或替调用者校验SSA。所有遍历/identity/类型字符串比较与克隆先计费/poll。
4. 完整body候选、List用途集合和精确get-cast站点在既有generic-call组件中stage，最终同方法同Signature重新核验；复用已有计划/AST相等与原始emission核对，完整声明和所有正文投影一次安装。getter分支不产生cast删除编辑。保留的Cast(E)使其后的源类型上下文不变，故不以是否astore/local赋值限制get用途：直接get().length()和局部绑定使用同一证据，不另建机制。原方法RecoveryReport.text/map保持原恢复平面；完整类投影不伪造全类map。缺证保留可靠原输出，Stop经原错误转换传播而不是None。
5. 复用标准库、reader的Signature/descriptor解析、已有SSA和值事实、opaque AST及emitter；不新增库。既有JDK8/23 javap List合同采集中已含E get(int)/(I)Object，root重核原始流；JADX只参考泛型替换/调用约束的思路，不复制其可变ArgType推断pass或代码。JADX本地LICENSE为Apache-2.0；没有新第三方许可负担。

## Risks / Trade-offs

- primitive候选过早抢占旧Parameter/Conditional等专门证明 → 审计所有分支和顺序，原有引用、数组、方法类型变量与class-scope泛型回归逐项保持；primitive不由宽泛“非引用”布尔代替精确descriptor。
- 只有接收者闭合，却忽略get结果的类型影响 → Signature E、物理唯一get→cast与AST直接child三方闭合；错cast及合法(Object)中介导致的另一cast控制保持拒绝。
- 改List<E>后隐式转换改变事件 → 保留真实cast，双javac完整类与双VM用null/heap pollution/自定义List size/get事件和抛错路径实际比较，不能以编译通过替代运行验收。
- 原始/投影map混用或停止半提交 → 独立raw方法UTF8/全部BCI映射审计、default/all/essential、lateStop/取消门和既有事务回滚；usage与代码事实分开比较。

## Migration Plan

root接受已冻结的缺口证据、最小方案与strict后按本change派Luna私有实现，与完整上游/控制基线采集并行；这些实际基线未被root独立接受前，不集成补丁或运行候选构建。root独占共享源/Git/网络/工具链，Luna不运行Cargo/JDK/CLI。root审查私有补丁并串行验证focused→适用广门→最终完整源码/JDK/JADX/Jarde对照→fresh workspace与产品自身CI（两个CI seed）。默认栈、至少5GiB余量、target1GiB每秒进程组守卫，所有Rust完成exactclean。来源/CLI/完整失败日志绑定产品提交，再同步主规范、账本、handoff及干净main；本片不计CF11整项完成。
