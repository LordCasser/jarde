# EM18 补充审计：snapshot hierarchy 事实与 `aastore` 站点

## 结论

原 spec 的“赋值子类 → 父类/接口数组分量”范围不能用六条 boxed `Number` 规则代替。六 wrapper 是可审计的闭平台表；当前代码另有一条基于分析快照 class-file header 的、受运行环境选择和深度预算限制的层级证明。它证明的是具体站点上给定 `source -> target` 的关系，不是全局类型系统 API。EM18 可以复用其层级 walk 和 header 选择机制，但不能直接拿一条 invocation 记录去授权 `aastore`：当前记录由调用参数 descriptor 产生，并绑定调用 BCI。

建议扩展同一证明入口，另加“initializer store site”输入：从真实 SSA `aastore` 读取存入值的 named reference 类型，以及同一 store 所用数组值的 named component 类型；仅在两者都是非数组类名、类型不同且 `snapshot_header_chain_widens` 返回真时，为该 `aastore` BCI 记一条 `(source,target)` 证明。`Builder::array_initializer_element` 只在它已证明的 fresh-array initializer 配对 store 上按 BCI、presented type 和 component type 精确消费该证明。现有数组/平台闭表与 null、exact、Object 处理保留原顺序；没有 snapshot 关系证据就维持拒绝。

## 生产者实际做什么

Atlas 的项目已打开。scoped 查询定位到 `crates/jarde-java/src/build.rs` 的 `Inputs::snapshot_hierarchy_widenings` / `Builder::snapshot_hierarchy_widenings` 字段；Atlas 当前本地 structural index 对 `src/facade.rs` 的 producer 命中不完整且行号过时（精确函数搜索未命中）。因此下面的函数、分支和行号是按当前 checkout 的源文件 `rg`/局部读取复核，不把 Atlas 的旧位置当成当前事实。分析范围局限于当前 `src/facade.rs`、`crates/jarde-java/src/build.rs` 和 `crates/jarde-java/src/report.rs`。

- `src/facade.rs:24994-25106` 的 `prove_snapshot_hierarchy_widenings` 目前只遍历 `invokevirtual`、`invokespecial`、`invokeinterface`、`invokestatic`。它从 method/interface-method constant-pool descriptor 取 reference parameter site（接收者也作为实例调用的第一个 operand），再按该 BCI SSA instruction 的 stack reads 排序，拿出对应参数的 presented `Value::Ref(RefType::Named)`。数组和未知类型不进入 class-name walk。
- Required type 来自所选调用的 descriptor；对 `Object` 跳过，因为前面的 dispatch 已无条件处理。只对 source 与 target 不同的 pair 调 `snapshot_header_chain_widens`，上限是 8 层。成功时产出 `ProvedSnapshotHierarchyWidening { bci, source, target }`，其中 BCI 是 invocation BCI。
- `snapshot_header_chain_widens` / `_with` 在 `src/facade.rs:24203-24273` 做受限图搜索：每个队列节点都 poll、收取 `AnalysisSteps`、记录 dependency depth；访问集合防环。每个节点用同一选定入口取得 header，然后从 `super_name` 和 `interfaces` 展开。未能读到 header 的分支停止；层数过界不继续。缺事实、预算取消、读取不完整时都不产出证明。
- `selected_reference_header`（`src/facade.rs:24095-24127`）对正在分析的类直接使用本次 `MethodIr` 的 declaration/header；其他节点调用 `resolve_class_source_dependency_read_raw(content, request.environment, Some(request.method), owner, ...)`，并缓存结果。该 raw resolver（`src/facade.rs:20268` 附近）使用请求中的 `ResolutionEnvironment`，经 `jarde_jvm::resolve_symbol` 按该运行环境/loader 选择定义；只接受 `Resolved`、执行完整、无 environment problem / unresolved dependency / competing candidate，owner 精确匹配，且恰有一次 requested-definition read（允许 enclosing class 的那次 read）。随后从被选 snapshot 读 class facts，拒绝 stopped/truncated 或 `this_class` 不匹配的结果。也就是说，walk 的中间节点不是在任意 classpath 上按名字搜到就算数。
- 特例 `proved_reference_widening`（`src/facade.rs:24160-24201`）另有 Java 8 `ArrayList -> List` 平台事实，但要求 ParentFirst + ClassPath、没有外部覆盖和 runtime transformation uncertainty，并通过 `platform_class_unprovided` 确认 selected order 没有 competing snapshot definition。它与一般 snapshot header walk 是不同证据通道。
- facade 仅在 `assembly_context.is_some()` 时于 `src/facade.rs:40477-40483` 调用这个 producer；method-only request 明确得到空列表。这是现有 scope 限制，若 EM18 想让 method-only 路径也使用证据，须明确扩大 facade 入口，而不是误以为 proof 一直存在。

walk 有一个重要语义：在 `snapshot_header_chain_widens_with` 内，先判断队列节点是否等于 target，再读取这个节点的 header（`src/facade.rs:24235-24240`）。因此 target 可以是某个已读 header 直接点名、但不在 artifact snapshot 中的类型，例如项目类直接 `implements` 的平台接口。它不要求 target 自己一定是物理 snapshot class。相反，中间节点必须能从同一选定 snapshot/read path 得到完整 header；缺失节点不会借 classpath 猜测后续链。

这与 `ProvedSnapshotHierarchyWidening` 当前文档（`crates/jarde-java/src/report.rs:266-277`）说“source 与 target 都是 snapshot physical class，平台 target/intermediate 一律不能证明”不一致。producer/walk 的可执行行为允许**直接被 snapshot header 点名的非 snapshot target**。EM18 规划应以 producer 的实际语义为准，并修正这段错误注释；不要因旧注释把已经存在的 direct-interface 证据缩掉，也不要把它夸成可解析任意外部层级的闭包。

