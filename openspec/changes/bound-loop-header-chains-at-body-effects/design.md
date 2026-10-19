## Context

见proposal。完整原件基线已持久归档在`openspec/evidence/java-syntax-2026-10-19/cf09-testbreakincomplexif-original-root-v1`：54命令，16目标/Point API，Jarde目标八组合均fallback；16完整源码编译失败（八JDK23缺return，八JDK8原始测试框架55/52边界），没有候选VM。原件和JADX各36有限观察相同。

root已执行真实Reader/SSA/Region探针1/0，生产region精确恢复到SHA1dd7dfce…且target absent。debug/no-debug均有自然header7、blocks[7,14,50,63]、latch63、exit72。body14含StringBuilder构造15与Map.get35，null测试47直接到72或字段测试50；字段判断57到更新63或goto60→72。更新63/66分别为length/x。实际recover首先在header_test_chain的后续14处报StatementFree/15，单表头ForHeader路径未到达；辅助独立ForHeader=None不能当实际遍历证据。地址仅记录实例，生产不得匹配BCI、slot或类名。

## Goals / Non-Goals

**Goals:** 在已有AND条件链里正确区分后续纯条件与带效果body边界，利用原遍历完整保留lookup、null保护、break和更新。

**Non-Goals:** 不扩大OR链或真正表头的StatementFree限制，不改自然集合、ForHeader、LoopTarget、CFG、Builder、Region/AST种类，不强制for/while或合并OR文本，不处理任意多入口/异常/clone结构，也不混入临时collector的工具债务。

## Decisions

1. **后续AND候选的局部边界。** 仅在同一外部exit的AND分支，至少一个测试已证明，后续候选精确返回本块的loop StatementFree拒绝时，才允许以该块为body并停止吸收。保留之前tests，继续已有精确body前驱集合检查。仅一个测试时仍返回Ok(None)，交原单表头路径；已有至少两个测试时保留完整复合前缀。初始header、OR及其它失败不改变。
2. **边检查不能因顺序遗漏。** proved_header_test先做branch_arity/leaving_edge与纯度，再验证normal successor/decoded target；因此StatementFree失败本身不等于target已证明。边界分支须核该候选两个正常后继、decoded comparison target确属后继和共同exit，不把未证target转换为成功。复用现有信息和标准集合，不新增跨候选证书或solver；后续body walk仍负责自己的分支与完整ownership。
3. **原表示与所有权足够。** 纯条件前缀仍进Loop.tests，带效果候选完整交给普通loop_body_sequence/If，现有gateway与LoopBreak保留最终出口72和goto60的关系。不能把整个block的表达式/赋值移入测试或清除它的效果。若普通body还暴露独立缺口，先保留拒绝并单列实际事实，不宽松声称Structured。
4. **停止与状态。** 保留已有poll/charge和Result层级；任何Stop直接传播，不退费或作为边界。链识别在占有前只读，新增有限检查应复用同轮数据并计费；不新增事务机制。真实IR测试检验后来候选处预算停止、预取消和visited未部分占有。
5. **JADX参考分工。** 本地IfRegionMaker合并条件与LoopRegionMaker插入break分开处理，启发本片把条件与主体归属分开。使用本仓已有AND/body/If/break逻辑，不复制JADX生产框架；标准库足够、无依赖。原件保留上游许可和完整outer/check/非静态Point。
6. **分层验收。** Reader物理解析、Java8方言、Runtime选择、完整验证/编译、源码恢复与有限运行分别记录。完整原始source和两profile不裁剪；四class/JAR×default/all组合核身份/全文/maps。用冻结最终CLI重编其API输出的完整TestCls/Point源码对，完整原件/JADX outer树精确绑定复用（候选API不输出外围JUnit wrapper，不裁剪目标源码、不注入原始Point binary），保留JDK8原框架边界，以JDK23 --release8与双VM比较九case（含null短路、99/100、99次有限遍历和溢出两步退出）。隔离parentless/空cwd/各目标同根CodeSource和错根负控；只比较异常类别，不混淆JDK诊断差异。原件/JADX仅精确绑定复用。

## Risks / Trade-offs

- [错误吸收有副作用条件] → StatementFree要求不变，只改变后来候选归属，验证调用次数、null保护和更新顺序。
- [简单回Ok(None)丢失多测试前缀] → 保留tests并走原前驱闭合，增加至少两纯测试后带效果的真实IR控制。
- [把结构或Stop当成可退让候选] → 精确匹配loop/StatementFree/本块；验证目标，OR/extra-entry/异常/预算负例保持边界。
- [恢复成功但来源或Point隔离错误] → 完整整类编译/双VM、唯一owner和全部来源审计，保留缺口，不提供原始Point到生成树。

## Migration Plan

先持久归档本次实际IR及源码恢复收据，Luna只在私有tmp实现小补丁/实图tests和公开fixture/tests；root审查后集成，受guard定向验证并完整对照。通过后提交产品并执行一次fresh workspace及本产品自身CI四jobs/独立官方ZIP步骤/双seed全部具名门，绑定Git/CLI/源码，主spec同步、strict、账本/handoff与clean-main。root独占共享写/Git/network/Cargo/JDK/CLI；Cargo5GiB余量、target1GiB、默认栈、每秒guard/finally exactclean，期间无并行共享写或手工目标运行。Luna只共享只读和私有tmp/Python-B。
