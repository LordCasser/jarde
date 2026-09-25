# Java 语法恢复持续巡查：2026-09-22

本记录只记本轮实际检查的样例，不把旧任务勾选、jadx 可读性或 `Structured` 状态当成执行等价。源码为自写；编译器 javac 23.0.1，`--release 8 -g:none`；jadx 1.5.6。基线使用本轮改动前的 `target/debug/jarde-cli`，不用旧 release 推断当前能力。

## 调度与边界

| 语法点 | 已核实的缺口 | 架构判断 | 当前处置 |
| --- | --- | --- | --- |
| 一元数值取负 | 四种 `*neg` 解码为 `Other` | 一个一元事实和一个 AST 节点足够；沿用消费点、类型、来源和 emitter | 独立 change `recover-unary-negation`，Luna xhigh 实施、主代理验收 |
| 普通引用转换 | `CheckCast` 已解码，但值构造只接受 bridge 认领 | 已有 `Cast` 可表达；区分“保留运行时检查”与 bridge 的“已证明可擦除” | `recover-explicit-reference-casts` 生产语义及主代理独立验收通过；全仓 census/严格门禁分别记录 |
| 特殊调用 | 两种 super 均被写成 this，执行触发 StackOverflowError | 必须分开 class-super、interface-super 和本类 private；借用已有 MethodIr 类头与 SSA | `preserve-special-call-dispatch` 专项语义验收通过；严格 clippy 既存债务独立记录 |
| long/float/double 比较 | 21 个数值分支均引用；int 对照已恢复 | 比较结果事实 + 现有 Binary/Not 足够，必须保留 NaN 极性 | 实现及root整类242项验收通过，Java包176项及相邻43项通过；新一轮语料总量冻结另行记录 |
| 调用重载选择 | 无 cast opcode 的 Object/char→int 参数类型丢失，已生成可编译错值 2/4/8，原值 1/3/7 | 调用上下文应保留目标 descriptor 的参数静态类型；可复用 Cast，不需要 resolver | 主代理已验收：fixture12行、独立12行及this参数对照通过；最终Java包176项通过，既存lint单列 |
| 静态初始化完成 | 顶层尾部return导致javac拒绝 | 共用emitter在已声明clinit尾部省去return，来源附到闭合字符；无需新AST | 主代理已验收：整类执行、空块、首次/再次初始化失败与真实来源预算通过；合法提前返回仍保留边界 |
| 裸throw | 事实已解码，但直线语句与消费呈现未接通 | 需要一个忠实Throw语句节点，复用已有区域及消费证明 | root已验收：完整正面类13行及独立15项相等，Java包178项/相邻61项通过；91类219文件冻结完成，既存lint单列 |
| blank static final 写入 | 已恢复初始化块中的限定赋值被 javac 拒绝 | FieldAssign 简单名与现有 NameTable 避让闭合，不新增 pass | root验收完成：181包测试、38相邻及JDK通过，四类五次执行和初始化失败六行一致；接口单列 |
| instanceof | 缺类型测试表达式；源码Object加宽在字节码中消失 | 一个操作事实与boolean节点，复用Cast和共享boolean判据；不需要层级resolver | 规划strict通过，15 Code正面fixture/22行原始执行已冻结；root以8cd1冻结CLI独立重放1124B类型边界：原类11行、jarde六处缺返回、JADX四处不兼容类型加一处方法引用语法错；见`instanceof/type-boundaries/root-8cd1/`，生产待实施 |
| 位运算及非短路boolean | 修前六个int/long opcode未恢复，boolean共用int指令 | 复用Binary和统一分组；用现有类型队列闭合整数/boolean证明，0/1字面量不能独立启动boolean | `recover-bitwise-expressions`已由root验收：最终CLI冻结重放见`bitwise/root-after/`，1303B/23Code整类原/JADX/Jarde 268行一致、零引用；合法Z/I混合仍保守拒绝，类型/预算/来源与census/fingerprint门禁通过；仅剩既存Clippy `region.rs:1736`债务 |
| 字段/数组复合 `+=` | 永久1113B/12Code完整类原class/JADX/Jarde七行运行全同；19个边界原/JADX全同，Jarde六个已对，其余保守拒绝 `+=`，但部分引用正文尚不等价 | 现有字段/数组赋值语句仅在 SSA 身份、单次消费及效果顺序均证明时发射 `+=`；无需一般栈复制机制。拒绝边界的可编译但不等价问题作为独立架构债务记录 | `recover-compound-lvalue-updates` 生产与root独立验收完成；冻结CLI SHA及完整回放见`compound-assignments/post-fix-fixture-replay/`，Rust/Java定向测试通过；严格Clippy及批量语料门禁另有记录，见 change `verification.md` |
| 字段/数组后置递增返回旧值 | 新增完整类1228B/11Code原class/JADX 19行全同，Jarde两后置方法缺return而整类javac失败；null、越界、溢出和调用顺序已覆盖 | 需要证明`dup_x1`/`dup_x2`留下旧值及写入新值，不能套现有仅局部接收者的`field_increments`或把复合`+=`计划扩张 | `recover-postfix-lvalue-values`规划strict通过；主类及十个合法边界均经root独立复制重放，1.1/1.2已验收；旧字段自增文本债务单列于`architecture-debt.md`，生产排在复合`+=`之后 |
| 注解默认值及类型声明 | canonical 注解头与嵌套默认值已修：Basic三行、Nested两行完整类的原/JADX/Jarde Java 8 编译、`-Xverify:all`反射结果相同；F/D默认值另见下行 | 现有 `MemberDefault` 递归拼写嵌套值并维持数组全有或全无；F/D沿同一路径处理原始bits，无需通用注解机制 | `spell-annotation-type-headers`及`recover-nested-annotation-defaults`均由root独立验收，见`header-minimal/post-fix-root-replay/`和`post-fix-nested-root-replay/`；F/D另见下行 |
| 浮点注解默认值 | 293B Java 8注解类四种F/D默认值修前省略、修后原/JADX/Jarde raw bits一致；受控负NaN及payload NaN class 可验证执行，Jarde修后对不可忠实拼写的默认值保守省略，F/D数组保持整段拒绝 | 沿用私有`MemberDefault`与reader原始bits；有限值精确十六进制拼写，标准特殊值准入，其余NaN拒绝；字段 ConstantValue 与 Code 浮点路径不扩张 | `recover-floating-annotation-defaults` 实施及root独立验收完成；冻结CLI、完整类及八项有限/特殊值、F/D数组和补丁重放见`annotation-float-defaults/post-fix-root-replay/`及change `verification.md` |
| 类声明上的注解使用 | 修前完整Java 8三类中`@Deprecated`与`@Retention(RUNTIME)`被省略，运行变成`false`和`null`；修后三套完整源码反射三行相同，CLASS-retention 注解在原/修后 class 属性中类型和值相同且反射仍不可见 | 复用现有`element_value`读树与`MemberDefault`拼写，在同次类读取中保留物理属性壳并按需读内容；重复/不可拼写条目整条拒绝，字段/方法/参数位置另案 | `recover-class-annotation-uses`生产与root独立验收完成：冻结CLI、复制重放、Rust/Java定向回归及严格规格见`class-annotation-uses/implementation/`和change `verification.md`；既存批量计费漂移单列 |
| 字段、方法及参数声明上的注解使用 | 修前三类原/JADX反射为`true / true / 1 / 5`，Jarde源码可编译却是`false / false / 0 / 5`；宽槽、varargs和不可见注解边界合法，两个受控属性补丁仍通过JVM验证 | 复用类级注解读树和有界拼写；成员各用自身属性壳，参数按descriptor位置而非JVM slot对应；参数计数不符及同位置重复时拒绝猜测 | `recover-member-annotation-uses` 已经 root 验收：最终 CLI 三套完整类 Java8 编译与验证同为 `true / true / 1 / 5`，不可见注解最小类重编译字节相同；boundary Jarde 仍因修前已有的正文缺返回而不能整类编译，见 change `verification.md` |
| 字段类型、方法返回类型及参数类型的 TYPE_USE 注解 | 429B合法 Java8 类的三处 `RuntimeVisibleTypeAnnotations` 有 FIELD、METHOD_RETURN、METHOD_FORMAL_PARAMETER 目标；原始反射为 `field / return / parameter`，JADX完整类及 Jarde 单独未编辑 subject 对照均为三项 `null`；Jarde完整集另因 runner helper 缺 return 而不可执行 | 注册表认识属性名但 reader 未读 `target_info`/`type_path`；复用注解值树并在限定引用类型名内部拼写可证明的 type-only 位置。双目标标量前缀会同时产生声明/类型属性；受控 primitive type-only class 已被 JVM 验证接受，但相同 `@Target` 的 `@A int` 源码无法单独表达它，应保留原事实并拒绝 | `type-use-annotations/`、`placement-boundaries/`、`primitive-boundaries/` 均由root独立复制或编译重放，三方/隔离与源码位置证据一致；`recover-qualified-type-use-annotations` 四份规格 strict 通过，root 最终 CLI 独立重放与定向验收完成；隔离 subject 三处反射恢复，完整类仍因 runner 缺返回不能编译，见 change `verification.md` |
| 接口字段的非常量初始化式 | 771B合法 Java8 接口的三处运行时初值原/JADX整类编译且执行同为 `ABT\|A1\|B2\|4\|7`；Jarde三字段缺 `=` 且输出非法接口 `static {}`，javac四错。字段表换序后 Code/运行不变；限定名前向读取样本原值 `L|0|9`、JADX重编译 `L|9|9`。新增常量阶段反例原class `0|9`、JADX重编译 `9|9`，两字段原本均无 `ConstantValue` | 必须跨字段与 `<clinit>` 做有界整组证明；字段表重排须保留前向读取默认值，运行时写入还须防止源码编译后升级为 `ConstantValue`；沿既有结构化恢复接缝，不需新 reader/通用初始化 IR | `interface-field-initializers/` 原/换序、额外效果/重复写/分支、前向读取/异常表及 `constant-phase-boundary/` 均经root独立复制重放；`recover-interface-field-initializers` 四份规格更新后 strict 通过，1.5已验收；永久 fixture 9 个 Java8 class、24 个 Code、正例与两项受控边界经 root 重编/验证及 corpus 指纹冻结，生产投影待实施 |
| 枚举常量与用户静态初始化 | 两组自写 Java8 双常量 enum 原/JADX 完整类及 runner 均可执行，Jarde 均把 `ACC_ENUM` 常量写为普通字段，javac 在此唯一报错；第二组 `<clinit>` 同时含编译器前缀及用户 `totalUnits = sumUnits()`；name/ordinal与`$values`/`valueOf`受控变体证明按名字或flag吞并成员会改行为 | 常量字段、构造器、`<clinit>` 与辅助方法需要一个类级有界整体证明；单方法报告不保留 IR，恢复报告不保留 AST，因此必须在现有分析/发射接缝携带私有有界事实，不从 Java 文本反推；不需新 reader 或通用枚举 IR | `enum-declaration/`四组经root复制独立重放，包括`helper-constructor-prefix-boundaries/`五种模式；`recover-proved-enum-constants`四份 OpenSpec strict通过，1.2已闭合、生产待实施；旧活跃 spec 已明确仅约束单方法正文 |
| 非覆盖型 `try/finally` | 593B/3Code普通类原/JADX两行一致而Jarde缺return；947B/6Code类证明返回快照，显式cleanup throw 是覆盖型负边界，JVM有效副本/范围变体改变trace；787B/5Code调用隐式抛错类原trace `29/19`、JADX一处错成`299`；907B/6Code覆盖完成类中JADX另有两处错副作用 | 现有guard/Region/Try承载结构；需要新增有界清理副本等价、每出口一次及保存返回/同对象重抛证明，不需新IR/通用等价机制；覆盖return/throw另案 | `recover-proved-finally-cleanup`四份规划strict通过，四组source-only证据均经root独立复制重放；先于生产修复保留catch-all拒绝 |
| `synchronized` 体内两条返回出口 | 646B/4Code合法类原/JADX三行验证执行相同，Jarde六处引用、完整类缺return；两个正常`monitorexit`与一个异常出口由两段保护区间共享handler | `guard::monitor`目前只认一正常加一异常出口，`Plan::body`与builder为平坦指令范围；已有`Region::If`能表达分支，但需证明各臂出口归属和在已证handler边界下递归；局部 certified guard-body 机制确有必要，不能只放宽exit个数 | `synchronized-multi-exit/`经root独立复制重放；`recover-synchronized-multi-exit`四份规划strict通过，负边界待补，生产与当前复合赋值/非覆盖finally串行 |
| 浮点常量 | reader保留bits，但java层未表达float/double字面值 | 忠实常量节点即可；有限值可精确十六进制拼写，NaN不能统一归一 | `recover-floating-point-constants`规划strict通过；36行基线、536项拼写验证及七种精确池/Code补丁经root冻结CLI重放：原class均通过JVM验证，JADX四种payload与真实取负改变raw bits，Jarde当前七份整类均编译失败；1.2已闭合，生产待实施 |
| lambda 合成名字 | 第二个捕获lambda形参与debug局部重名；修正后整类还与javac合成方法名冲突 | 非槽名字须统一候选检查；同类lambda实现方法的声明/引用策略另案分析 | `synthetic-names/analysis.md`保存原/JADX相等、jarde0引用但javac失败及明确方案实验；不混入字段/顺序主体 |
| clinit 全路径抛异常 | 恢复的无正常出口静态块被javac拒绝，JADX同样失败 | 声明上下文需要遵守Java正常完成规则；可复用if/block，不能仅凭无return判断 | `throws/grammar/analysis.md`保存两类实测及明确标注的恒真if方案实验；独立债务，未并入throw/字段改动 |
| 延期值跨独立语句 | 初版复合表达式15项曾有11项错值/异常顺序，且0引用、javac成功；修订后四组66项、复合15项、嵌套5项与内联/guard对照均经root逐项重放 | 共享位置证明跟随组合表达式到最终求值位置；复用Declare/Local，拒绝边界完整引用 | `preserve-deferred-value-order`已由root验收，908c冻结CLI及完整对照见change verification；独立lambda适配RED另案 |
| 泛型目标的方法引用 | 普通javac输入6项中3项错值，0引用且完整javac成功；漏掉String检查并错选Object重载 | 复用Lambda/Cast保留擦除SAM、动态检查与实现参数类型，不新增泛型系统 | `preserve-lambda-descriptor-adaptation`规划已strict；完整三方证据在`generic-method-references/` |
| 即时函数式调用接收者 | 原class/JADX对数组构造器与`Math::abs`即时调用均可编译执行；Jarde两类零引用却把lambda/方法引用裸接`.apply`，整类javac分别拒绝；先存有类型局部的对照三方执行一致 | 工厂descriptor已提供目标类型，只在实际调用receiver位置复用现有Cast并核对接口owner；无需新AST、数组专用机制或第二次恢复 | `immediate-functional-receivers/`三类经root独立重放，两个仅用于论证的手写cast控制均编译且执行等于原class；`type-immediate-functional-receivers`四份规划strict通过，生产待实施，合成helper名冲突独立 |
| 类型限定符被局部遮住 | 两份普通javac完整类12项中10项错目标；冻结fixture另8项中6错，均零引用且编译成功 | 复用现有NameTable reserved，按本方法静态owner预占类型路径首段，无需新resolver | 规划strict通过，fixture已由root独立三方复核；Rust RED和语料冻结待Cargo窗口，生产串行排队 |
| 丢弃后接静态调用的限定值 | 修前主类将`receiver()`调用1次变2次；接口static合法原类反编译后javac失败；受控Methodref补丁另证静默错绑 | `Invoke;pop;invokestatic`且唯一消费时复用弃值调用语句；其它限定值需验证Java类型/池属主，接口static禁止实例限定；不造新AST/pass | `preserve-popped-static-qualifiers`生产与root独立验收完成：重建冻结CLI复跑四个完整类原/JADX/Jarde全同；异属主补丁有来源保守拒绝，census/fingerprint/fmt/strict通过，Clippy与type-qualifier既存债务见`verification.md` |
| 基本数值转换 | 修前15个conversion opcode均未恢复；三个完整类180行与独立转换链73项为原始基线，JADX分别有13项重载及7项链式错误 | 复用单条转换事实与现有Cast，保持每个中间转换及转换后静态类型，不扩张无显式转换的窄局部/ireturn | `recover-primitive-conversions` 实现已由root冻结 CLI `8b86c729…756e9` 独立重放：180/180、73/73 与永久fixture 41/41逐行一致、零引用；合法 boolean descriptor 保守拒绝，`fconst_2` 相邻边界单列；2.4 的output/IR/取消及source-map专项通过，工作/深度/来源维度仍待独立证明，见 change `verification.md` |
| 窄整数局部回读与返回 | 普通javac类20项原/JADX一致；字段自增窄返回另有零引用编译失败；真实stack-join switch 的 B/C/S descriptor 变体均合法却引用 | 复用真实ireturn返回位置Cast，闭合普通/同步/switch/自增返回，不扩张局部范围推断 | `recover-narrow-integer-returns`规划strict通过；实际stack-join与boolean操作数拒绝变体另有独立证据，永久回归未完成 |
| 窄整数数组写入 | 历史760B核心类147项原class有效、旧Jarde 9引用、JADX完整javac失败；含Z的196项独立存档 | 已在`array_write`按真实store opcode与数组元素事实复用Cast；Z最低位仍另案 | [`recover-narrow-array-stores`](../../changes/recover-narrow-array-stores/verification-root.md)已8/8：当前Jarde完整类零引用、147项值/异常与另24项数组/下标/值求值顺序均和原JVM一致；Z、boolean操作数及未知B/Z合法输入保持来源可见拒绝 |
| 窄整数字段写入 | B/C/S 267项永久样本与285项核心已由root冻结CLI重放并逐行一致；含Z完整380项中Z仍70项不同，JADX整类编译失败 | 在putfield/putstatic已验证消费位置沿用field_value/Cast，保留producer与null先后；Z低位另案 | `recover-narrow-field-stores`已完成实现与独立运行验收；Z可编译空操作债务单列，见verification |
| boolean字段整数写入 | 修前完整380行中Z 95行有70行错值/漏调用/漏异常，最小40行有26行不同；root修后独立重放两组均逐行全同、零引用，JADX最小补丁整类仍编译失败 | 只在已证明Z字段消费位置用现有余数和比较表达最低位，直接put与verified accessor共用，普通boolean/未验证accessor保持原边界 | `recover-boolean-field-stores`实现与root验收完成；reader105/694、fingerprint新增8项和strict54/54通过，窄整数返回独立RED见verification |
| 只分配前缀维度的数组 | `new int[n][]`、`new int[a][b][]` 等69项原class/JADX逐项相同；修复前jarde18引用且整类编译失败 | 在现有NewArray保存总rank与已分配rank，写出尾部空括号；不增加数组推断层 | `recover-partial-array-allocations` 根验收：69项jarde零引用、整类编译执行与原class/JADX一致；重载数组协变的72项版本仍是独立边界 |
| 数组 `clone()` 返回类型 | `int[]`、`String[]`、二维浅拷贝、`String[]→Object[]`、重载及null六项原/JADX/jarde整类均编译运行逐行相同 | 真实array-owner `clone:()Object` 后的 `checkcast` 已由现有转换表达式保留，重载与NPE正确；不需要克隆专用机制 | `array-clone/`保存冻结CLI三方重放、670B/8Code类与全部六行结果；正面对照，无新change |
| 非空数组初始化器 | 原class/JADX完整14项相同；修前jarde38处引用、四个返回方法令整类javac失败；空数组已恢复。另有1741B/15Code合法边界类，JADX把协变`aastore`错折成不可编译初始化器 | 对单一分配身份、常量长度、连续索引与类型建立块内有界证明，在已有NewArray呈现元素序列；数组逃逸、跨块及可能改变`ArrayStoreException`的折叠必须拒绝 | `recover-array-initializers` 在转换实现后由root用冻结 CLI 独立重放永久fixture：整类编译验证成功、零引用、14/14 行值/trace/异常逐字相同，3.1已验收；补充边界11项相同，协变fallback仍不算语义等价，见 change `verification.md` |
| 条件表达式的栈汇合值 | 最小424B/6Code原class/JADX两项相同而jarde缺return；完整16项有七个条件值方法缺return，真实整类javac失败 | 在既有Region::If与SSA Phi上证明双臂唯一join，增一个必要的条件表达式节点，保留未选臂不执行 | `recover-conditional-values`独立规划，三方根重放见`ternary-values/root-7747/`，生产串行排队 |
| 字符串switch的源码形态 | 635B/10行原class、JADX、jarde完整类均可编译执行且逐项相同；jarde输出hashCode/equals加整数二级switch，而JADX还原`switch(String)` | 行为已正确；恢复语法需在现有switch结构上有界认领同方法内的字符串分派，并扩展case标签类型，不能只替换文本 | `recover-string-switch`四份规划strict通过；876B/18行扩展样本另发现第二级普通switch穿透拒绝，归既有2c.4任务，二者分开实施 |
| 普通switch非数值顺序穿透 | `lookupswitch` 394B/7行中case9穿透case1，两行91错成90；独立`tableswitch` 390B/8行中case4穿透共享case1/2，两行41错成40；两者原class/JADX逐行相同，jarde整类可编译运行却均留一处引用 | 先在region证明前臂只落入后case入口，后臂独占块；按真实入口BCI而非key数值顺序发射，才可省略前臂break | `present-proved-java-structure` 2c.4已补两种switch及共享标签门禁；`switch-fallthrough-order/`保存整类三方实测，字符串二级switch是独立交叉证据 |
| do-while体内跳转 | 普通及带调用的条件三方整类可执行；498B核心7项原class/JADX相同，jarde对跳闩锁/出口两方法引用且整类缺return，root复跑同结果 | 区分头块的体内if与头测；仅证明本层唯一出口时用最小break语句认领转移桥，保持条件次数/来源 | `recover-do-while-body-transfers`四份规划strict通过，当前是区域归属缺口，待永久夹具和串行实施 |
| enhanced-for数组与Iterable | root独立重放：`int[]`/`Object[]` 两类原/JADX/jarde整类编译运行各6/7行全同，jarde用下标while；`Iterable<String>` 原/JADX执行10行仅JDK helpful-NPE归因文本一处不同，jarde因hasNext条件调用拒绝、整类缺两处return；Object体内嵌if另触发既有LoopShape | 数组行为已闭合，不从相同字节码猜foreach源码糖；迭代器复用现有while+调用条件，归已列2c.6，嵌套if归循环区域债务，不新增foreach节点 | `enhanced-for/root-replay/`与`root-replay-object-if-red/`保存root三方重放及区域RED，冻结CLI哈希不变；无新OpenSpec change，继续既有2b.6/2c.6任务 |
| Class类字面量 | 851B/7项原class/JADX完整执行一致；修前jarde 11引用且整类缺return；root冻结25bf重放后jarde零引用、未改整类编译运行七行全同 | 在现有常量值→表达式链加受限`T.class`，真实BCI/CP来源；非Class、非ASCII、`$`歧义仍拒绝 | `recover-class-literals`实现与root独立验收完成；四份额外命名边界class已入永久fixture，Unicode/跨类解析及基本类型`.class`形态分开 |
| Java `assert` 编译协议 | 1346B完整类原class/JADX在`-ea`/`-da`逐行相同；Class字面量实现后jarde引用4→1，`<clinit>`的BCI13栈Phi未写入合成final字段，整类仍javac失败 | 条件/消息方法已有if/throw形状；剩余是已证明双臂0/1值在真实`putstatic Z`消费位置呈现，复用条件值与既有final字段左值规则，无须先造assert专用AST | `assert-syntax/root-after-class-25bf/`保存root重放；已纳入`recover-conditional-values`的字段消费者及`-ea`/`-da`验收任务 |
| null初始化的try-with-resources | 670B最小类原class/JADX整类可编译且执行相同；修前jarde把 `aconst_null; astore` 当普通赋值、合成Throwable当用户catch，生成`Object.close()`导致整类javac失败 | 在已有TWR守卫内有界准入null头并复用完整关闭证明；资源类型仅由当前类直接AutoCloseable声明与正常/异常close同属主共同证明，不能依赖未读取的StackMapTable | `recover-null-resource-headers`实现由root验收：冻结最终CLI下普通/三资源/null/正文抛错及普通用户catch整类均可编译、执行等于原class；损坏抑制与继承接口保守引用。`root-after-null/`留三方证据；手写无抑制清理里的普通null局部仍输出`Object.close()`，独立类型债务 |
| 数组上溯调用与重载目标 | 源码独立核心28项原class/JADX一致；`int[][]→Object[]`、`String[]→Object[]`、`String[][]→Object[][]` 三处被jarde拒绝；无checkcast局部上溯另有更具体重载 | 用数组组件的封闭子类型事实准入，在调用处沿用Cast固定Methodref参数类型；不扩张任意类层级 | `preserve-array-invocation-widening`规划strict通过，28项核心与目标选择样本已实测，生产待串行实施 |
| 移位表达式 | 基本723项和long距离转换200项原class/JADX一致；jarde31/13处引用且javac失败 | 忠实建模六个shift opcode、已有Binary扩展与左值独立提升，不等同一般二元数值提升 | `recover-shift-expressions`四份规划strict通过；root-core隔离位运算后的723项另测，排在错值问题之后 |
| new与引用消费者组合 | new→cast/aastore仍被构造认领名单拒绝，new→field/call已可呈现 | 复用既有消费者与唯一消费证明，无需新AST；不能无条件合并不同阶段的名单 | `construction-consumers/analysis.md`保存14项原class/JADX相等、jarde11处引用；先修已证实的延期错序，再拆分有限组合任务 |

