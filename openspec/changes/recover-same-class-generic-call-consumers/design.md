## Context

前置主线 d158989b 的最新 CI 37836010542 四项成功，含真实 JDK25 oracle。问题及冻结输入见 proposal 与 `openspec/evidence/published-generic-call-adaptation-patrol-2026-10-09/summary.md`。本片目标是普通同类泛型调用消费位里程碑，既有字段/构造器/raw receiver 片继续回归，不能把源码层改善当成 reader 或 JVM 支持范围变化。

当前 `GenericReturnCandidate` 只有参数、条件、void/null 与特定 allocation/functional 返回，没有 invoke 结果证书。`SameClassInvokeUse` 只有 caller 文本标签/BCI/目标与 constructor receiver；普通调用没有 SSA 实参/结果。`prove_same_class_method_binding` 只处理 Object 父类、无 interface、同名 arity 不重叠的身份证明。facade 的 deferred method commit 在 7872 附近，字段专用 published parameter table 在 7942，field commit 在 7959；现有逐成员出版能组合出 Object→T 错误。

`ClassSourceMethodAstSource` 已有同次 Program、物理方法、完整 Code/BCI 与 call targets、参数名，但默认普通 Signature 不保留 AST，且公开 opaque 接口没有调用类型投影。直接 constructor candidate 排除调用 RHS 和 EH，ExceptionHold 需要独立调用/初始化证明，不能取消旧 guard 冒充支持。Atlas scoped 结构缓存报告旧行号，当前磁盘内容及 root/Luna 三方实际审阅为准。

## Goals / Non-Goals

**Goals:** 见 spec。实现闭包以一个物理类的完整输入及真实源方法集合为界；直接 T/T[]、simple bounds、mixed primitive/宽槽、直接方法 binder 代换、有界无环 relay、constructor 的 Object() 后调用与结构化普通 catch、封闭重载上转型共同验收。保留 class/method binder 身份、raw 选择、真实 BCI 和全入边；类变量与方法同名变量不会被混并。

**Non-Goals:** getter 的字段→方法反向 publication、跨类/继承/接口目标解析、任意 `List<T>`/wildcard 推断、复杂 raw alias/phi 和局部泛型传播、this/super 委派、通用循环类型推断、全程序调度。既有 AST 支持的 T[] 与普通 catch 不作为一刀切禁区。引用类类型与已支持声明拼写可沿用，但不能因此承诺未知容器调用代换。

## Decisions

### 1. 最终 Program 是唯一源码形来源；扩展既有 opaque AST 与同次事实

条件保留候选声明及其同类调用者的 opaque AST，复用现有 emit/projection。Signature/同类调用清单决定需求，没有相关类型声明与调用的类不支付全量保留成本。精确 receiver/argument/result 分类从实际 Program 提取，物理调用身份、SSA 参数值与真实全部 uses 在已有 same-run Code/SSA census 交叉验证；不能从报告文本反解析、根据 AST presented 擦除类型或参数名猜 source 类型。

普通调用按完整 descriptor 参数顺序与栈值槽配对，不把 receiver 写死为某一深度；long/double 不能造成实参错位。物理 caller 使用 PhysicalMethodId，site 用 BCI/opcode/owner/name/descriptor。唯一 AST Call/New/constructor-init 节点与 site 必须匹配；synthetic accessor 折叠成 field、method reference/bootstrap、origin 多义等不能误用普通调用证书。调用结果只支持已封闭的直接返回、作为另一个已证明调用参数或 descriptor 可赋值的字段写入；多 use 必须全部检查，未知用途拒绝。

优先给 opaque AST 加最小带预算的查询/投影接口；若保留 SSA 生命周期需要小的非序列化 site facts，可在现有 ClassSourceRecovery 侧车添加。不得新建通用类型 IR/框架或 public RecoveryReport 字段。所有遍历/clone/BCI join/输出都计费并 poll。

实施补核：逐个 requested site 匹配不足以证明 census 完整。关联 caller 的 Code/CP 扫描所得同类 invoke keys SHALL 与其 opaque AST 已保留的完整 call_targets 在 BCI/opcode/owner/name/descriptor 上精确等量交叉核对。复用现有两路同次采集，提供一个窄的 inventory validator，不新增 pass、parser 或 IR；缺项、重复、目标差异或不完整 Code 为普通拒绝，预算/取消为 stop。空 expected 只有在实际同类物理调用集合也为空时才可通过。`IncompleteSite` 冻结源码是合法 conditional 单调用正例，缺失 census 控制必须用完整物理事实加故意缺项的合成负例单独验证，不改原输入凑拒绝。

