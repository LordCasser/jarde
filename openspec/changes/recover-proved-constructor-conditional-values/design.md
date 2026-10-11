## Context

见proposal.md。真实基线在../../evidence/java-syntax-2026-10-11/cf11-testsequentialloops2-baseline-root-v1；实际同IR在../../evidence/java-syntax-2026-10-12/cf11-testsequentialloops2-same-ir-root-v1。13blocks/18edges，new91/dup94/ctor96输出V53，仅有Phi99的无BCI use；live Stack0 Phi V14输入V48/V53，仅Return99读取。outside_readers跳过该use，Sites拒绝nothingreads。该证据只确认init/Sites首拒。

现有report先构造Sites，再Region与Builder。Builder的prove_conditional_value已证明准确branch两条正常边、互不重叠straight arms、全入出边、两输入live stack Phi及唯一join消费者；build_conditional_value再证明每条arm指令均被表达式/transfer消费。最终emit成功后field::committed_presentations已有单次带预算AST遍历，之后才产生summary/refusals和RuleDetails。现有Sites直接把accepted视为presented，Publication仅筛选类别/位置；新路径不能借这个旧假设。

## Goals / Non-Goals

**Goals:** 补一个严格局部所有权关系，复用完整条件证明和最终AST扫描；成功覆盖固定构造条件尾部及真实控制，统一所有公开呈现判定。

**Non-Goals:** 不递归alias/Phi、不支持local/replaced/self/multi-input/nested Phi、不新增CFG proof/pass/公开IR或可变Builder emitted-set/checkpoint；不修普通Site既有发布债务、foreach/泛型或不相关source-map缺口。finished_value仍保留组合数组的既有身份含义，不能把它直接改成合并结果。

## Decisions

1. **限定一跳完成值。** 只认准确constructor instruction的已初始化输出：该值没有replacement、只有一个无BCI Phi operand use；目标Phi为同use block内live、未replaced、两个不同的显式Value输入、Slot::Stack，构造值恰好一次。Phi结果只有一个Some(consumer) use，消费者在自己的join block内、instruction实际读取该结果一次，仍通过renders_its_reads及既有field::Plan::owns。原allocation/dup/ctor/argument扫描、其它外部reader拒绝及计费保持。两臂不同构造各自独立获证；此处不决定branch方向或证明另一臂。
2. **完整语义仍归Builder。** 复用已有prove_conditional_value/build_conditional_value、new_expr和same-run Sites，不新建Site侧CFG算法。不复制IR、不移动branch或条件生产者。若后续另一个真实首拒出现，先记录与现有表达式消费的缺口，再在本片语义要求内调整实现；不能把未恢复的固定目标算完成。
3. **最终呈现事实复用一遍AST。** 给Site保存最小conditional-path标记。现有committed_presentations遍历同时收集候选constructor BCI的实际ExprKind::New次数（封顶2、缺项为0），只认primary.method().is_none()且primary BCI准确匹配。继续已有New qualifier/args/Conditional等子节点遍历；foreign inlined origin与method reference不能匹配。仅保存新路径candidate规模的稀疏事实，既有节点计费保持，额外匹配/记录按既有Budget计费。无需第二遍AST扫描。
4. **同一事实、同一判定。** 仅conditional-path Site需要count==1；0/2+形成有已知Site元数据的not-presented记录和一致refusal。Sites.refusals/counts/materialize使用相同immutable最终事实及同一个私有判定，仿照现有field::Plan接口即可；优先不新增ResolvedSites对象或复制整份决定集合。answered是否有候选仍是原语义。report在选择门前得到事实，default/all/noRuleDetails的summary与gap一致，范围仅决定record交付。普通Site保持原路径，旧发布风险单独留账。
5. **参考既有工作而不引入依赖。** 本地JADX TernaryMod.makeTernaryInsn先核两臂结果同一个OnlyOneUseInPhi，再wrap为Ternary；Jarde已有更明确的readonly CFG/SSA与完整来源证明，复用这些即可。此片只参考算法关系，不复制外部代码，不新增第三方包或许可证负担。Reader/Java方言/Runtime选中class/verification状态与source recovery分开核；JADX输出作为对照，原class才是行为oracle。

## Risks / Trade-offs

- Site获证但Builder回退→实际New唯一性作为最终门，同一判定驱动gap/count/record，覆盖零/重复/foreign origins和default/all/range。
- 把Phi消费当完整分支证明→严格保留Builder全部门与独立effect/exception反例，Site不拥有CFG。
- 构造实参抛异常、被复制或顺序改变→双JDK完整类记录events/身份/异常，所有raw失败保留，不编辑生成Java。
- 新计费/取消路径遗漏→真实late Stop核dimension/usage/location及原子无部分产物，pre-cancel另核；不硬猜费用。
- 基线其他成员掩盖目标→完整类优先并原样保存失败；相同source隔离控制独立标明，不能替代固定完整目标恢复。

## Migration Plan

Root先接受控制基线，再接受Luna私有实现并应用共享源码；root独占Git/Cargo/JDK/JADX/CLI。沿用5GiB free/target1GiB一秒进程组守卫、默认Rust栈且结束clean；不新增功能分支/worktree。先定向证明/公开API/物理map，再冻结同完整source closure的新CLI并实跑固定及控制完整双JDK三方；最后fresh workspace、fmt/CI等价Clippy/global strict、自身产品CI、推送和实际clean audit。整个CF11/71完成数按固定账本更新，不按本片任务数增加。