没有把位运算、浮点常量、汇合值、自增和区域债务带进取负修复。Atlas 的缓存符号位置与当前未提交文件不一致，本轮关系只作导航；结论与修改位置以实际文件和编译样例复核。

基线 CLI SHA-256：`48f53c503a0656e5bfeffd29052000cc3aa8063f872bcfa6a0f56f8590bdf2b2`。CastProbe.class SHA-256：`a2c51d4eb2ee57489902340f22140f3831f79c128601b845f5dd7b92c90e4528`。debug 可执行文件在本轮改动前已存在；本轮会重新构建后执行验收，不能以旧 binary 代表之后的源码。

## SpecialProbe：结构化结果中的真实错值

永久审计快照在 `special/`，包括源码、javap、jarde/jadx 文本及三份执行输出。原始程序返回 `value=8`、`defaultCall=11`、`own=7`、`other=7`。jarde 生成文本重新编译后，前两项都是 StackOverflowError；jadx 生成文本的 defaultCall 返回 7。class 与 jar 入口的 jarde 文本相同，均报告 6 个 structured/java，verification 仍是 not_performed。

源代码明确区分 `BaseProbe.value` 与 `DefaultProbe.super.value`，对应池项也分别是 MethodRef 和 InterfaceMethodRef。修复不能只把 this 改成 super，否则会复制 jadx 本例的错值。`MethodIr` 已持有同一读取的 ClassFacts，可以借用直接父类和接口声明，不引入全类路径搜索。详细实现及非目标见 `../../changes/preserve-special-call-dispatch/design.md`。

