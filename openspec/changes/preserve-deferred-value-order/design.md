## Context

证据为`../../evidence/java-syntax-2026-09-22/construction-consumers/interleaved-effect/`和`../../evidence/java-syntax-2026-09-22/deferred-evaluation/`。补丁仅在生产值后、return前插入已有CP的独立void调用，并调整Code长度；均通过JVM验证。构造的原轨迹12被写成21，字段旧值5变9，数组旧值7变8，先发生的CCE/NPE被后续调用的异常替代。这里的原始oracle是**补丁class**，不是未插入调用的javac class。

现有Builder的produced_value_reaches_a_reader只判断存在受支持的读取者，deferred记录用于失败时引用调用。render_value已经把局部旧值检查放到最终at，但调用、构造、字段、数组和cast的可观察求值也被一起移到at，未证明它们能跨过中间独立语句。SSA说明“读哪个值”，不会自动保证重新计算该值的时机。仅修改init消费者名单会扩大这个问题。

Builder已有instructions、block_of、SSA uses、结构化region、构造Site、Declare/Local及统一push/预算。NameTable.free_name已为没有JVM槽的lambda形参提供命名，Builder另有已使用的合成名称集合。无需更换IR或AST。

## Goals / Non-Goals

**Goals:** 对原本能够表达的直线值生产/消费，区分可安全内联与必须保留已计算值的情形；固定四份直线审计57项及独立结构化作用域12项的原样完整类执行，并保留嵌套表达式对照的内联质量。

**Non-Goals:** 不重建stack phi、任意控制流值合并或guard，不写通用Java临时变量提升器，不新增字段/调用解析，不把全仓纯度分析作为前置。范围外形状仍须明确拒绝，不能保持已知错序的正常文本。

## Decisions

1. 在既有Builder准备/值构造中加入有界的放置判断，复用同一SSA和指令索引。判断的是一次表达式实际消费的依赖链及BCI顺序，不是“pure名称”或“有uses”。当生产者和最终呈现之间存在不属于该表达式求值链的独立语句时，不能继续把生产者直接内联到后面。多个嵌套调用本来同属最终表达式、且按Java操作数顺序与字节码一致时，保持现有内联。未知指令、未认领形状、phi或不能证明的路径不得被当成顺序透明。
2. 使用现有Declare/Local保存必须跨语句保留的值，声明放在原生产者求值位置，后续读取绑定同一个SSA值，禁止在消费者处重算。普通调用、已认领字段读取、数组读取/长度/分配、整数除法/取余、普通cast和已验证new站点共用这一项规则；不是每个语法点各建一个缓存。构造值在constructor位置形成完整new表达式，Site的new/dup/constructor身份指向同一绑定，不重复构造、不把uninitialized值命名为已初始化引用。
3. 唯一必要的新状态是本次Builder中“SSA值或已验证构造身份 → 已成功呈现的局部名/类型/来源及可见作用域”。已有region路径与语句范围可以作为构造上下文传递，不另建作用域系统。不能通过编造LocalVariable槽号、扩大max_locals或改写SSA来模拟它。先成功构造表达式、确认可声明类型、计费并提交声明，再让后续render_value读绑定；失败不得留下未声明名字。用于声明初始化的第一次呈现必须绕过自身绑定，防止`T value = value`。
4. 只在能够证明Java作用域覆盖消费者且不改变区域执行次数的位置建立绑定。复用已有region路径/指令位置和直接语句列表；分支或try内的值不能因为BCI较小便在兄弟/外层可见，循环条件不能把每轮生产搬到循环外。必须将普通同块直线、分支内直线及测试前缀正面闭合；不能表达的跨区域/phi/guard形状保留来源完整的拒绝。这里不新增跨区域声明提升机制。
5. 不以第二份opcode表猜异常或副作用。独立语句及未被依赖链证明承载的指令构成顺序边界；必要的异常范围信息从同一canonical事实读取。构造仍遵守Site原本的参数顺序、唯一消费和所有权。现有赋值、字段写入或void调用的位置不变，不把后面的语句倒着移动来掩盖生产者延期。
6. 合成名称复用NameTable.free_name及Builder已有非槽名称分配，统一检查既有局部、lambda形参、其它绑定及final-static保留字段名；不得建立彼此不知道的多个名字空间。每次因合成名字冲突追加后缀后仍须检查原局部/字段；`synthetic-names/`的两个捕获lambda已证明只在初始候选调用free_name不够。该输入另有整类lambda实现方法名冲突，记录为独立成员呈现债务，不并入本项。名字不声称来自debug元数据，也不声称原class存在相应局部槽。
7. 声明初始化保留生产者及必要构造/操作数BCI，后续引用保留真实值来源和消费者位置；不伪造新的物理BCI。默认与完整证据正文相同。判断遍历、绑定条目和新增语句计入现有预算，轮询取消并守住深度限制；commit/replay只读取已决定的同一程序，不在证据阶段重新决策位置。
8. 不依赖第三方库：缺口是已有SSA呈现位置与值保存之间的语义，外部库不能替代本地作用域、所有权和预算契约。复用Rust集合及现有节点即可。没有新crate、持久缓存、全图索引或额外恢复pass。

## Risks / Trade-offs

- 所有值都拆成临时变量掩盖证明不足 → 固定嵌套receiver/argument/new参数等原本有序输入，要求保持可读内联，只有需要跨独立语句保存的值才绑定。
- 构造或调用重复执行 → 绑定与成功声明提交一致，构造Site多身份归一，运行比较身份、计数和先后失败。
- 只修调用而遗漏读取/检查 → 同一回归类同时覆盖call、new、getstatic/getfield、array load、array length、newarray/anewarray/multianewarray、checkcast、整数除法/取余；null/CCE/bounds/NegativeArraySizeException/ArithmeticException和生产者自身抛错均执行。
- 局部旧值被重新求值 → 绑定初始化使用生产点，后续引用读取已捕获值；不篡改现有局部相等性判断或未绑定值的最终at。
- 源码作用域与CFG块误当成同一件事 → 区域正反例验证；同一基本块内可能穿过try边界，不能只看block id。`deferred-evaluation/exception-scope/`已有合法三项对照，当前guard先行拒绝，不因本项要求扩大其准入。
- 新遍历/名字/条目不计费或递归溢出 → 将每项工作纳入现有预算，固定小预算、取消、深链与失败回放测试。

## Migration Plan

这是内部恢复语义修复，不保留错序文本兼容性。先冻结真实红测试，再串行实现共享放置/绑定和全部生产者，最后用原样完整类及独立字节码验收。若部分形状尚未完成，不以放宽测试或把正面全集变成引用收尾；继续实现已约定的有名值保存。new→cast/aastore消费者扩展保持独立，等本项顺序契约稳定后再处理。

## root 补充：嵌套 producer 的保存位置

不能仅因inner的直接consumer也是待保存producer，就删除inner的保存计划。合法直线 `value(); mark(); take(value); mark(); return result` 要求内层值在第一段mark前固定、外层值在第二段mark前固定。沿用同一有界依赖/区间证明，分别以实际保存位置判断是否允许内联；只有inner→outer之间没有独立效应且一次消费已证明时，外层声明才可承载内层表达式。该问题不需要新IR或另建效应框架。switch在构造arm前预渲染返回时，也必须看到已经决定的拒绝事实，不能等instruction提交拒绝后留下已生成的旧调用。
