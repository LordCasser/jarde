## Context

见proposal。诊断八请求已独立接受，非usage JSON同755fd基线，临时源码完整恢复、target absent。真实顺序为outer header4→branch9→prefix14→child header16；branch9没有postdominator，但forward-join准确选37。outer自然块为{4,9,14,16,21,31,37}，child自然块为{16,21,31}，两层均已证明终止叶29；完整canonical共12条Normal边，无handler或clone。child实际AST为一个Loop，ForHeader(init15/update31/slot3)，body末端If(join=None)的两臂是return29与latch31；普通出口/实际next均37，隐式latch来源34。

首拒是`continue_prefixed_loop_arm`读完child后，`loop_arm_join_source`返回None。按同一日志AST和原函数，首先不满足仅接受If(join=Some)的closed_body；随后纯natural所有者、ForHeader却传None的latch证据，以及拒绝所有无后继块的末端门也都不容纳已证明的return。这些是同一消费契约缺口，不是新循环发现或SSA求解问题。

原始基线28命令/56raw及矩阵42命令/84raw已root接受；Jarde八javac23缺return，八javac8及JADX四javac8为真实框架major55边界。JADX四javac23完整重编成功，8core双VM运行24case及4check23case同原。原样本return0在固定范围不可达，所以另用A/B完整类的六调用观察真实早返回、先前尾部效果及累计状态；补充采集现65命令/130raw终态，原16VM48case分别同A/B各自oracle、16Jarde的run均fallback；独立审查及全源码矩阵尚待，不能倒填成功。

## Goals / Non-Goals

**Goals:** 让现有条件臂消费者接收natural块与同次已证明return叶的准确并集；保持唯一普通出口、唯一前缀入口、完整边/块归属、ForHeader和goto来源；原始及补充整类语义/物理身份/全BCI闭环。

**Non-Goals:** 新公共IR、Frame字段、pass、通用solver、可约性放宽、终止叶识别规则扩张、异常/finally/throw新组合、任意嵌套region续接。外层两更新导致ForHeader=None可继续用正确while，不强制美化。

## Decisions

1. **复用现有证明而非引入新证书体系。** 在`loop_arm_join_source`中使用当前child自然集、准确normal join与同一Frame调用`loop_terminal_returns`；额外owner必须恰是其证明的return叶，不能把任意owner超集视为闭合。保留旧无额外owner路径，新增扫描/复制先charge并poll，真实Stop原样传播。helper证明只读Walker归属状态，不新增事务机制。
2. **将方法完成传给既有latch消费者。** child已有ForHeader时，把准确return集合传给`implicit_tail_latch_origin`，再与`gateway_origins`物理BCI逐字相等；不清空gateway、不改goto来源、不放宽update+transfer和唯一latch门。若补充类已读取为等价CompoundIf返回/latch形状，只能在相同准确return证据下复用这一有界两臂证明，不引入新模式发现器。
3. **结构与图闭合同时成立。** 仅允许现有Straight/Sequence/If/CompoundIf语句组合；无join分支须由同次return完成证据闭合，不能单凭`join=None`接受。完整owner multiset必须无重复、等于natural∪return叶，全部已被同次walk持有且在父scope/同clone path内。所有incident canonical边仅为Normal：唯一外来入口entry_source→header、唯一普通出口source→join，其余均内部。无后继owner只能是准确证明的return叶。原prefix/tail边闭合、唯一claim与最终overlap/coverage门保持。
4. **不复制JADX的宽容重写。** 已读本地LoopRegionMaker.process对loop条件出口建立stack边界后递归makeRegion(body)的职责划分；原JADX输出确实保留两层循环和早return。本片使用Jarde既有不可变facts/AST与严格owner模型，不引入ADDED_TO_REGION或processed-block容错重写。现有Rust标准库集合与Runtime足够，无新增库或许可变化；原JADX测试来源按本地Apache-2.0记录，不复制其生产实现。
5. **层次边界。** Reader/classfile解析、Java8 major/校验、JDK选择和隔离运行分别取证；恢复层不通过修输入、删check方法或重写生成Java掩盖问题。原始check真实调用JadxAssertions且依赖major55，Java8失败单列；JVM23完整check与双VM核心方法分开比较。default/all只比较实际正文/质量/fallback/物理来源，允许usage和可选证据不同。
6. **先可复现RED，后产品验收。** 先保存完整原始/补充、物理Code与原oracle，再冻结公开API失败；Luna只准备私有生产候选/测试，root审查集成及独占工具链。必须实际观察补充class和拒绝门，不能把静态草稿、collector terminal或未执行JVM当通过。

## Risks / Trade-offs

- [单纯去掉join限制接纳未证明完成] → return集合来源不变、完整canonical和owner multiset闭合，缺/错/共享叶、额外入口/出口/Exception/clone直接负例。
- [latch转移被遗漏或被误作return] → 原ForHeader+隐式latch证明与实际gateway BCI相等，错误origin/重复claim负例和全部物理BCI审查。
- [新扫描吞Stop或绕过预算] → 新工作charge/poll、直接proof的late budget/cancel/deadline与公开停止门；不回滚已消耗预算、不产出部分Java/map。
- [原常量case无法检验早返回] → A/B同实例真实早返回、正常多迭代、已有尾部效果三类观测，各自原oracle，不以源码手算代替JVM。
- [未证明形状被当已完成] → 所有复编/运行/工具边界与源码映射单独计数；CF-11及71/612仍部分已测。

无需迁移或兼容层。Rust构建采用既有5GiB余量/target1GiB/每秒进程组守卫、默认栈及finally exact clean；实际诊断CLI与最终产品CLI严格分开。