主代理独立补充的 `special/independent/` 包含带包名接口、void class/interface super、实参副作用与其它实例 private 的七项执行对照。CLI 完整输出直接重编译，七项均与原程序一致。随后 `special/refused-producer/` 的合法 JVM 反例揭示拒绝路径遗漏延期实参调用 BCI 1；最终输出补全为 `7 4 1`。主代理复跑 17 项相关集成、168 项 Java 包测试和 JDK 执行对照均通过；严格 clippy 的既存区域债务仍单独保留，未称全仓全部验收。

`survey/` 保留横向 SyntaxProbe 的 14 个语法方法及构造器。debug 基线是 2 个完整结构化方法、13 个 fallback；其中普通数组读取已恢复，不能引用旧 release 的结果把它重新登记为缺口。比较、循环调用、位运算、数组初始化、throw 与浮点常量仍分别登记，不与本轮两项混合。

## CastProbe：复用已有 AST 的缺口

文件在 `casts/`：`CastProbe.java` 是源码；`javap.txt` 是指令；`jadx.java.txt`、`jarde-before.java.txt` 是同 class 的反编译文本。可重放：

```sh
mkdir -p /tmp/jarde-cast-replay/classes
javac --release 8 -g:none -d /tmp/jarde-cast-replay/classes openspec/evidence/java-syntax-2026-09-22/casts/CastProbe.java
target/debug/jarde-cli class-source --input /tmp/jarde-cast-replay/classes/CastProbe.class --class CastProbe --policy single-class --release 8 --format text > /tmp/jarde-cast-replay/jarde.java.txt 2> /tmp/jarde-cast-replay/jarde-report.txt
jadx --no-res -d /tmp/jarde-cast-replay/jadx /tmp/jarde-cast-replay/classes/CastProbe.class
```

