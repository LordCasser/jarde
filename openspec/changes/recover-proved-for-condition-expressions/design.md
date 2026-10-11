## Context

见proposal.md。基线与真实首拒证据分别在openspec/evidence/java-syntax-2026-10-11/cf11-testnestedloops2-baseline-root-v1和cf11-testnestedloops2-same-ir-root-v1。原方法10节点/12边/28SSA指令；outer header4、init3、update41/transfer44，准确两输入phi。Invoke@6写ValueId22，SSA表中该值仅由同block branch@11消费。inner header26因preheader末尾并非其induction store而拒绝，继续while。

生产唯一入口header_tested_loop先test_is_pure(header,test_bci,true)，再prove_for_header。前者已证明调用/字段读取属于branch值依赖、数组长度/元素读取单消费者；后者重复使用更窄的operation表，遗漏这些已支持producer。Builder已有test_expr原位条件及唯一init/update消费。

## Goals / Non-Goals

**Goals:** 消除两处准入不一致，使满足既有计数循环所有权与消费证明的条件值表达式保留for；覆盖调用、动态字段与数组读取、取负等已支持表达式的真实行为和来源。

**Non-Goals:** 不改现有条件assignment/postfix准入，不新增Store/Increment/Copy准入，不引入purity walker/新证书/IR/pass，不 hoist条件或改自然环、更新/continue目标，不修generic body proof或其它loop形态。已有可靠while/refusal保留；固定inner while不强制for。

## Decisions

1. **复用已有条件前提，扩现有operation match。** 在保留induction/invariant Load处理后，让非局部写入的Invoke、Field Read、ArrayLength、ArrayLoad/ArrayElementLoad、Negate、NumericComparison通过现有扫描。该扫描仅识别local角色，表达式能否留在条件由此前test_is_pure决定；注释明确生产caller契约。保留已有Push/Arithmetic/Comparison，不调用第二遍条件证明或建立新的pure表。仅加Invoke会留下同一架构矛盾，重建通用expression证明则无必要。
2. **局部、CFG与消费门原样。** 保留唯一latch/回边、所有入边、更新前effect、唯一preheader、末尾准确Store、init栈生产者单用、两输入phi与init/update单用、循环内induction/invariant写和循环外访问限制。Builder继续原条件test_expr、准确init语句、唯一赋值update与同名局部门；不移动未吸收语句，不重新拼表达式。for内continue仍映射准确update_block；原body/exit/gateway所有权不改变。
3. **效果按原位置执行。** 调用和字段允许有副作用或抛异常；保持头部每轮比较的operand顺序，不能缓存到初始化或移到正文。数组既有单读者门保留。真实多调用条件、初次/后续异常、body修改field边界及array null/越界控制必须同原class；复制/共享effect未获现有消费证明时保持可靠拒绝，不为本片增加另一个一般机制。
4. **停止与来源用既有账。** 只是既有每条instruction匹配范围，不增加扫描/集合/存储；prove_for_header现有预charge、poll和传播保持。获证后更多Builder工作仍按现有计费，记录实际晚Stop及预取消，核不发布部分正文/来源。所有条件producer、init、update和回跳必须有合法完整物理来源，default/all正文和maps相同。
5. **参考JADX已有分解。** LoopRegionVisitor.checkForIndexedLoop基于init/update/phi/loop-use，将既有条件用于ForLoop，未单独禁止条件调用；本仓复用更明确的SSA和所有权证据。无需新依赖或复制外部实现；参考算法、许可证与维护边界不增加当前包负担。parse/physical Reader、Java方言、恢复、映射与运行行为分别验收，JADX结果不是行为真值。

## Risks / Trade-offs

- 调用被复制、顺序改变或读取被缓存 → 多operand调用记录events/calls，并核早/后异常和动态field/array对照；共享producer或独立statement明确拒绝新for。
- local boundary在body写入 → 继续原invariant拒绝，保持可靠while及原行为；Field内容变化不同于local重新赋值，不能混同。
- 新for吸收更新后来源/continue错误 → 真实continue与全部BCI/UTF8 span核验，旧ForBoundaries、gateway、foreach、return-arm控制回归。
- whole class其它成员fallback掩盖结果 → 两个正例类和一个拒绝类先跑完整基线，原始失败保留；不改生成源码消除错误。若出现当前片之外缺口，独立记录，不扩大范围。
- budget成本变化 → 真实observed usage/location与原子Stop验收，不将预取消写为异步中途覆盖。

## Migration Plan

Root先冻结真实控制基线，再按tasks接受私有Luna候选。Root独占shared/Git/Cargo/JDK/JADX/CLI，5GiB free/target1GiB一秒进程组守卫且结束clean，不新增分支/worktree。实现前移除全部临时logger/test；新CLI绑定完整输入闭包并做完整双JDK三方及fresh workspace/CI等价lint/strict，自身产品CI和干净main后交付。原失败、静态候选与实测事实保持分开。泛型raw List债务独立，不增加CF11/71完成数。
