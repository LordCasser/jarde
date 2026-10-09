# GC10 same-class generic-call 计费审计

范围：只审阅 generic-call component 图规划中的实际重复工作及其预算覆盖；此次源码收窄仅涉及 `src/class_source.rs::generic_call_components`。没有 Cargo/Git/target 操作。`facade.rs` 与共享临时克隆记账债务仅记录为后续建议，没有在本片处理。

## 已确认并在本片收窄的工作

原 Kahn 阶段每取出一个 callee 都遍历整个 component 的 `members`，并在每个 caller 的依赖 `Vec` 上调用线性 `contains`。N 个顶点、E 条依赖时最多是 O(N² + N·E)；稠密 DAG E=Θ(N²) 会达到 Θ(N³)。原 `N+M` `AnalysisSteps` 预charge已对应候选/依赖输入扫描，不能再抵扣组件遍历、indegree、或拓扑阶段的重复处理；原内层也没有 poll。

Kahn 更新现在沿 `adjacency[callee]` 只访问相邻顶点，并用已排序的 `callee_dependencies[caller].binary_search(callee)` 判定有向关系。拓扑更新因此从全成员×线性依赖搜索收窄为邻接边访问加对数查找。图阶段现对额外逻辑工作显式 charge `AnalysisSteps`，charge 在工作发生前调用预算 poll：候选图节点初始化；各待排序 Vec 的逻辑元素；BFS 的成员处理及邻接访问；成员排序、indegree 节点/依赖检查、ready-set 扫描；以及每次 Kahn 出队和相邻顶点检查。排序按输入逻辑元素预charge，而非尝试为比较次数引入 CPU 计量模型。

新增定向测试构造 8 个 callee、8 个 caller、64 条 dense fan-in 依赖：`AnalysisSteps` 只设为 `candidates.len() + dependencies.len()`，即输入清单预charge额度；测试断言该额度不足以完成后续图处理，充分预算时断言 callee-first 完整顺序，预取消 token 时断言取消传播。该测试已写入现有 `class_source.rs` 单测模块；本片未运行测试，格式化/整仓门禁由主线程执行。

## 未纳入本片的后续债务

`facade.rs` 的 `has_edge` 在每个 component 上重复扫描 dependencies、invokes、AST，并以 component 成员 Vec 做查找；component 外层只有一次 poll。前置 M×A 方法索引与 I×N 依赖生成以及本函数的图预算均已用于各自阶段，不应复用作这段扫描的预算。建议后续用一次 component membership 标号（method index → component index），对 dependencies/invokes/AST 候选做一次性分类并标记各 component 的 has-edge/触发状态；在分类输入循环内按实际处理项 charge/poll，从而避免 component 数量乘全输入扫描。若采用该方案，membership 标号构建本身也应计入；不要再对每个 component 重扫完整输入集合。

`record.item.identity.clone()` 会为每个 staged candidate 深拷贝变长 `PhysicalMethodId` 字段。按输入的 AnalysisSteps 可覆盖候选步骤数，但不是变长复制字节上限；`OutputBytes` 用于发布输出，不能借作临时 clone 字节预算。`attribute_shells` 是既有共享 helper：它扫描成员所有属性并深克隆匹配的 `AttributeShell`；generic contract 新调用点在 `attribute_facts` 内容计量之前触发它。单个标准 Signature 名称的 payload 至少复制 raw 9 字节和 UTF-16 18 字节，另含 span/容器开销；重复属性使克隆量按输入增长。建议与通用临时存储/字节记账设计一起另立债务，不将 helper 全库改造并入图算法收窄。

预算维度依据：`AnalysisSteps` 表示 worklist/processing step，工作前 charge；`IrItems` 专指派生 IR 存储项，`OutputBytes` 专指输出。这次图候选、依赖、成员和邻接关系并不扩大为新的 IR/registry 实体，也未引入新的预算维度。