### 2. 在既有 method→field publication 之间扩展关联事务

只做“leaf 已发布再逐个改 caller”不够：relay1(T) 的新入边可能仍来自 relay2(Object)，回退 relay1 而保留 identity(T) 仍不能编译。必须保存相关成员原始 descriptor 头/正文和已获证待发布头/AST，按真实调用依赖建立有界关联集合；关联的新头和因它必须适配的已有叶子一起准备，成功后一起最终发布。独立不关联的既有合法泛型成员不回退。

依赖边为 caller→callee，callee 的 body/erasure/binder 证书先获证再给 caller 使用；只能消费原本已发布类型或在本事务内已独立获证、准备共同发布的明确声明，不能读取被拒 raw Signature。有限类方法数上进行 callee-first 有界遍历，声明顺序不影响结果。待发布 contract 与最终会发射的 header 必须相同；所有相关 incoming invokes 在同一最终 header/body 集合下核验，不能把中间 methods 向量的一次修改视作对外发布。类型推断依赖环不借自身候选证明自身，环及依赖它的相关投影明确拒绝，不能不断重试直至猜到类型。

关联集合可复用现有 ClassSourceMethod staging/clone，局部索引复用 PhysicalMethodId；不要求建立长期 registry 类型。失败时同时恢复关联头/原始正文，再保留具体拒绝证据；预算停止先整组保持原始表示并传播既有 execution。最后一次原子提交前完成全部 charge，防止一部分方法被改后下一 charge 停止。与失败集合独立的投影保持。

字段专用 `same_class_published_method_parameters` 只收字段 writer/receiver，不能冒充 call registry。增加实际最终 callee 参数/返回摘要或复用其公共内部解析过程，但 field table 仍只在调用事务成功之后读取最终 methods。字段返回证明不反向用于 caller。

#### 实施审阅：关联 staging 只保存可变源码投影

v14 审计确认完整 `ClassSourceMethod::clone()` 会连同不参与投影的 `RecoveryReport`、注解与来源数据深拷贝，而输入数量 preflight 不能覆盖副本成本。关联事务 SHALL 改为保存必要的私有源码投影状态：declaration、text、markers、body proof、generic projected/refused 两标志与 constructor source-tail/erasure-refusal 两状态；原成员的物理身份、outcome 与注解保持唯一只读事实来源。原入口的签名解析、擦除、flags、声明拼写和拒绝门槛继续复用，不增第二套证明。

待提交 contract SHALL 对应其已获证待提交头，不能读取被拒 Signature；完整入边与最终 overload 重验之前只准备源码状态。成功与失败集合均在全部新文本/marker 收费后一次安装；保存投影前状态及暂存字符串/body proof 的遍历和复制必须先计费并 poll。预算/取消恢复使用已有状态的 move/swap，不在停止后分配或收费。此实现修正不改变 GC01～GC10 范围，不引入通用serde计费框架、报告IR或永久registry；完整矩阵与独立成员/双链停止测试继续保留。

#### 旧声明与预算停止的保留边界

关联事务只接收声明 method formal 或参数、返回、throws 中实际使用 TypeVariable 的 Signature。具体 `List<String>` 等没有变量代换的声明继续由原 ordinary/deferred 路径证明；新 caller 消费它们时只读取当前已发布、未拒绝的 contract，并继续逐调用验证 AST/SSA、唯一物理目标与 overload。constructor 自身 method formal 保留原独立路由。入口判定复用现有 Signature 解析和递归预算计费，不以类型名或固定 arity 分类。

Signature 准备和候选收集发生预算或取消停止时，沿现有 execution/diagnostic 路径记录 Partial，并移动恢复尚未提交的 overlay；不能将正常停止泄漏为顶层 API Err。共享 Signature 解码或缓存是另片架构债务，本片不提前全局解析，以免改变 malformed/duplicate Signature 仅拒绝头部投影的既有语义。

旧普通/deferred路径已实际投影的abstract NoBody声明，是通过原Signature擦除、flags、注解、名字、层级与闭集binding门槛的独立声明证书。事务可将其当前完整头作为callee leaf，继续验证最终incoming；不存在的Code/AST不构造body证明。新caller失败时退回原raw源码，保留这一不依赖新caller投影的既有声明。未投影NoBody仍沿原拒绝原因，不能因Signature存在就取得证书。

