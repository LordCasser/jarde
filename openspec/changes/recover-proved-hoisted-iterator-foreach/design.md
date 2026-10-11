## Context

动机见proposal。固定上游源码来自JADX checkout2fb1b16386941660fda07e9017285aec40fcb37f，已接受的完整三方基线见java-syntax-2026-10-12/cf11-testnestedloops-baseline-root-v1；same-ir-root-v3明确inner42在iterator_declaration_before_init、outer13在body_first_not_initialized_declaration。当前Builder先要求首个元素Declare，再在当前parent statements找iterator空声明。实际local5空声明位于祖先词法列表，初始化赋值在当前列表。当前CLI wholeclass与临时诊断v1/v3已root独立接受；inner42的init/test/next LocalVariable均为slot5/index0，placement为Elevated owner[1]、已发出且可见。继续后的准确首拒为iterator_value_and_phi_use_ownership；header35 V12=[V37,Itself]且替换为V37，Phi97 V22=[V37,V37,V37]、uses空、替换为V12，V37的三条None@97正好对应三个输入。更后的next/cast/effect门未执行，不将日志事实当门通过。

## Goals / Non-Goals

仅支持已有完整iterator证明能够覆盖的内层循环；越过声明门后仍拒绝的形态保持原while并记录具体证据。目标是修正证明事实的消费位置，不扩张planner/自然环/类型系统。外层Assign、泛型Signature、死声明优化和普通Site发布债务各自处理。

## Decisions

1. 声明证明复用reuse::Plan::variable_at与Declarations.placements/at_region。init Store、hasNext/next Load在准确BCI上必须映射同一LocalVariable；不能用slot/name相同代替。Elevated owner必须存在相应at_region条目、词法包含初始化和消费路径、且已安排发射；Incomplete/缺owner继续拒绝。当前列表原有空声明路径也需身份核验。owner是loop path前缀仅证明可见性，不能据此推定空声明位于另一statement list：直接单层loop的owner常为[]、loop path含子索引，但空声明实际仍在当前self.stmts。若现有事实不足，只收集真实首拒，不新建全局声明索引或跨frame编辑协议。
2. 候选只返回当前列表的可删除位置。以经准确身份/owner核准的空声明是否实际存在于self.stmts区分两条路径；若存在，沿既有同列表路径删除其当前索引并保留旧间隔准入。若不存在，只接受已发出且完整可见的Elevated严格祖先owner，按新路径保留其它列表的声明。不能以owner.len()<loop path.len()替代真实当前列表匹配。祖先其它列表声明不被删除；当前初始化赋值只能在完整证明成功后删除。当前列表的空声明可沿既有路径删去；不同列表的索引不能混用。保留一条无用但合法空声明是此窄片可接受取舍。变换失败或Stop时不发布部分改写。
3. 原initializer、descriptor、header Phi/inputs、额外读写、next→cast→store唯一消费和handlers/effects证书保持。仅在既有iterator use门消费SSA已有replacement事实：对每个None-use block扫描全部Phi，只匹配输入中含当前精确ValueId的Phi；其精确input occurrence总数必须等于该block的None-use记录数。每个匹配输出必须已替换、uses为空且沿现有有界replacement链解析到同一iterator，每个Value arm同样解析，Itself不计次数。无replacement、live/mixed arm、未知block或计数不符拒绝。不含该精确value的其它slot Phi不参与判定；含它的其它slot必须完整检查。计数按精确ID，replacement仅用于语义核验，不按代表合并occurrence。不改SSA/reuse、不遍历指令alias，不增加通用liveness服务。新增owner/Phi/input/hop工作需明确计费并传播真实StopReason，不能由Option闭包吞掉Stop。

   新祖先声明路径（准确空声明不在当前self.stmts）只允许init与loop之间的空声明；literal局部写入也可被iterator()抛错后的catch观察，不能仅凭无调用放行。旧同列表路径的此类时序风险单独登记，当前不混修。实际目标无执行语句间隔。inline cast与separate Cast(Local)旧证书深度不同，不宣称两者具有同一显式CheckCast/handler检查；目标应依其真实分支验收。
4. 复用已有Object iteratorElement头及原body cast/元素绑定。raw List足以表达当前类型，不新增typed String头或generic body证明。参考JADX LoopRegionVisitor的iterator模式检查；其可变类型fix不构成本仓proof，不复制代码或引入JADX依赖。
5. root独占工具链/共享编辑/Git，Luna仅私有候选和测试准备。用户已授权先spec后实现；本片首项是受控诊断及真实控制基线，当前构造条件值产品已本地接受并推送、自身CI已root独立接受；其交付期间执行的临时诊断/只读观察均按原字节恢复并绑定原产品；完成当前关闭文档clean audit后才应用永久候选。保持5GiB free/target1GiB一秒进程组守卫、默认Rust栈、结束clean。

## Risks / Trade-offs

- 祖先同名局部或split slot误配 → 精确LocalVariable/owner和各物理load/store身份核验，添加拒绝反例。
- iterator创建被移到outer loop外、next/cast重复或异常边改变 → 保留原入口位置及所有既有effect/handler证书，用调用次数、异常顺序、null/wrong-element/空列表完整运行对照。
- 只删初始化导致来源丢失 → 用已有foreach origins完整承接init/hasNext/next/cast/store及回边，逐物理BCI核对；不能以runtime成功代替来源验收。
- 声明门后的SSA拒绝已确认 → 逐block核精确Phi input/use multiset和有界replacement；未知/非平凡继续拒绝，不预先宣称更后的完整proof通过。
- 中间局部写入在初始化抛错时可观察 → 新祖先路径仅容空声明，带catch观察的written-gap控制记录原/JADX/Jarde真实结果；旧同列表风险为独立债务。
- blank declaration保留降低源码简洁度 → 此片接受其合法存在，死声明清理独立。

## Migration Plan

先固定current CLI wholeclass观察和真实声明owner/后续门，再按证据最小修改既有helper；永久测试、完整双JDK原/JADX/Jarde对照、fresh工作区/Clippy/strict/自身CI后交付main。只涉及私有实现，无公开API迁移；失败候选不作为已接受产品。

## Candidate review refinement

私有v1把strict owner prefix直接当作新祖先列表，root及独立审阅以既有RawIterableIterator.rawCast真实source/javap与region/declare_at/take-restore规则证实会回归：init/store0–6、result=0@7–8、header9；owner[]是loop path祖先，但其空声明就在root当前vector。v1静态不接受、未应用/编译。v2按当前list的准确匹配区分，不新增frameowner服务；新路径只删本list初始化，其他列表空声明保留。该澄清不修旧同列表literal-gap时序债务。
