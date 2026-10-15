# Design

## Context

动机与外部行为见 proposal.md 和 specs/java8-recovery/spec.md。此片需要设计，因为同一个消费者既有直接 Error 吞没，也有 Report→无事实的源码边界；后续 typed snapshot 证明可能让最终 Partial 看起来正确，却不能证明首停已传播。

基线 main f30df5d929cf88d0e97514ea4d5b0806eeecfe2e 的产品输入与 aadf71、冻结CLI c2db4721a879c47493b537e946b0ff827986e1e58e7b021222df1e66a8c9c3cd 相同。EM11七个未改Java由受控javac23 --release8编译，各class SHA与历史实验一致；此次容器是新建5001B STORE JAR，不称原始JAR。InputHierarchyCalls.run()Ljava/lang/String; 的 NEW 位于BCI0，构造器调用BCI4，invokestatic take(HMid)位于BCI7；source-map的producer_bci4不能冒称NEW指令位置。

已有实际8命令screen：宽预算4096及cap3/4 Complete，cap1/2 Partial、exit4且run正文空，原Java23 Runner为固定五行219B。实际GDB观察cap1有两次Budget::exceeded：先reference-overload/raw-reader，后typed snapshot；cap2只有后者。wrapper exit0与inferior exit4分别记录。仅归一收据记录的10个usage.elapsed_millis叶子后，整份JSON与普通同cap结果一致，所有limits保持原样。GDB未打印返回Error/Report或catch执行，因此渠道损失为绑定源码的静态风险；没有动态覆盖取消、期限或平台缺失停止。立项时尚无fresh完整Jarde类族重编；后续实际实验以绑定最终CLI的收据为准，不把旧实验或仅原Java运行冒称此证据。

## Goals / Non-Goals

**Goals:** 用已有 DependencyProofFailure、恢复前工厂与proof closure承载本消费者的首停；测试能区分两原始渠道、普通拒绝及首次停止后的真实tail，最终源码与CI绑定到新修复。

**Non-Goals:** 不更改其它raw-reader消费者、全局预算/报告合并/发布准入；不扩展匹配或终端策略，不引入新依赖、通用proof框架或测试ledger。method-only无assembly context路径不因本片增加证明工作。

## Decisions

1. **局部接入既有typed handoff。** reference_overload_calls_presented/prove_reference_overload_calls 及其必需owner/source读取返回既有typed结果，由当前class-source proof closure传播，移除本消费者将BudgetExceeded/Cancelled转空vec的吞没。selected_dependency_proof_header复用selected_reference_header_with的cache、poll及收费；snapshot_dependency_proof_widens沿用该消费者原u64::MAX，不能换成相邻snapshot消费者的8层固定界。可能失去唯一消费者的旧适配器仅在确认无其它使用时移除；Result桥接使用的snapshot_header_chain_widens_with继续保留。替代方案是修改全局raw-reader None契约，会扩散到无关消费者，故不采用。

2. **平台缺失不等于无Header。** platform_class_unprovided仍使用既有resolve_symbol及完整谓词：state=Missing、execution=Complete、environment_problems/unresolved_dependencies/candidates全空；source与target两次检查的短路顺序不变。先交接原报告中真正结束共享请求的停止，再计算原判据，不能用typed header None代替平台缺失。ArrayList→List、Java8、ParentFirst、ClassPath、external_override/runtime_transformation均None的原条件不变。平台真正Missing无候选不一定收费ClassHeaders，不声称cap0必能覆盖该报告渠道；实际可达的Report停点与未覆盖组合须如实记录。

3. **冻结正常判据。** 保留NEW来源发现、仅invokevirtual/invokestatic、单引用参数descriptor、唯一目标/private分支、name/descriptor/staticness/flags/receiver及BCI规则；保留snapshot walk的目标命名即可到达、普通缺失/歧义/不适用及值级隔离。已有15个counted维度、缓存命中前poll/charge和paid-prefix准入不变，无退款/重置/重试或后继poll重建原因。

4. **真实首停回归。** 复用现有cfg(test) class_source_proof_inflight_tests的最窄stage/callback，新增reference-overload边界及准确PhysicalMethodId绑定（物理owner、run、descriptor），不要只凭方法名；激活后应继续记录同一方法的后继stage，避免过滤器让tail断言空成功。分别记录原Error与原Report的实际产生、handoff语义和原诊断，不以地址相同作契约。取消采用初始clear token与实际accepted charge触发，期限采用到达该stage后真实ledger/checkpoint，禁止入口预取消、sleep、伪Error。覆盖实际ClassHeaders拒付Report时必须真正到达外部HLeaf/所需owner读点，不能用IR快捷路径代替。每个渠道先在旧生产运行，按首个真实失败断言报告；直到动态确认前不将现有GDB推断当渠道验收。

5. **公共报告与产物边界。** plain Budget及sticky ledger分别验证首停原因/诊断、15维counted用量及空后继tail。EM11的run前有构造器和两个take，应按实际成员顺序核已支付前缀，不能抄DT29的prefix数量；当前record是否交付遵循既有准入。公共库/JSON CLI宽预算及cap矩阵必须真有重载转换/来源；旧低cap本已Partial，新增红例应针对首停丢失或真实tail，不把已绿的exit4重新称作失败。

6. **实际整组源码和产品门。** 最终冻结CLI对原七类生成完整Java，全部原样编译，禁止改文本、删类/方法、替换oracle或把原JAR混入rebuilt classpath。原始类与重编类在Java8/23上以-Xverify:all运行固定五行219B oracle；可用原Runner独立交叉调用时也保留其未改字节。验证绑定输入、CLI、生成源、命令/退出码/raw与源码before/after；若组装/编译失败就保留失败并定位，不能只取片段宣称成功。物理Code与source-map转换锚点分开核实。采用现有javac/运行脚本及项目测试依赖，无需增加库、许可或维护面；旧JADX正例仅作历史背景，只有出现新差异才按需重跑基准。

## Risks / Trade-offs

- [后续Partial掩盖首停丢失] → 不只断言最终exit4，记录原始渠道、handoff用量与真实stage/tail，并保留旧失败raw。
- [平台报告停止与普通缺失混淆] → typed交接限于原共享请求停止，完整原absence谓词和正常阴性控制逐项保留；未动态覆盖组合不得冒称完成。
- [测试observer自身屏蔽后继工作] → exact物理身份绑定且激活后所有后继stage可见，source/owner实际读取另记；不引入全局observer框架。
- [预算停止后无法支付当前成员record] → 只要求合法已发布前缀、原类诊断与coverage，不能强迫额外准入；在确实可准入的plain Budget控制要求实际Stopped record。
- [宽预算产物/原始行为回归] → 完整七源原样重编、五行oracle双JVM对照、普通缺失歧义与平台条件控制，加修复commit自身完整CI。编译/verifier成功不等于一般语义等价，固定Runner不穷尽输入。

## Migration Plan

先封存基线和可真正变红的永久回归，再局部接线并复核正常判据，运行局部门禁和最终CLI实验；exact clean保留无target，提交产品并等自身CI/独审。只有上述验收完成才勾选任务、同步主spec并正常归档，最终main绑定到该产品源码。实施进展与勾选见tasks.md，前片aadf71的CI不能完成本片。