## 为什么 invocation 记录不能原样挪给 initializer

Recovery 侧的消费点是 `crates/jarde-java/src/build.rs:25518-25527`：它要求 `proof.bci == invocation bci` 且 source/target 等于该实参 presented/required 名称，然后才允许原参数表达式通过。producer 的 `bci` 表示被 constant-pool invocation descriptor 约束的调用点，不是一般“此类型关系在任何位置成立”的 token。直接把别处的 call proof 对照 initializer 的 `aastore` 使用，会错配站点；即便类型名碰巧一样，也没有证明该 store 的 SSA 存入值与数组 component 正是这对类型。

initializer 路径已有独立安全边界：`Builder::array_initializer_element`（`crates/jarde-java/src/build.rs:26188-26246`）拿 element expression presented type 和 `component` 比较，当前仅接受 null、精确类型或 `Object`；`store_bci` 已用于锚定转换；外围 `ArrayInitializers::prove` 负责 fresh allocation、真实 store 配对/次序和表达式唯一使用（见前一份 `/private/tmp/em18-next-architecture-audit.md`）。因此应给这条**已证明的 initializer store**新增匹配的 snapshot proof，而不是把 invocation 缓存变成无站点的全局 assignability 表。

## 最小可复用算法与约束

1. 保留现有 `selected_reference_header`、`snapshot_header_chain_widens` 和同一请求 `ResolutionEnvironment`。把“给定 source、target、site，是否由该 snapshot 证明”的证据提取为共享的小逻辑；header 选择与调用参数/数组 store 的 operand 抽取分开。
2. initializer 侧只处理 opcode `aastore`（`0x53`）。从该 SSA instruction 的 stack reads 按实际 stack slot 顺序识别操作数 `arrayref,index,value`，而不是从源码表达式/局部变量名猜类型。必须确认数组引用是 named 一维 reference-array descriptor `[L...;`，存入值是 named non-array reference；从数组 descriptor 取得 component internal name。primitive、null/unknown、不完整 SSA、malformed descriptor、数组 component、其他 array op 均不产生 snapshot class proof；既有 closed array proof 仍负责数组对数组转换。
3. 只有 `snapshot_header_chain_widens(source, component, depth=8, ...)` 为真才输出以此 `aastore` BCI 锚定的证明。预算 poll/charge、dependency depth、resolved selected header 限制沿用当前 helper；budget exceeded/cancelled 继续 fail closed。不要新增无预算全局 hierarchy cache 或第二套 classpath walk。
4. recovery 仅在现有 `ArrayInitializers::prove` 已将该 BCI 认定为某一 fresh allocation 的 element store 后消费该记录，并再次精确比较 store BCI、SSA/presented source 和 array component target。成功只消除人为拒绝，不插入 cast，也不改写运行时数组类型、求值顺序或 `aastore` 的兼容/异常语义。
5. predicate 顺序继续是已有精确/`Object`/null、数组闭表和其他当前 reference widening facts，最后才看 snapshot hierarchy。平台六 boxed `Number` 表继续只代表其明确记录的 JVM/API edge；custom snapshot subtype/interface 必须走真实 headers；两者都不宣称完整 Java 类型检查器。

注意 target 是否是 snapshot class 的细节：当前 walk 能凭一个真实 snapshot header 的 direct `super/interface` 名字证明 direct target，即使 resolver 不从 snapshot 读到 target 自身。一般链式继承则需要每个中间类都从同一运行环境选到可信 header；不可把“一个名字出现在字节码中”扩展为后续任意继承链。若同名类型在 loader/domain 下的身份无法由现有 resolver/environment 事实确定，或解析出现歧义/transform uncertainty，停止证明并保留失败；不能用简单字符串类型名弥补 identity 证据。

## bounded 验证建议

- 正例：artifact snapshot 中 `Child extends Base`，一个 `Child` typed expression 填入新建的 `Base[]`；另一个 `Impl implements LocalInterface` 填入 `LocalInterface[]`。覆盖 direct 和两跳（每个中间 header 可解析）层级。加一个 snapshot 类直接实现平台接口的 direct-edge 例子，验证 target 可按 producer 的实际 walk 语义不在 snapshot。
- 负例：source/target 同属 snapshot 但没有 super/interface path；中间 class header 缺失/截断；超出 walk 深度；解析选中 definition 有歧义；以及同名类但请求 load domain 不足以确认同一运行时身份。都必须维持原失败文本/或原拒绝类别，不能用调用点的其他 proof 越站点授权。
- 站点负例：方法内另一处调用曾证明同一 `Child -> Base`，但被测试 initializer 的实际 source/component 组合不同，确保 call BCI 记录不授权 store。对确属数组 covariance 的旧引用（静态 `Base[]` 指向更窄 runtime array）不要放宽一般 `aastore`；只允许 initializer fresh allocation 的 exact component case。
- 回归已有 Number initializer 六闭表情形、primitive arrays、array-reference widening、null/Object/exact、side-effect 顺序与 `TestArrayFill2.test2`。后者是 `int[]` 写入表达式语义/NYI 语料，不是 boxed `Number` 或 class hierarchy 例子，不应拿它支持本算法。

这项架构工作完成前，EM18 的验收标准应明确包含至少一个非 boxed 的 snapshot class/interface 子类型到数组 component 正例，否则即使六个 `Number` wrapper 全过，原 spec 的 broad assignability 目标仍未满足。
