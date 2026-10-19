## Context

动机见proposal。完整固定原件是1216B的TestBreakInLoop3.java，8API都在writeMore因owner11重叠整方法回退；JADX debug/no-debug完整源码两JDK23编译、四VM均得到builder12135678且check通过。NYI/@Test注释是测试启用状态，不能据此宣称实际JADX失败。

同轮四profile探针1 test通过并恢复源码。Loop3的NormalFlowView没有自然循环；异常行row0为[17,28)、handler31、type32。canonical块11包含未受保护lead(11,17)及受保护suffix，不能杜撰一个canonical block17。handler31在34分支：37更新flag并由45回11；另一臂48分到61/70，汇合76，再到正常join82。原正常完成28也到82。当前guard可识别Try，但retry事实None，handler重试臂递归再次构造同一Try，使prefinal树重复11/31；最终owner查验正确拒绝，不能删除该查验。Sites::empty探针不是完整公开Builder验收。

## Goals / Non-Goals

**Goals:** 用既有named retry契约表达同一canonical块内未保护前缀+精确保护suffix，正常完成的catch DAG与retry臂闭合，单次拥有各块；保留初始prefix0和join82后的完整效果。

**Non-Goals:** 不把exception cycle当普通自然loop，不增加通用CFG修复、AST种类或任意handler图接受，不扩Smali/Dex前端，不删除owner/lateStop门，不扩大异常保护区，不因JADX的源码形态约束自身等价拼写。

## Decisions

- 保留唯一具名物理row、canonical handler row、原捕获类型与throw-site/指令范围为权威；lead起点11与保护start17必须分别验证。Header与受保护范围不等同，不得通过把row.start改为11来接受前缀抛出的异常。
- 扩展既有RetryLoopFacts/RetryHandlerBranchFacts及Frame精确转移授权以承载这一受证形状；旧Loop2条件重抛合同保持原状。只有新增事实确有区别时补最小私有字段/变体，避免另造通用图框架或重复Walker。候选必须先通过非空lead与单row等固定形状门，避免所有try新增全图扫描费用。
- retry臂唯一回到11，正常臂是单入口、无回边、唯一边界82的有限DAG；精确验证各canonical Normal/Exception/Call/Return边、已占用/父scope及handler用途，完整owner集合先证明再组装。复用现有If/Sequence走catch diamond，既有Try lead输出前缀，Endless/LoopContinue及现有完成转移表达退出，不递归重开同Try。
- 同轮组装探针显示正常臂内层If先到join76，既有延续读取随后正确得到`[Straight(76), LoopBreak(79 -> header11)]`，却因只接受纯Straight尾部而拒绝。扩展现有直线延续组合时，仅接纳精确的Straight加LoopBreak：来源等于最后物理指令、目标为当前LoopTarget的header、break target等于原frame boundary，继续通过原有scope、前驱、直线边和新owner核对；79是invoke后的普通fallthrough，不要求虚构goto。无需另加Frame载体或放宽ArmsDoNotMeet。错误目标/来源/边界、额外区域和Stop仍拒绝或回滚；此设计修订本身不算实现验收。
- 证书不认证未证明的helper调用。catch参数、异常指令范围和局部变量类型仍由既有guard/Builder证明；若真实公开路径另有缺口，先定位再修既有合同，不能以宽松boolean/槽身份猜测补洞。
- Region修复后真实Engine公开路径另在局部3拒绝：循环前File构造/保存已在Straight0呈现，但`presented_reference_store_value`将构造表达式限于protected arm。只移除构造分支的位置限制，保留同块、单次消费、verified Sites、完整expression/run与全量SSA reaching-write检查；声明提升不移动初始化效果，静态调用限制不变。root临时仅改这一条件后8API及预取消测试共2项通过，并已恢复原源；永久补丁和全回归仍待Luna/root验收。无需新增槽/phi/重试专用Builder旁路。
- 比较“放宽所有异常回边”“跳过重复owner”和专用源码模板：前两者丢失范围/执行证据，后者依赖fixture表面形状，均拒绝。标准库集合、当前Reader/Runtime及已有图事实已足够，不新增依赖；JADX LoopRegionMaker的break/continue互斥只提供参考，不能直接移植其可变Android IR。fixture保留Apache LICENSE/NOTICE。
- 原件空路径仅覆盖首次重试与后续失败正常完成；另外以无文件副作用的有限控制覆盖正常成功/重试后成功/两臂diamond及prefix/handler自身异常，不能冒称原件File.exists真分支已执行。

## Risks / Trade-offs

[前缀或handler异常被误捕获] → 逐指令保护范围证据，运行外部失败控制，禁止扩大row。

[重试或正常尾部重复归属/执行] → 证书闭合后一次组装、全树owner查验，比较完整事件trace/builder与状态变化。

[共享入口、row漂移或Call/Return穿越] → 原拒绝边界不放宽，真实IR派生负例直接检查拒绝，不以fallback被其他不相干规则拒绝充数。

[候选扫描/组装中Stop] → 完整checkpoint回滚visited及tail/Frame状态；用量不退款，保留预取消和晚期停止的实际位置；旧精确预算门不改。

## Migration Plan

两项改动可由Luna在私有base86a200...上并行准备，root逐一审查/合入，冲突由root按函数职责合并。真实fixture/Reader及公开API通过后，冻结同一新CLI重做两缺口完整三方，重开DoWhileBreak3已有正向与Loop4/5/Loop2/finally关键门；完整workspace、本产品提交自身CI、源Git blob绑定与主spec同步后才能交付。所有本机Rust遵守默认stack、≥5GiB余量、1GiB target上限、每秒守卫及finally精确清理，期间禁止共享修改/Git mutation/网络/人工并行工具链。CF09/71/612保持partial，Smali6继续只记输入边界。