| 方法 | 源码 / jadx | jarde 基线 | 判定 |
| --- | --- | --- | --- |
| direct | `return (String) x` | 引用 BCI 1、4，拒绝不是 bridge 擦除的 cast | 可覆盖但尚未恢复 |
| receiver | `return ((String)x).length()` | cast 导致接收者与返回一起引用 | 可覆盖；验收括号与 null/CCE |
| array | `return ((int[])x)[i]` | 数组表达式存在，但操作数 cast 拒绝 | 可复用已有 Cast/Index；不要新数组规则 |
| referenceArray | `return (String[])x` | cast/return 引用 | 目标类型须按数组类描述符拼写 |
| widen | 先存 String 局部再返回 Object；jadx 合并成直接 cast | cast 的 store 引用后，局部使用也被引用 | 无需仿照 jadx 合并局部 |
| twice | `(String)(CharSequence)x` | 两次检查均拒绝 | 必须保留两次检查的顺序，不能按最终类型消掉前一次 |
| invocation | `(String)f.get()` | `arg0.get();` 仍有语句，cast/return 引用 | 当前生产者没有丢；恢复后调用只能执行一次 |
| catches | try 中 cast 后 length，catch CCE 返回 7 | catch 保留，保护块整体引用 | 另有异常块准入边界，不以单补 cast 宣称此方法完整 |

