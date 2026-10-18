## Context

见proposal。真实Seq1补充run头部BCI61为dup_x1、62为putfield、67为比较，SSA栈复制已由JVM层表达，Java解码保留Other是正确的保守事实。FieldCopies已有Chain与Receiver、moved接收者及保存源值的field_chain_lead；其消费者能将复制输出映射回原值/保存局部。当前Other-owned分支只跳过复制，没有执行saved lead。

## Goals / Non-Goals

**Goals:** 精确识别同块的实例int字段写后直接整数比较，以现有保存局部承载那一次原值；不读回字段，不重复调用，不提前写字段。

**Non-Goals:** 不新增公共AST/Operation或跨块求解，不扩展局部赋值规则，不修改循环region证明；仅接受明确this接收者和int字段，不支持category2、任意接收者、handler或短路赋值。

## Decisions

1. 接入既有FieldCopies的dup_x1拒绝后分支；只在同块紧邻putfield、其后仅整数常量及块末直接整数比较的候选上做完整证明。复用dup_x1_operands/writes、field claim、reads_this和single_use_at。证明三输出分别仅到相邻store的receiver/value以及该comparison，源值同块位于copy前且仅copy使用，所有ValueId未替换、定义同一完整canonical身份；field为I、真实putfield且已证写入，receiver来自this；只支持存活int值为比较的一个操作数，另一个为本块copy后常量。拒绝外来/缺失/额外读者、错slot/定义/物理opcode与任何候选区间handler。各遍历计费并轮询，Stop原样传播。
2. 将此形态写为既有Chain的单store、lead=true、saved=true，moved精确携带receiver输出及原receiver。这个carrier的语义仍是“同一原值被多个消费者使用，先保存一次再写store”；比较是额外消费者，不伪造第二字段写入。若审查发现carrier消费者假定至少两store，先修正文档再实施，不默加新公共抽象。Other-owned复制分支仅对这类既有saved lead调用field_chain_lead，所有其它形态保持原处理。field_copy_expression继续按原moved处理接收者、按source binding处理两个值副本。
3. 保存发生在copy处，field_write仍在putfield处。RHS调用异常时保存未完成且字段不写；写字段后比较saved原值，绝不增加getfield，故volatile写后也不引入额外读。receiver限制this避免移位求值的接收者效果/空异常问题。保存类型须经现有render_value及field赋值规则证明，不能仅凭int栈形认定源级boolean为int。
4. 不移植JADX代码。标准库集合、现有SSA/field/预算/AST已具所需能力，第三方反编译器或通用栈解释器不能替代本地来源/所有权证明；无新增依赖、许可或runtime。

5. 第四轮volatile公开门抓到提前折叠缓存失效：保存/写入正确，但布尔return缓存仍内联next调用。真实完整生成源码双javac编译、四双VM运行均复现调用次数2而原1，不能接受为Structured产品。不能全局重排prepare/deferred绑定阶段，也不能无条件重跑多族prepare并丢失allowed_exception_row。仅对当前branch中精确saved单store dup_x1、已发布Int源绑定且phi未发布的Folded候选，在test_effects执行一次后复用既有单If conditional-value证明；必须同phi、原缓存Boolean、唯一直接Return消费、两叶为0/1常量且无异常row上下文需要，重建同类型Expr成功后原子替换一项缓存。失败移除旧phi、明确Refused/fallback，Stop照常传播；复杂carry/tree/handler不靠旧缓存继续输出。保留既有deferred计划的同phi/唯一Return/类型身份，不新增缓存失效系统、延迟IR或绑定载体。新增局部选择/重证计费，旧无此复制的路径不变。

## Risks / Trade-offs

- [复制被当通用dup导致重复RHS] → 独立次数/ledger/指定抛出，同块唯一消费者与源码saved检查。
- [写字段后重新读取，丢失原比较值] → 检查生成比较读取保存局部；加入volatile同形控制，不用线程非确定性当oracle。
- [与已有chain/receiver计划冲突] → 原优先级不变、只有原拒绝才尝试，旧链/复合更新控制必须实测。
- [失败留下半个plan或部分输出] → 完整局部候选成功才提交，预算/取消/期限与源/map门实测。

## Migration Plan

冻结首轮失败；Luna私有候选由root逐块审读后集成。先真实Reader正反例、公开补充/停止及旧字段链控制，再与Seq1同一最终CLI完整重编/双VM验收；本依赖单独记录范围与结果，不把父循环task自动关闭。默认栈/5GiB余量/target1GiB/每秒守卫及finally exact clean。完整通过、提交推送自身CI与干净主线后才能交付。
