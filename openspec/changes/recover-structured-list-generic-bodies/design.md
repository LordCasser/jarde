## Context

动机和边界见proposal/spec。基底产品5f5fb2c6、本片前最终main f12668ed0；前片8/8、自身CI38197592542、最终外置audit均真实通过。固定class SHA `8ece89e4d38751c20529622b57417afc8601368fc20078656ea2b24c17ed8231`。现有真实诊断v3有45Code/45operations、13SSA blocks/21Phi、13statements/5roots、无stop/handler，三请求candidate=None并在同一return首节点门拒绝。实际Cargo test/clean均0且4784source恢复；采集器错误把usage快照逐字比较，原失败保留，独立结构事实接受另列，不能把它冒充产品通过。

Root实际读report.rs generic_return_candidate/generic_void_unread_parameter_slots、ClassSourceMethodAstSource、class_source_invoke_ast_sites、project_class_source_invoke_argument_edits，及class_source.rs ordinary_parameterized_declaration/GenericCallProjectionState。现有presentation-wrapper只接纳Call或Null，s1为Local，故现有移除接口需有限匹配扩展；无需新增RecoveryRequest泛型carrier。当前平台raw继承表不是泛型契约，不借它证明E。

## Goals / Non-Goals

**Goals:** 完成已恢复结构化void List<S>正文的泛型声明和调用可赋值性闭环，作为一个CF11里程碑；至少String和不同元素型，不硬编码fixture。物理census、源级consumer证明、签名擦除、运行对照各自验收。

**Non-Goals:** 不改自然环识别、SSA/Phi类型，不新增泛型solver、identity/层级服务或第二AST/pass，不恢复wildcard capture、用户任意层级、参数重赋值、raw别名escape、handler。不删除空Iterator声明、不把Object foreach头改成S头；保留Iterator.next之后真实checkcast，有heap pollution时异常时序必须相同。JADX许可代码不复制；仅参考算法。

## Decisions

1. 复用既有VoidBody完整性候选，而非先把签名强行接纳。将物理访问核验从唯一block扩到真实所有blocks：按BCI准确配对Code/typed operands/SSA/effects/operations，保留multiplicity，拒绝缺失/重复/额外/stop/handlers，核参数任何写并逐计费/poll。不以21Phi存在作为拒绝理由，也不从Phi推导泛型。必要读→单Stack输出→唯一Invoke接收者的有限映射在既有同次VoidBody/census中产生，并可附入既有候选或保留AST内部数据；它是当前物理事实的有界handoff，不是新的RecoveryRequest载体、pass或服务。每条映射必须核全部SSA uses、receiver堆栈位置及实际CallTarget，源consumer再逐项配对，禁止按相邻BCI猜。完整结构化Program保持non-ragged且void出口/可呈现节点来源闭合；已有AST/census遍历优先复用。unread列表只记录真实未读，绝不把已证明List用途冒充unread。
2. 源级证明在class-source当前方法签名已parse+erasure后进行。用既有保留AST/physical parameter slots/formal names和calls库存核准每个List<S>参数的全部用途：只接受直接foreach iterable及闭集List接收者调用；所有物理参数读取逐一由源级consumer闭合，不从相同名字猜identity。实参类型来自同AST实际局部声明/值形状、精确作用域及producer/consumer事实；presented辅助信息不能单独发起generic类型证明。参数locals不重赋值只闭合写侧，未覆盖alias/escape必须拒绝。
3. 维护最小有限Java平台List<E>契约，owner/name/descriptor/调用kind完整匹配并受现有release边界约束。固定用途包括iterator()、size()、clear()、add(E)Z、remove(Object)Z；add的E由确切receiver List<S>绑定，remove仍是Object，与remove(int)E区分。String和至少另一已可拼写精确reference元素类型的相等赋值可接纳；null可赋给引用S，必要闭集widening沿已有证据。未知子类、任意泛型成员和wildcard不由raw层级推测。平台契约须用本机Java8/23官方类文件签名或JDK源码和JADX调用算法独立核对，无隐式运行时依赖加载、无新依赖；不建完整JDK泛型数据库。
4. 精确cast移除复用ClassSourceInvokeArgumentPresentationCast与project_class_source_invoke_argument_edits。扩大wrapper识别只允许已核准直接Local等本片必需形状，仍要求与child同primary/原derived前缀及唯一调用Derived锚、精确调用method/BCI/opcode/target/argument index。这仅区分合成包装，不能代替泛型/重载准入证明。固定add@79 wrapper源77、physicalStringcast@22不同；remove@91保留Object包装。不得修改invocation_argument的全局Object规则。
5. 源级声明和投影后的AST重发射在现有GenericCallProjectionState overlay内完成，只有完整用途、所有edits唯一匹配、原AST emission与原report正文一致、新声明/正文均成功后安装。ordinary parameter gate只接受本次完整源证明对应参数；不放宽全局read参数规则，不重跑整个recover。任何局部证明布尔/槽列表只在同方法同parsed签名这次调用内使用，不构造新的公共服务或跨方法缓存。停止沿既有Result传播，阶段中止不得安装部分overlay，所有克隆/遍历/输出计费。
6. 本地JADX LoopRegionVisitor.fixIterableType参考其receiver泛型元素与实际变量关系逐项核准，MethodInvokeVisitor.resolveTypeVars/processOverloaded参考先替换callee formal再判断cast与重载，TypeInferenceVisitor invoke use-bound参考同时核实际参数用途。JADX全classpath mutable inference不适配本仓只读IR；复用本仓有限AST投影即可，不照搬pass。标准库及已有reader签名解析/证明足够，无新增库许可维护成本。

## Risks / Trade-offs

- 用Signature或无重写冒充consumer证明 → 实测所有参数读取/调用/迭代闭合，alias/错误元素和unknown owner反例。
- 把物理cast当合成wrapper → 实际opcode/origin和唯一derived锚控制，保留Iterator.next cast、另测显式Object/Stringcast。
- 恢复List<Integer>误选remove(int) → 按完整descriptor/argument位置分别实跑对象/索引路径，不能全局删除Objectcast。
- 泛型签名与输出正文不同步 → 现有事务完整stage后一次安装，失败保留可靠raw正文与准确refusal；测试取消/预算和三profile。
- AST递归编辑深度旧债务 → 独立记录，不混入当前语法；所有新增扫描有预算，不引入无界递归。资源usage快照与结构事实不同 → 原样保存各请求计数并分开比较，不能抹掉结构差异。

## Migration Plan

先接受同次观察，冻结JDK平台契约与正反控制；Luna私有候选/root审查后串行应用。root构建/CLI/双JDK三方完整类+Runner及永久公开测试、lateStop/precancel、必要回归/freshworkspace/fmt/CIClippy/globalstrict；绑定最终来源与CLI，提交推送产品自身CI与fresh clean-main后关闭。本次持续授权已含spec后派发实现，按apply继续，不另设确认门。磁盘最低5GiB/target1GiB一秒进程组守卫/defaultstack/finallyclean；root独占共享Git/Cargo/JDK/JADX。