`CheckCast` 在 `decode.rs` 已保留池类型；`build.rs::render_value` 仅在 `bridge_owns` 时擦除 cast，其余返回错误。建议将普通检查接到现有 `ExprKind::Cast`，同时保留 bridge 已证明的擦除路径。这里不需要 callee Body、类型闭包、动态验证或新 pass。

需要的反例：错误类型产生 ClassCastException、null cast 本身不抛异常但后续调用可能 NPE、嵌套检查按字节码顺序执行、被丢弃的 cast 结果仍不能丢掉检查效果、无法呈现的调用/字段生产者仍有语句或完整来源引用。`catches` 的保护块准入是独立债务，禁止顺手重写区域。

## 一元取负独立验收

主代理复跑 43 项相邻表达式/旧值/拒绝/数组/调用回归、93 项库内测试、1 项 JDK 数值边界与效果对照，全部通过。`negation/` 的额外 NegAudit 是独立构造的 8 个方法，包含分支、循环、concat、装箱、局部存储和复合浮点表达式；实际 CLI 整类输出从 20 处字节码引用降为 0，直接重编译执行结果与原 class 一致。没有手改恢复方法来使测试通过。详见 `../../changes/recover-unary-negation/verification.md`。

## 独立登记的仓库门禁债务

- 严格 `cargo clippy -p jarde-java --all-targets --locked -- -D warnings` 被既有 `region::try_level` 四项 tuple 返回类型的 `clippy::type_complexity` 阻断。该函数属于本轮前的区域工作，取负只给既有纯值列表增加 Negate；不为掩盖 lint 添加 allow 或混入区域重构。命令行暂时 `-A clippy::type_complexity` 的针对性检查不能记作严格门禁通过。
- special 最终独立复跑时全仓 fmt 通过（`/tmp/jarde-final-fmt.log`）；随后 cast 最终检查遇到新 invocation/numeric 测试的格式差异，各文件由其任务收尾。全仓格式状态会随新工作变化，不由专项语义测试推断。
- reader 的 class census 原钉 63/295/55/150/8，在本轮开始时已有其它 fixture 未计入。冻结取负+special 的第一阶段实测为 85/417/74/197/8，cast 加入后为 86/431/74/197/8，两阶段均钉住并复跑通过。该阶段指纹 5 passed / 1 ignored；比旧快照增加的 45 个条目中本轮三项占 11，其余 34 是既存清单缺口，不能全部归功于本轮。numeric/invocation 后续新增两类尚待冻结后集中再验，当前不引用该旧数值声称全仓已过。
- 引用转换实施时发现 `pop; invokestatic` 的静态限定符识别缺少类型/成员证明，可把被丢弃的 String cast 挂到另一类的 static 方法。cast change 只收紧自身准入，检查保留引用、调用独立呈现；一般 qualifier 类型证明另案处理，不引入继承解析器。
- 独立 CastAudit 初版含静态字段初始化，生成的 `static { ... return; }` 经 javac 实测拒绝（`casts/independent/with-clinit/`）。静态初始化块终止语句是另一个呈现缺口，不混入 cast；主 cast 输入改为 driver 初始化后重新编译原class，完整恢复类的23行运行结果已与原class一致。
- null资源头验收的同类型手写清理反例把普通 `NullResourceCore resource = null` 局部写成 `Object local0`，随后 `.close()` 令整类javac失败；这是一般局部声明类型缺口，不能凭TWR头的关闭证据替所有null局部推型。另一个独立诊断细节是 `jre_guard_suppressed` 对缺失 `addSuppressed` 调用仍表述为“receiver错误”，虽拒绝种类与来源正确，文案应在后续诊断任务单独校准；两者均不混入 `recover-null-resource-headers`。

