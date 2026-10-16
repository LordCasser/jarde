# Design

## Context

动机见proposal，行为契约见delta。双编译腿的新独立 `LooplessFinally` 均为major52、456B，`loopless([I)I` Code50B；唯一catch-all为[0,16)→37。getstatic返回字段后在15保存，正常清理16..24后24加载/25返回；落穿清理26..34后34 goto48；异常37保存、38..46清理、46加载/47重抛；48常量-1、49返回。三个println调用分别在21/31/43。CP编号可不同，不能按显示文本或原FY字节相同来判等。

当前 `prove_finally_copy` 只从handler前的一个线性return后缀推导正常副本；FY.loopless的尾部实际是另一条清理和goto，静态阅读预计先在该正常后缀假设处拒绝。现已由真实双腿Facts probe核实旧proof返回None和该首拒事实，原raw与SSA/图见instrumentation.md；没有给生产guard加日志。后续即使选到16..24副本，既有protected-normal-edge只允许留在保护区或到16，仍未解释跳26的另一出口。

现有 `Shape::Finally`、`FinallyCompletion::SavedReturn` 与 `Region::finally_body` 只承载单一正常完成；后者在无named catch时拒绝非空next。仅设置Plan.join不足以增加另一个出口。既有 `prove_conditional_finally` 证明的是清理内部null分支，不是本片的受保护条件体；`SharedFinally`要求named catch，本片不能伪造该事实。

## Goals / Non-Goals

**Goals:** 增加最小的私有三副本证书/完成形态，完整绑定实际值、行与块；通过现有bounded Region生成条件体、Try/Return/Finally并回到真实continuation；在原值位置输出return而由Java的finally语义保存待返回值。必要时复用已有临时绑定，不能强制新建temp框架。

**Non-Goals:** 不放宽其它finally规则、不新增通用SSA/CFG等价器或公开API；不修改JVM分析事实；不把可编译、解析成功、dialect、runtime选择或verification报告混为恢复语义成功。父计划与完整FY/FX的剩余失败保持。

## Decisions

1. **先保存物理事实，再有界证明。** 沿同次MethodIr/Facts读取已有canonical、SSA、effects和exception rows，不重跑P2。用固定有限opcode/行/出口形态作候选分流，避免对无关P5形态启动新计费扫描；可变proof扫描、分配与比较仍必须沿既有poll/charge。候选分流不等于证明，不能以少数opcode代替消费者或异常范围核验。
2. **私有完成证书明确两个正常出口。** 记录保存/加载/返回链、返回清理、落穿清理与其transfer/continuation、异常保存/加载/throw、唯一row、owned块和origin。本案仍只有一个saved-return配对，优先复用现有SavedReturn与既有continuation表示，仅给私有证书及已有私有字段的Plan补落穿copy元数据（crate内getter），保持公开FinallyCompletion枚举形状不变；只有实际消费无法表达时才加最窄完成变体。只增加本形态所需数据，不给每条compiler布局造一套通用框架。完整证书前不发布ownership，最终poll后才交付。
3. **复用清理等价与边界。** 使用已有`cleanup_sequence`的operation+本副本SSA producer角色核三份直线cleanup；三份均在受保护半开区间外，无从中间进入、额外异常row、外来正常前驱或未解释效果。所有受保护正常边只能留区内或去两个明确副本入口；保存值只有本次返回链消费，异常对象来自实际handler并同值重抛。拒绝多return、loop、清理内部branch/return以及混入其它消费者。
4. **复用受限Region及原子发射。** bounded walk仅拥有已证受保护块，分别表达return arm和落穿arm并核exact visited/owned集合；清理副本和handler不混入正常body。Try内在实际save位置消费已计算producer，不能在清理后读当前字段。受限body的next应是已证落穿copy入口26，外层Guard完成清理后才经34交回continuation48，二者不能混为一个Plan.join。实际双腿canonical已核实48/49融合在block26，须使用既有Continuation::Tail接缝或相同大小的受限尾部表示，不得认领整块后漏tail或重复walk cleanup；这不是可以窄拒绝掉的未知形态。正常落穿交回实际join。任何不完整child、未声明local、fallback或停止恢复checkpoint，不能把半个结构标成已恢复。正文提交后可选证据遵循既有阶段契约，不泛称撤销已交付兄弟。
5. **独立完整类是正验收，原FY整体是范围对照。** 保留目标cleanup原有println，通过未改driver安装PrintStream计数、改字段和抛专用异常；目标finally不加计数/throw helper。双真实compiler先核Code/row/副本形态，再冻source/class/driver/oracle。全量report.text原样重编，driver只对rebuilt编译，Java8/23隔离运行。原FY所有方法保留；只完成loopless时其noCall仍红，不给删方法留下通过路径。

