## Context

见 proposal.md。基线 `790579e2` 的 Hold 输出仍是 `Hold(Object)` 和 Object 字段，正文中的 `super(); this.v=arg1;` 已完整恢复。字段写证明已经使用实际发布方法参数：facade 的 commit 先结清 deferred methods，再采集 published parameters，最后结清 deferred fields。

Atlas 局部事实定位了 project_method_signature → generic_constructor_declaration；report 的 call-graph 查询失败，结构 detail 与源调用点 recover_inner 单独核对。Atlas 对 `bounds.is_empty()` 的同名推断不是类型解析事实，不据此推导依赖。

本地 JADX SignatureProcessor 解析类、字段和方法 Signature，校对擦除并交给类型处理；TestConstructorGenerics 实测的是 Map/HashMap 菱形，不是类变量构造参数。TestGeneric8 则断言非静态成员类 TestNumber<T extends Integer> 的 `TestNumber(T n)`；本片先闭合其顶层字段初始化子形，合成外部实例参数仍由成员类单元独立处理。复用 expandTypeVariables 按类/方法作用域解析参数并逐位校验 descriptor 的思路，不继承按名称补发缺失 binder 的宽松假设。新 fixture 必须检查反射 GenericDeclaration 身份。

## Goals / Non-Goals

**Goals:** 在既有构造器候选上形成一个完整字段初始化正文证明，覆盖 Object() 后零个或多个原参数赋值、末尾 return，保留原 AST/body 和物理 identity。构造参数与字段类型分别原子发布。

**Non-Goals:** 任意正文泛型推断、泛型子类型引擎、修改字段读位门、源级 cast 修补、raw receiver 类型、非 Object 继承/this 委派/异常体、throws/注解/嵌套恢复。这里拒绝的是未证明的泛型投影，不应改动已存在的正文恢复策略。

## Decisions

### 扩展单次候选，不新增分析阶段

在 report 的既有 GenericConstructorCandidate 内区分字段初始化和旧空体/转发证明；优先以新增必要的字段写事实集合表达区别，不建立全局 IR。字段事实仅记录原 BCI、owner/name/descriptor 与原参数槽。旧 forwarding/empty 消费者显式确认字段写为空，不能隐藏新正文。

提取以完整 Program/Code/SSA 为闭合域：单块无 phi、无 EH、未停止、无 ragged/fallback；第一条完整 Object() 前导与 InitRecord 对齐，最后 return。每个中间 AST 必须是直接 this.FieldAssign(=, 原参数)，每个物理段必须是初始化后 this load、原参数 load、putfield。同一原参数允许赋给多个字段，每个加载值只服务其对应写位。所有指令/SSA/operation/effect/AST 逐序闭合，重复 BCI、额外读写、重写局部、调用或 cast 一律无候选。

复用 descriptor 的参数位置和 NameTable，不按参数序号猜槽。参数局部定义必须为 Entry；接收者局部必须是同一 Object() 初始化定义后的 local0，不能把 UninitializedThis 或其它对象误当 this。参数/字段物理描述符在本片要求一致，避免引入新的赋值转换规则；宽槽按现有 Type/SSA 表检查。

### 物理字段与泛型字段分开

class_source 接收本次已读的物理字段表，对候选每个写位验证本类唯一声明的同名同描述符实例字段；不再解析字段 Signature，也不等待字段投影。已发表类 scope 才能为普通类参数提供 binder。reader 现有 MethodSignature 擦除/作用域证明及源码类型拼写处理 T、T[]、bound 等。

取消 `<init>` 仅限 method-level formal 的分流限制：存在构造器 Signature 即进入既有 constructor declaration 路径，仍需全部形态、scope、擦除和完成正文门。保留旧方法级空体/转发支路；新字段初始化支路仅处理无构造器形式类型参数、已发布类 scope 的参数（包括交叉类变量），不顺带恢复 ConstructorUses 的方法级赋值正文，也不得泛化非 Object 父类或继承形。所有 flags/throws/annotations/nesting/源名字原门保持。