这些是可见的门禁状态记录，不把问题藏到语法任务里，也不以它们推翻已运行的具体行为对照。

## cast 主代理最终行为验收

2026-09-23：Java包168项、eval-context10项、reference-cast5项及1项ignored JDK通过，另40项相邻回归通过。source map实际请求并校验转换、操作数、返回及拒绝消费者BCI；旧R9改为合法的丢弃转换结果内存变体，保留字段效果回归。额外CastAudit整类实际编译的23行与原class一致，jadx删除无用局部检查造成3行错误。具体日志与限制见 `../../changes/recover-explicit-reference-casts/verification.md`。

## 调用参数主代理独立反例

`overloads/independent/` 在修前即可完整重编译jarde整类，但12行有3行错值：super/this构造器均1→2，多参数顺序仍为12而目标返回41→42。原始Object/String同型局部、原有显式cast与null/CCE为稳定对照。jadx的super同样1→2。修后重新构建debug CLI，实际整类重编译的12行全部相同；`self-argument/`额外验证 `(Object)this` 选择Object重载，输出1。初版裸throw的独立拒绝保留在该目录with-bare-throw，没有删生成正文绕过。未知接口反例也已用新CLI复测：原patched class通过验证且返回7，jarde明确拒绝Object→Runnable，没有凭空加入运行时cast。