复用评估：项目现有Rust证明、Region与AST/emitter提供所需分析和发射，维护与许可沿现仓库；引入外部decompiler或图同构库不能提供同次预算/物理身份/停止契约，且非本小片所需，故不新增依赖。JADX若对疑难行为有帮助可单列对照；原class oracle与Jarde完整重建才决定验收，不以另一工具输出代替证明。

## Risks / Trade-offs

- [清理被自身handler保护而执行两次] → 逐指令核半开范围与效果handler归属，driver分别在待return/原异常时让println抛异常；增加错误范围负控制。
- [静态返回字段被清理改写] → 实际getstatic保存值链及唯一consumer核验；driver用3000→99，返回必须3000，不以常量3000样本代替该值身份。
- [一个出口或外来边漏认] → 两正常copy、异常copy、continuation和exact块集合全部在证书中闭合；故意修改copy/row/消费者的真实分析负控制保留原拒绝。
- [新proof改变无关计费] → 候选分流先行，尽早跑未改P5 all-features原快照；不更新计费表、不退款或放大预算掩盖回归。
- [独立类成功被写成父计划完成] → 分别记录LooplessFinally完整rebuild、FY.loopless结果、完整FY/FX重编与noCall/finReturn原因；FO控制沿已验prefix片保留，不增全局覆盖数。

## Migration Plan

先冻结红基线及首拒证据，再落永久fixture/指纹与具名回归，最小实现后完成停止/负控制和兼容门。前片归档交付后重绑定主线，冻结最终CLI并原样重编运行，再以本产品提交自身CI终态日志完成产品门、同步和归档。代码未完成不勾任务，计划/原类成功不算恢复。


## Implemented boundary and intermediate results

生产候选用27个raw opcode固定分流（返回producer为getstatic int或sipush int）；真候选仍核全部同次SSA定义/唯一uses、五个确切canonical块/正常边、唯一异常row/effects、三份cleanup的operation+本copy producer角色、handler入口异常对象同值重抛与全部ownership后才poll交付。临时metadata曾放公开SavedReturn字段，静态核验发现guard为pub模块后已移到Plan私有字段；公开enum保持原样，不以public模型扩展冒充私有变化。

现有bounded walk以真实block26为body.next边界，Plan.join保持None；48/49融合尾使用既有Continuation::Tail，在Try提交的同一checkpoint内发射。保存字段值在try内实际15位置绑定local1并返回；清理呈现一份println。tail若fallback/undeclared local或停止按既有checkpoint恢复，不退款已消费预算；正文提交后的可选evidence仍遵循既有阶段契约。

中间WIP实际private3/public2/CLI1通过；冻结8583a2… CLI的完整生成源码双腿各1225B、两真javac编译与两组Java8/23 rebuilt-only驱动均0，四份134B oracle exact。历史四变更源快照已逐hash绑定到该4922 manifest。随后真实class负控制与兼容67/0/3通过，包括不同String常量、未消费保存值、未重抛primary、额外保护区边、自保护cleanup；P5 all-features计费原断言未改。以上是中间验证，最终固定Git head/重放与自身CI尚待单独验收。

前片已正常归档并交付main7970ca890（产品052 ownCI成功）；本地三未发布历史提交重放到该主线。两Loopless类真实人口增量2class/4Code/2handler/6branch-target-records/0jsr；26在同方法有两条目标记录，不能去重。reader人口预期1177/5263/525/3451/10，只维护cfg(test)人口，不改decoder。
