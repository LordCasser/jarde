## Context

见 proposal 的 Why 和固定基线。实际收据 `da22ca92…` 的19B方法形成 prefix@4/header@7/latch@11 的双入口 SCC；原始和 JADX 重编48观察一致，Jarde在 `recover` 的不可约前置检查整方法拒绝。`NormalFlowView` 的分类是正确图事实，不应修改；`Region::Loop` 依赖自然循环，普通 Walker 尚未开始便已返回。

根审代码范围：`region.rs::recover/Region::blocks/overlapping_owner`、`normal_flow.rs::irreducible_blocks`、`build.rs::loop_region/instruction/compound_if_checkpoint`、`canonical.rs::CanonicalBlockId`。Atlas 的文件级符号查询及直接源码用于定位；一次宽目录 Operation 查询仍在后台准备，没有使用其未完成结果作结论。JADX FixMultiEntryLoops 的两种块复制提供算法参考；本片不复制其可变 CFG 操作，也不依赖其启发式成功标志。

## Goals / Non-Goals

**Goals:** 关闭受证的条件初始 int 前缀循环形；同一字节码的初始路径、每轮更新、退出值、溢出与全部物理来源均保留。最小范围是无异常/调用的完整方法，五种图角色闭合，任意类名/BCI/局部槽号、参数位置和有符号 iinc 常量不能成为匹配键。

**Non-Goals:** 任意不可约图、循环内异常/调用/内存效果、原 Smali 的数组与 String、嵌套此形、任意语句前缀、通用 CFG clone/state machine/solver、外部依赖或公共 IR 修改。普通 equals 展示 cast、checkpoint 命名/其它计费债务另列，不混入。

## Decisions

### 1. 必须证明图形，不能放宽分类

从 canonical 的真实 normal 边辨认 E(入口条件)、P(前缀)、H(循环条件)、L(尾段)、X(返回) 五个不同角色：E→P/H，P→H，H→L/X，L→P，X终结。可达 live 图必须正好由这些角色覆盖；SCC恰为P/H/L，只有E的两条外部入边、H的一条出边。每个正常 successor、predecessor、branch target/polarity均核原事实，不根据BCI排序猜 fallthrough。拒绝重复/额外边、其它live块、jsr path、任何异常记录/异常或call边、monitor、方法同步及缺失/未完整payload。无关不可达指令沿用独立完整性检查，不可静默抹去。

首片指令合同：E只含局部load及单值零比较；P只含一次int局部 Increment及可选直达H的transfer；H只含int局部load及单值零比较；L只含对条件局部的一次int Increment和回P的transfer；X只含prefix局部load和int return。基于已解码 Operation、同次 SSA/frames/effects及完整 MethodCodeFacts证明，不重新解码或从JSON摘要建表。E测试局部、P更新局部、H/L计数局部必须是互不混淆的三份实际绑定；P与计数局部类型必须严格为Int，不能把Boolean/Byte/Short/Char/Unknown按JVM int-shaped猜成源码int。E可为Boolean或Int，实际分支sense交由现有test_expr。iinc常量可变，不能匹配固定+3/-1；返回descriptor和真实消费必须为int。

Region负责拓扑、Operation和SSA角色；frame的Int只证明JVM类别，不等于源码Type::Int。严格局部类型及return descriptor的最终门放在Builder既有类型决定上，在任何AST提交前通过；不能把参数descriptor重复塞进IR或把窄/布尔类型宽化来满足证书。Builder拒绝仍quote完整候选。

SSA中的头部phi/回边允许承担同一源码局部，不要求每轮同一个ValueId；须证明每条输入与实际槽、宽度、局部身份一致，不能以同槽号猜跨作用域同名局部。任意额外读写、stack逃逸/交汇、冲突或不完整事实拒绝整个候选。先增加只读事实测试记录真实布局，再实现判断。

### 2. 一份物理所有权，两处受证展示