## 第二阶段语料冻结与初始化规划

2026-09-23，两组新class冻结后，reader实测并重钉 `(88,480,74,227,8)`：新增49方法体、30分支目标，无新增异常项或子例程。fingerprint从197到205，8新增/0修改/0删除，reader census通过，指纹5通过/1忽略。此状态仅验证输入集合；numeric和invocation实现验收尚未完成。原始日志在本目录corpus/。

`recover-static-initializer-completion` 的proposal/design/spec/tasks已通过strict。root核实StaticConditional、StaticLoop、StaticTryCatch原本已呈现if/while/try-catch，return确实是最外层尾句；无需扩大区域准入。生产实现仅改共用emitter，20个发射单元测试通过；`static-initializers/root-after/`的6个整类7次运行与原class一致。随后source-only边界审计验证空块仍为0、外部helper首次失败为EIIE而再次访问为NCDFE；合法提前返回两路径原class为1/7，jarde仍含非法内部return而编译失败，未误宣称本项覆盖该控制流。

`initialization-final/` 新增独立实证：普通blank static final的限定赋值被javac拒绝，ConstantValue和实例final初始化为对照。jadx完整生成类的4行执行与原class相同。这个字段名字绑定问题单独排队，不混入只处理末尾return的初始化change。

`instanceof/analysis.md` 记录10方法×8输入的80行对照：原class与jadx一致，jarde有22处引用且实际javac失败。直接测试需忠实boolean表达式及既有bool证明叶子；否定值另受0/1汇合限制，不把两项混成一个机制。