raw caller的唯一直接返回可用已验证单method formal/单普通Class上界，检查实际invoke擦除结果等于上界，再用已有封闭assignability核对上界到caller raw返回描述符的单向兼容。Number结果可交给Object或Serializable，不将T推断为Object。abstract声明leaf的本类直接形参receiver须实际发射raw形式、唯一AST形参名/descriptor槽/SSA load及BCI匹配；已发布参数化receiver、alias或继承关系不借此扩容。所有实际参数与结果消费仍逐项证明。

字段 receiver handoff 只有 `Prologues::answered()` 时才计费复制真实 InitRecord。普通方法与静态初始化器的 `NotThisRule` 是常量空 record，没有拥有的初始化事实可复制，不增加构造专用费用；原 ordinary/class-source 报告等价和局部作用域停止锚保持。

预算截断针对尚未提交的关联组：请求在后续阶段停止，不撤回此前完整提交的独立组。总usage减一只能证明请求Partial，不能声称停止在头部投影中间。验收须核对AnalysisSteps的真实execution/usage/诊断，并验证每个头与proof marker完整匹配；事务中途停止另由单链/双独立链与overlay预算测试覆盖。

### 3. 直接类型代换与赋值有独立 binder 环境

Signature parse、descriptor erasure、flags/annotations/name 与 RuntimeProfile 校验使用 reader 和 class_source 现有实现。变量身份由原物理 class/method 声明及变量序号决定；method formal 先遮蔽 class formal。直接 callee method formal 可从确切实参源类型获得一致代换，再逐个核对所有使用该 formal 的参数和返回；多实参不一致、bound 不成立、仅有擦除相同均不接受。无约束 `<U>` 可映射到 caller 的已获证直接 T，Number bound 只接受已证明符合 Number 的直接源，未知层级不猜。

数组递归保留维数/组件 identity，primitive 不参与 reference binder 推断。实际 receiver 为 this 或精确已发布的本类参数化形式时使用可证明 class formal 映射；raw receiver 的方法选择按真实 raw 规则核对，不自动套 this 的 T。复杂 receiver 重绑定/未证明 alias 保守。表达式 source 类型来自最终参数头或真实局部声明、获证 callee 结果，不从未发布 Signature 借用。

