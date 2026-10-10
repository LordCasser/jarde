## Context

动机和可观察行为见 proposal 与 delta spec。实际 CF12 原样基线保存在 `openspec/evidence/java-syntax-2026-10-11/cf12-upstream-java-root-v1/`。Java 输入上游 JUnit 六方法为5通过/1因重复block警告失败；独立原/JADX完整类重编及运行六腿全部相同，该警告未证语义错误。Jarde default/all：普通 fallthrough 和 Labels 四腿原样运行相同，char两腿有明确quote且运行不同，NoDefault和条件fallthrough各两腿编译拒绝。

build.rs 的 decide_types 从每个 LocalVariable 的第一写建立既有 Decided map；非 boolean 和专用 guard 类型之外，以 written_type/frame 定类型。value_type(Int)=Int，value_type(Null)=Reference(Object)。调用消费读取同一 decided_type，严格 invocation_argument 因此正确拒绝未证转换。charAt 的 C 描述符和后续所有 String literal 写并未成为此处的普通源类型证据。已有 SlotUse/reuse 槽位生命周期、SSA Value/Definition、Operations、Type、Budget 足够，不需要扩大 JVM frame 类型或新推断框架。

## Goals / Non-Goals

**Goals:** 在既有类型决策处增加有限、完整全写证明，兼容现有参数/boolean/guard专用证据，沿现有声明和调用呈现消费；修复真实两份完整上游 class。

**Non-Goals:** 不由 callee 所需类型或 LVT 单独推定实际值；不放宽调用 narrowing/downcast，不修改 Frame/SSA 类型域。不进行任意 copy/phi 递归或跨 class 层级推断，不恢复条件 fallthrough，不修嵌套根常量投影，不改源码发布质量标记或引入公共证书实体。本片成功不等于 CF12 完成。

## Decisions

1. **使用既有源变量身份。** 依赖 LocalVariable/reuse 与 SlotUse 的完整物理写集合，精确消费 stored operand；仅非参数普通局部候选，保留现有 descriptor/boolean/guard 优先级。先核真实两锚的同次 SSA/operations，所有写可检查且无未知生命周期才准入。LVT用于核对，不是证明种子。
2. **char 的 producer 类型与全写一致。** 直接准确 C 调用返回、C 字段、i2c 或已证 C 参数可提供种子；有限同次操作数读取复用已有工具。每个写均须同为准确 char producer 或可按现有规则赋给 char 的范围内整数 literal，并至少有准确 C 种子。不能把一般 int/arithmetic/未知 phi 或混合写认作 char。未证候选沿旧类型路线，不加任意 cast；已有转换和 switch emitter读取准确 Char 后输出。byte/short 和任意跨局部 fixpoint 不在本片。
3. **null 不决定非空写的类型。** null 首写候选仅当每个非 null 写都具有相同准确 Reference 类型，且至少有一个非 null 写，才把该类型放入原 map。Unknown/Object fallback不是准确类型证据；混合类型、未解 phi/copy/循环及全部 null 保留原结果。本片无需寻找共同父类或下载依赖。以全部写为条件，避免专用 finally 旧先例只取一条写的范围被无证泛化。 直接来源限同次Instruction的String/Class常量、引用返回/字段描述符、allocation/array creation；不复用可追copy/phi的lifetime helper。构造完成的alias仅在准确Special/<init>/void且同BCI SSA Ref(Named)与owner相同才准入，依据既有frame token转换事实，统一复用reference spelling。
4. **一次有限证明与原子停止。** 使用标准库和现有读工具；对每个检查的写/实际遍历节点计费并poll，不无界递归、不复用未经计费的全图扫描。Stop传播出原声明阶段；Builder/Report不发布部分结果，来源仍由原物理 store/producer/call/switch提供。
5. **完整源码验收。** 新 CLI/meta源 pins 独立冻结；真实五测试六 Java 输入保持全部 class、check()、Inner 和 SDK。两锚 default/all完整源码原样重编，以相同Runner和SDK比较exit/stdout/stderr，验证源/JADX矩阵及物理BCI覆盖。三个其他fixture保持基线正文/map与实际失败分类；Labels数值投影和条件fallthrough不能被本片统计为修复。加入无debug、混合写、范围边界、槽复用/未知合流、预算和取消反例。
6. **依赖选择。** 现有 Runtime、IR 和标准库已提供全部事实，不引入库或新的许可维护负担。AssertJ/JUnit等仅为原上游测试SDK，任务私有按官方坐标/hash获取，不加入产品依赖。

## Risks / Trade-offs

- [C producer 后续写任意 int，误窄化会改变调用] → 全写准入，近邻混合类型和边界值重编/运行；保留原调用严格门。
- [null 与未知引用被强转成消费类型] → 非 null 全写准确且类型相同，不以参数期望作证；全部 null 和异型写负例。
- [槽复用或未知phi扩大生命周期] → 复用原变量分割并拒绝不能覆盖的候选，不修此片以外的复用机制。
- [测试SDK中的原目标class掩盖输出缺失] → 运行class path保留实际目标输出，helpers目录排除switch测试目标，编译失败单列；不strip check或stub AssertJ。
- [资源与范围失控] → 5GiB free/target1GiB守卫，root串行工具链，完成清理；条件fallthrough与常量名另立项。

## Migration Plan

先root独立接受冻结基线观察，核真实IR和读完周边；Luna准备私有补丁与反例，root审查并顺序应用。完成相关/完整类对照、source-map和CI同范围检查后提交推送产品，独立接受其自身CI，再更新账本/handoff并交付干净main。历史失败和SDK/source/class/raw不覆盖，不维护旧错误行为兼容。