## final 写入扩展审计与普通 throw 的边界

`initialization-final/expanded/` 用五个类、六次执行区分了两个缺口：四个普通类的限定 final 赋值需要合法简单名，`FinalLocalCollision` 还证明必须避让恢复出的 local0 及既占后缀 local0_2；接口则缺字段声明初始化且不能有 static 块。原 class/JADX 六次结果一致，当前 jarde 完整输出全部按实际 javac 失败记录。普通类的四份 OpenSpec 已通过 strict，接口初始化归属另案处理。

`throws/discarded-stack/` 的合法 patched class 让有副作用调用的返回值留在异常对象之下，原/patch 均经 `-Xverify:all` 得到 `identity=true:calls=1`。SSA 记录的是 athrow 真正读取的异常值，清空剩余栈并不增加读取；因此普通 Throw 不需要新的栈清理消费机制。普通 throw 的四份 OpenSpec 已通过 strict；带 guard 的 finally/synchronized 仍按原模式边界拒绝。

## 新一轮独立执行基线

`comparisons/independent-flow/before/` 是另一个完整类的242项数值对照，包含NaN、long边界、嵌套分支、double while和左右调用抛错。实际JADX有11个NaN错值；当前jarde整类有10处引用并被javac拒绝。修后必须原样重编译完整输出，不能仅验证比较字符串。

`throws/final-fixture-before/` 记录root对新ThrowProbe的独立验收：源码复编译与946-byte冻结class逐字节相同，9个Code方法；原class/JADX 13行一致，覆盖异常身份、null、调用次数和producer自身失败。当前jarde整类仍javac失败。新的throw及final字段夹具完成后另一次集中冻结census/fingerprint，旧88类数值只代表上一阶段。

`comparisons/independent-flow/after/` 已用数值比较实现后的 debug CLI 复跑：完整输出从10处引用变为0，原样javac成功，242项结果全部与原class一致；JADX仍有11个NaN错值。`lower-stack/`验证比较结果处于非零栈深度仍可恢复，其原样单方法重编译2/2一致；`old-local/`则验证保留在栈上的旧long值跨槽写入时，现有最终求值位置检查明确拒绝错误重算，保留比较、分支及旧读取来源。

`initialization-final/final-fixture-before/` 固定另一份完整类基线：1089-byte FinalStaticProbe.class 与源编译逐字节相同，SHA256为2bdb603ff8b3638d48256163fb729a0d5ba34a07a65161221b339ecb7ce59a45。原class与JADX在两个新JVM分支中的8行输出一致，jarde完整源码javac失败。后续只修受证明的final写入左值和名称避让，不把合法的限定字段读取也改成简单名。

## 新夹具与函数式适配审计

`deferred-evaluation/fixture/root/` 独立重放14次完整Code补丁，1338-byte/19Code/hash与冻结输入相同。runner每例重置状态，并补齐null数组与空数组异常；当前83行原class/JADX一致，jarde0引用、完整javac成功但48行不同。这是永久夹具的输入验收，与前述69项独立审计分开计数。

`floating-constants/final-fixture-before/` 独立确认1722-byte/29Code输入与源码字节相同，原class/JADX36行一致；jarde56引用且完整javac失败。浮点计划依赖共享值保存，以保留真实运行运算而避免Java常量折叠改变NaN观察值；不把手写候选当作已恢复。

`lambda-captures/` 的直接调用结果捕获探测被现有replayable检查明确拒绝，并保留生产调用，不是已确认的重复执行缺陷。`binding-hypothesis/`只证明未来可复用已声明局部来扩大该边界。`synthetic-names/` 的后缀冲突与整类lambda helper冲突仍分别登记，不混入参数适配。

`initialization-final/root-after/` 已用最终CLI 7527b03a重放四个普通类五次运行，与原class/JADX一致；`failure-after/`补充初始化正常/第一调用失败/第二调用失败及再次访问的六行一致结果。字段规划与名称扫描预算已经补齐，root独立181包测试、38相邻及1项ignored JDK通过，final-static最终验收已完成。接口仍保留独立javac失败边界，严格clippy仅有既存region.rs:1736。

`initialization-final/root-regressions/` 保存此次统一门禁：reader 94/579/75/236/8，fingerprint 232文件（新增13、既有219无修改/删除），fmt与39项OpenSpec strict通过。后续新增shift规划另行strict通过，不改写该历史计数。bitwise、floating、deferred、instanceof的Rust红测试均实际编译运行，输入冻结不代表待实现语法已恢复。

`shifts/root-core/`是移除源级xor/or后重新编译的独立ShiftCore，719 bytes、723项原class/JADX一致，jarde29引用且完整javac失败。原basic723项仍含位运算依赖；不能把这两份同数量但不同输入的结果混为同一基线。`shifts/boolean-boundaries/` 的合法descriptor变体24项原/JADX一致，固定boolean与JVM Int形状不能混淆的直接表达边界。

`generic-method-references/fixture/root/` 已独立验收2205-byte/9Code冻结输入，扩展source-only runner到27项，原/JADX全部一致。null receiver配坏参数先CCE而非NPE、constructor错误输入调用0次均有显式观测；jarde完整类仍按实际javac失败登记。新永久输入待Cargo窗口归还后统一冻结，不冒称当前census已含它。