新增一个窄的区域角色（建议 `Region::EntrySplitLoop`），仅保存上述已证角色、测试BCI/sense及必要更新站点；不增中间CFG或虚拟节点身份。现有Region无法同时表达这个共享前缀的两个源码位置，新增角色是为表达真实来源所有权所必需。不要把P作为两个兄弟 Region 再放宽 overlap 检查，也不要挪用只表示jsr调用栈的CanonicalBlockId.path。

在不可约整方法拒绝前尝试该完整证书。命中后使用一个候选区域及原X尾段，仍进入既有最终 uncovered、overlapping_owner、unaccounted_instructions 检查；不能通过早返回跳过这些门。未命中保留原 whole-method Irreducible 原因和物理块。区域遍历/结构化判定/统计/来源读取按新角色闭合，不能用通用set去重掩盖真实重复所有者。

Builder用既有test_expr、局部命名/类型决定和int Increment语义形成：`if (takePrefix) { updateA; } while (continueTest) { updateB; updateA; } return A;`。没有新Java AST语法。先完整证明两次展示使用同一已决定的Java局部，prefix的Assign AST在局部候选中构造一次，受证复制到两个位置；不能在第二次经过SSA读取时重算为错误的初始值或借助合成状态变量。

两处前缀保留同一真实method/BCI原点，source map允许两个源码span指向一个物理指令；入口测试、头条件、尾更新、隐藏goto及最终return全部可追。区域物理拥有P一次；实际生成的两份AST/语句/文字按两份收费，不把源映射origin去重误当只需一份输出成本。

### 3. 完整提交与停止

图/SSA/边和集合读取先计费/poll再分配或保留；候选证明失败不改变原图/visited/所有权。Builder沿用既有完整结构试写 checkpoint（现有 compound_if_checkpoint/restore 可覆盖相同触碰集），不复制第三套大型状态表、不顺便重构finally状态。核实实际触碰字段均可恢复；试写后普通拒绝恢复状态并quote整个原候选，Stop恢复后原样传播Err，使用量不回滚。没有前缀成功而尾段失败的partial AST，也不借一次后置SourceMap停止掩盖前置漏费。

固定上限是五个闭合图角色及已列出的指令合同，不搜索任意路径或枚举循环迭代。输入解析、恢复、证据请求、JVM verification和受控行为运行分别报告。

### 4. 依赖与参考

使用现有Rust标准集合、petgraph图事实及现有恢复Runtime/预算；无需新库/安装工具。JADX已有源码与Apache许可快照仅作算法/对照参考；采用图上的前缀剥离思路，自行按本项目只读事实和唯一所有权实现，不搬运其可变BlockNode模型。JDK/JADX只用于用户已明确授权的测试，不进入核心依赖。

## Risks / Trade-offs

- [phi/局部同槽不等于同绑定] → 用真实SSA输入、精确类型与局部身份；错位/别名/不完整单变量拒绝控制。
- [前缀复制改变每路径次数或溢出] → 固定图角色证明，原/生成完整类双编译器、双VM，skip双值、零/一/多轮、MAX/MIN附近及变常量/槽位对照；不执行需要数十亿轮的输入。
- [新Region绕过旧完整性门] → 复用最终所有权和物理指令覆盖检查，专门验证多余live块/未accounted字节的拒绝。
- [来源重复使统计和收费失真] → 物理owner一次，两个展示span均有锚，克隆及实际输出两份费用；真实小预算/取消不发布半成品。
- [只覆盖窄形却宣称追平整族] → 账本保持CF08 partial；原Smali及其它多入口形仍单列。

## Migration Plan

先冻结基线、写永久正反用例及只读事实，再由Luna私有实现；root审查后才集成共享主线。root串行编译/对照，生产变更通过适用native、完整workspace与自身CI后再同步主规范并勾最终交付。失败保留raw与拒绝，必要时撤回本片候选，不改原始/JADX文本或oracle制造通过。资源门沿用可用5GiB、target1GiB、默认栈、每秒守卫和finally精确清理；Cargo期间不改源、不做Git mutation、网络或手动JDK/CLI。