根补核 [JLS8 §4.8](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.8) 与 §4.6：raw 类自己声明的非static方法选择会擦除整个方法类型，包含 method formal；static方法保持泛型声明类型。不能仅因 callee formal是独立U而在raw instance receiver上进行U→String推断。`raw-method-binder-rule-control` 的原javac单腿前置诊断已冻结：raw instance `<U> U id(U)` 直接赋给String编译失败，保留物理 `(String)` cast可编译，static独立U经raw限定仍可编译；此处只证明原语言边界，不计反编译验收。[§4.10.2](https://docs.oracle.com/javase/specs/jls/se8/html/jls-4.html#jls-4.10.2) 的变量每个声明bound均为其supertype，Number上转型证明不要求其它interface bound也为Number子类。

### 4. 恢复 caller 的全部消费位，保留现有正确声明门槛

empty/void/direct return/relay 复用已有 candidate 与声明拼写，扩展一个真实 invoke 结果或完整 body 消费证书；不能伪装为 direct Parameter 返回。不同参数、receiver、null、原有 cast、字段写、后续调用和返回须在最终类型下核对。mixed primitive 不因全参必须 T 的旧窄形被误拒；未知 container/wildcard 的约束不是“Signature可拼写所以安全”。

Object() 后 call-result 字段赋值 constructor 从同次 InitRecord/Code/SSA 验证 uninitialized-this 与初始化先后，再验证全部相关参数读取、调用和赋值。ExceptionHold 沿实际完整 Try/Catch AST 检查正常/handler 中消费与写入；保留原 handler 顺序和抛出/清理效果，不移除 `generic_constructor_candidate` 的全局 EH guard，也不把新 body 当成可隐藏空 constructor。T[]、bound、多参数可以共用上述窄消费证明；getter 与任意 assignment/cast/phi 推断另片。

### 5. 在最终源类型下保留重载，cast 仅允许已证上转型

继承/interface/varargs/boxing/不可闭合目标保持拒绝。读取物理类完整同名 declaration 集，按实际最终 header 的参数类型计算已支持适用关系。对唯一目标保持原 Call；发生竞争时，只有目标 descriptor 参数可作为已证源码上转型、不会增加 checkcast、能使原物理目标唯一适用，才在相同 AST argument 位投影 cast。BoundOverload 的 T 明确有 Number 上界，所以 `(Number)v` 合法且无运行时检查；Object→T、Number→未知子类不在此许可内。

不能把 arity-only binding 全局放宽：对有竞争 generic 头的新许可必须附带完整相关调用的源选择证书。待投影的 overload 本身和调用者按最终事务类型共同核验。AST 重发射与 source map/physical origin 同时保留，类型 cast 不制造新的物理 checkcast 事实。

相关 overload 的独立声明不能靠另一重载的调用推断类型。若其参数是 `Comparable<T>` 等既有 Signature 已明确的类参数化类型，而完整正文不读该 formal，复用 `VoidBody` 候选携带逐物理槽位的未读证据。证据 SHALL 在同次完整 Code/SSA/effects、无 phi/参数写入的原候选生成处取得；逐 BCI/opcode/handler 与读清单必须一致，不能从缺项 sidecar 的零读数推断未读。声明仍经过原 Signature erasure、已发布 binder scope、flags/annotations 和 source spelling。已读的参数化参数继续需要完整消费证明；此许可不增加容器调用或 wildcard 类型推断。

### 6. 借鉴 JADX 消费位思想，不移植其猜测更新队列

已只读核对本地 JADX TypeUpdate.java:413–444、InvokeUpdateCallback.java:78–143、TypeUtils.java:238–260、MethodInvokeVisitor.java:102–175、TypeCompare.java:251–273。借鉴 exact invoke 参数/结果与 receiver substitution，借鉴以 compiler-visible types 选 overload；不移植可变 IR queue/fixpoint、未解析 T fallback 或 all-bounds 都 narrow 的比较器。BoundOverload 的四腿失败不揭示真实 IR 分支，因此只记录条件性风险。参考 TestGenericsInArgs、TestGenerics7、TestGenericFields、TestGeneric8 的实际断言边界，不宣称 Jarde 已通过这些测试。

现有 Rust 签名解析、AST/SSA、speller、Budget 已足够；没有外部库能替代本项目真实来源和 publication 证明，不新增依赖。仅参考算法/测试思想，不复制 JADX 源码；无新增许可内容。parse/erasure、Java8 source dialect、真实 JDK8/23 input compilation、JVM verification 和 reflection 分别记录，CLI 0 与 Structured 不是正确性证明。

## Risks / Trade-offs

- [关联事务漏旧 leaf 或 unknown incoming] → 冻结深链、逆声明序与合法同擦除/不兼容 binder 控制，逐全入边核验；失败不留下 Object→T 新错。
- [额外 retained AST 影响旧预算/普通恢复] → 需求条件化、计费；覆盖预算/cancel 和 class-source/plain 有关既有对照，不让不相关零 formal 静态初始化器付费。
- [实际 renderer 已将 alias 折成 this 或 setter折成字段] → 最终 AST 与 physical census共同匹配，后续 AST投影不得沿用失效证书。
- [catch 参数槽/多use可能混并] → 真实定义/全部消费者检查，完整AST递归收费，不以 EH 正文已恢复代替 type proof。
- [类型上转型误变成运行时检查] → exact declared bound/既有封闭 JDK关系核对，重编 javap 与异常/marker 验收；未知不猜。
- [失败集合回退侵蚀独立 API] → 回归不相连独立 generic leaf 和全部旧三片反射，按关联集合回退而非全部擦除。

## Migration Plan

main 原位实施，不新建 branch/worktree。Luna 负责已定边界的生产和冻结对照，root独立架构审阅及验收；共享 Cargo 仅 root/jobs1/incremental0/testthreads1，20GiB停建线，验收后清理。四腿冻结 baseline CLI hash、真实 JADX launcher/lib hashes、JDK版本、source/jar/result/probe hash，保留失败历史。原/JADX/baseline/candidate 独立全类重编，显式空 classpath/sourcepath，Probe runtime 只新 classes；不借原 jar、不删失败成员。marker、数组 identity、目标调用次数/异常、GenericDeclaration 身份分别断言。

完成本地 fmt、CI同口径clippy、两固定 seed、显式 ignored 门禁与strict OpenSpec后提交/推送main，再确认该最新HEAD真实JDK25四job CI，更新handoff和71单元中的局部状态；不将本片完成等同整个泛型单元完成。
