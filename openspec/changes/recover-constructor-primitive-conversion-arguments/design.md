## Context

动机见proposal。前片CLI2为78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273，原18腿16/18，旧direct完整家族0/2；五wrapper仍在参数conversion BCI以StatementFree拒绝，nested covariant child-array与BigDecimal仍独立未覆盖。前片本地全门禁已验收，代码5a6264de已提交推送、确切SHA CI37947791151四job全部成功；本片2/7任务，产品尚未实施。

已有recover-primitive-conversions实现全部15opcode、显式Cast、源类别/Boolean拒绝及完整数值对照。`init::verify_metered`普通参数区间缺少该operation分类。JADX不是DEX-only：JavaInsnsRegister:217-231将classfile0x85..0x93转换为公共opcode，经共享InsnDecoder带source/target类型生成CAST。Jarde无须照搬类型简化器或删除widening cast；已有Cast路径保留转换链。历史DEX-only勘误见前片results/primitive-constructor-conversion-next-audit-v1-erratum.md。

## Goals / Non-Goals

**Goals:** 接通普通new的实际参数转换依赖，并保持完整区间、值身份、handler覆盖、来源、预算与失败正文；完整wrapper与普通return-new正例双javac验收。

**Non-Goals:** 不扩成员类专用参数白名单和statement-position-news，不允许未知dup/alias，不添加数值常量折叠、范围求解、一般DAG共享保存、shift/bitwise/checkcast，不处理旧new所有invoke的handler债务。普通construction verifier既有漏计需通过已有Budget管道补齐，限定这一入口，不审计其他规则。

## Decisions

### 1. 复用解码和Builder，仅扩大准确的参数operation边

在普通构造参数既有scan内允许PrimitiveConversion，当且仅当该BCI属于物理实参的argument_dependencies。其唯一stack input、Java presented source类别及target由已有render_value/primitive_conversion_source_matches检查，new_expr再按constructor descriptor验证，Int栈形不能替代Boolean/窄数值Java证据。不能复制一份类型表到init；不增加AST/pass/库。外部库不能提供本引擎SSA与物理消费证据，增加维护/许可负担没有收益；参考JADX公开算法但不复制其冗余cast删除策略。

完整两类基线在两真实JDK均有效：Jarde旧CLI均拒绝/编译失败；JADX none/default四腿均compile/run0但两long往返值错误（16777217→16777217而原程序16777216；9007199254740993→9007199254740993而原程序9007199254740992），0/4语义成功。保留baseline-v1完整历史；不为制造JADX成功删除舍入目标成员。

parse、dialect、runtime选中、结构verify、Java恢复与完整重编/执行分别计证据。Builder拒绝不销毁真实Site来源，但整正文必须保持拒绝，不能半份new或可编译空体算成功。

### 2. 转换依赖沿原参数位置执行，额外dup继续拒绝

参数仍必须在allocation-copy之后、constructor之前产生；不属于依赖的转换进入原拒绝路径。普通额外dup仍以StatementFree拒绝。真实dup会读原ValueId一次、写两个distinct ValueId，所以不能声称mark输入有两个uses而由deferred-binding保护。负控制复用冻结PrimitiveLongPair(JJ)：从storedLocalReuse(I)PrimitiveLongPair的已存在pool entries派生 `new;dup;markInt()I;i2l;dup2;l2f;f2l;PrimitiveLongPair.<init>(JJ)V;areturn`，保持实际返回类型与constructor descriptor；显式核对category-2 dup2的一读两distinct ValueId及实际StatementFree拒绝BCI。不用未存在的Pair(IJ)冒充真实fixture，也不增加仅用于该控制的helper类型。未知Duplicate的Builder路径也无一般copy表达式，但不能把第二道拒绝当作已支持共享DAG。

已经存入local再分别load的两个参数复用的是此前值，作为正控验证mark恰好一次；相反在参数区间插入store或不属于依赖的效果仍拒绝。数组caller已有参数single-use/use-containment继续保留。

### 3. 仅新路径启用已有handler区间检查

PrimitiveConversion本身不抛JVM异常，不代表参数调用、allocation或constructor可以跨handler重排。现有init handler loop只因embedded concat/dynamic/array/nested启动。记录本Site参数区间确有新放行的conversion，并把该条件加入同一loop：比较allocation到constructor的实际handler ordinal序列以及唯一consumer。只增加此feature必要的触发与准确拒绝说明，复用原meter逐读取收费，不再另扫SSA或建立effect服务。旧普通invoke范围的handler债务不扩围。

真实普通入口的verify目前创建budget:None，不能把它说成已有共享预算证明。生产sites_after_array_composition及其内部census接收report本次Budget并返回Result<Sites, StopReason>，普通候选直接调用已有verify_metered：Refusal仍登记，Stop直接返回并由report映射stopped，不发表局部Sites。数组pending Sites仍单次move；停止时局部plan丢弃、后续region/build/materialize不运行。该修正必要地把整个普通construction verifier既有实际读取纳入计费，除此不迁移其它pass的旧漏计。不能先全SSA预扫conversion再重复验证，也不能用长度估价或feature开启开关掩盖早期扫描。测试便利入口可显式unmetered，但不能作预算证据。

### 4. 复用完整语料，不制造假的成功分母

复用旧direct-v3全部六类输入，不删其ownGrid等失败成员；若五wrapper已恢复但完整direct仍失败，分别记录局部呈现与完整失败。另从创建起建立最小完整wrapper/普通return-new家族，保留可观察mark、转换后的值与对象类型，双真实javac输入原/JADX/Jarde全部源集重编、运行-Xverify:all、原始双流逐字一致；不得借原class、helper或剥离成员。

15opcode源/target与数值链复用已有decoder、source-category、p3-primitive-conversions完整类和边界回归。新测试集中构造参数与失败边界，不重复建立纯cast框架。code派生负控只在内存使用真实reader CodeSpan，精准检查extra-dup、无关conversion、handler split与Boolean；不执行故意变义类。所有历史失败保留，新runner拒绝覆盖。

## Risks / Trade-offs

- [dependency集合被误当唯一consumer] → 额外dup真实SSA与实际StatementFree拒绝，stored-local正控分别验收。
- [纯转换放行绕过参数调用handler] → 仅激活新conversion路径的原闭区间handler loop，边界不同完整拒绝。
- [结构verified误算Java成功] → 独立核对body marker、完整source compile、exit及原始双流。
- [普通构造漏计被误称已闭合] → 单一生产入口透传既有Budget及Stop，真实普通return-new停止/取消对照，无半Sites。
- [扩大成员/一般alias或数值折叠] → 保留旧拒绝，另片记录；不改AST和类型系统。
- [磁盘产物增长] → root串行Cargo、20GiB停建线，冻结CLI与证据后清理本项目target。

## Migration Plan

前片产品源码和CLI2已冻结，允许在等待其确切SHA CI时先对固定输入完成旧CLI/JADX只读对照并保留全部基线；前片CI全部成功仍是本片产品改动的前置条件，task1.1不得提前勾选。本片基线、最小实现和root完整对照各独立保存。无外部格式迁移或兼容适配。Luna只负责限定实现与fixture，root负责Cargo/Git、真实双JDK重放、对抗验收、双seed及最终代码CI。