方法先发布后字段消费，所以 `CrossHold<T,U>(U)` 可恢复 U，同时字段 T 安全擦除。强制 ctor binder 与 field binder 相同会丢失本来可证明的构造器反射；使用待发布字段 Signature 证明 ctor 则引入循环。两种方向均不采用。

### 构造器声明必须同时保持既有调用源码可编译

初次 root 全族重放发现 ThisDelegateHold 四腿回退：调用者仍发布 Object，而目标构造器先恢复 T，物理 descriptor/arity 绑定成立却不能证明 `this(Object, 0)` 可赋值。此结果保存在 results/initial-root，不计为验收通过。

在既有 SameClassInvokeUse 中保存该次 SSA 的构造接收者分类，复用 scan_member_uses 已有唯一 BCI 表：UninitializedThis 是 this 委派，Uninitialized/new-site 是独立分配，其余为未知。只对新增 class-scope 构造投影按目标 owner/name/descriptor 检查完整 incoming census；实际 this 或未知接收者保持物理参数并给出 unproved_this_delegate_call，明确 raw new 仍消费原绑定证明。未知接收者不污染无关成员的全局 census complete。Pending 仍沿原 deferred 阶段结清；不增加迭代发布、调用适配引擎或从 caller 名字推测接收者。方法级构造器旧路径不变。

本片不恢复 this 委派的泛型参数；必须同时拒绝会使未证明调用者源码失效的目标投影。整个冻结集合中基线可编译并行为一致的类，在候选中也必须保持，不以正例通过掩盖边界回退。

独立 new-site 的放行不等于从 SSA 推断原始源码 raw 类型。root 和独立 audit 核对了 build::new_expr 与 emit：本片既有 top-level（无 `$`）、Object 父类、无接口形会输出 raw `new C(...)`；AST 没有显式构造类型实参，两个 diamond 来源分别限定 member-inner 和固定 HashMap()V 泛型局部，均不能参数化本片目标。因此使用实际发射源码的 raw 构造选择与原 descriptor 参数处理证明兼容。若将来扩展这些来源，必须重新审查此约束。PeerNewHold 的 caller 含额外分配仍不满足完整字段初始化候选，只恢复二参数 callee；该类作为部分恢复边界，不能计为全反射正例。

### 发布状态仍以实际产物为证

继续使用 project_generic 的原子声明和输出预算路径，以及 generic_signature_projected 的真实成功标记。Settled/parsed/candidate 均不代表已发布。证据选择 essential/all 不改变文本或证明；预算/取消不可转成 unsupported 后继续发布。

### 复用和维护

不新增库、parser 或依赖，无新增许可证负担；现有 jarde-reader、jarde-jvm、jarde-java 满足解析、验证与候选提取需求。Signature 可解析、擦除成立和 JVM 可验证只分别提供各层事实，不能替代 Java 源码恢复。双真实 JDK 输入固定 major52，runtime release8 与编译器身份分开记录。

## Risks / Trade-offs

- 新候选误入匿名转发/空体消费者 → 消费者显式排除字段写，冻结成员/匿名族回归。
- 同名 T 或宽参数槽看起来相同 → 物理槽 + Entry/初始化 SSA 定义 + reader scope；反射核对 GenericDeclaration 身份。
- 新泛型参数使既有字段声明不可赋值 → 方法先发布，已有字段全写位证明随后保守擦除；不得改读位门或添加猜测 cast。
- 同类调用的 generic 参数适配是更广债务 → 原 call-binding 门完整保留，本片不放宽 overload/handle/不完整清单；fixture 的外部 Driver 不进 decompiler 输入。
- 单次候选读取更多指令产生资源占用 → 遍历前 charge 并逐项 poll；重复 BCI/停止无候选，验证边界预算和无半发布。
- 几个 fixture 成功不表示泛型单元追平 → 记录每条腿完整编译/行为/反射，拒绝族独立统计，71 单元计数不变。

## Migration Plan

复用已附属、干净的 detached 工作树创建一条实现分支；root 是唯一 Cargo 构建者并使用主仓共享 target。冻结三方基线后实施，root 独立验收再合入、推送、解除分支占用并更新 handoff。至少 20 GiB 可用空间；验收后清理 target，保留真实原输入和可重放取证。
