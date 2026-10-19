## Context

已核当前build.rs：prove_array_update约17153，现有CompoundUpdate::Array无op，严格只准iadd；Builder::array_write固定AssignOp::Add。字段复合赋值已经携带ArithmeticOp并支持Subtract，AST/emit已有减法赋值，无需机制。Atlas索引行号陈旧，root以当前rg与全文为准。原公开失败真实记录指向BCI37dup2/38iaload/41iastore，新增循环Region已实际通过完整Java lib510。

## Goals / Non-Goals

仅受证单槽int[]、非返回值语句的old-rhs减法写。复用相同lvalue、完整读写/dup消费、RHS依赖及预算/事务。非目标：其他primitive、任意重排/跨块、返回值-=、复合表达式值/副作用扩张、未知operator；不重命名旧通用add字段或重构周边。

## Decisions

1. Opcode和Operation必须精确配对：0x60/Add保持旧路；0x64/Subtract只在return_bci=None准入。无乘除/反向减法泛化。不交换old/RHS，不把未知类型当int。四dup输出、同一array/index、每个消费、整前缀/RHS区间及异常顺序现有闭合全部不变。
2. 复用现有ArithmeticOp字段类型，与Field计划一致；Array计划记录已证op，array_write映射Add/Subtract到已有AssignOp。返回值ReturnedArrayUpdate仍仅Add，原调用前筛和新match边界显式保留，不增第二套返回值计划。
3. 同轮实际IR测试读父片原件debug/no-debug，证明现有加法与新减法完整lvalue；至少一个等长真实Code operator变体（例如imul）重读SSA后拒绝未批准operator，不伪造SSA/借frame假装物理反例。量测现有proof预算停止及预取消，无半个计划/statement泄漏。修改字节若未独立JVM验证，标parser/IR-only。
4. root逐差异审查Luna私有实现，禁止顺手扩展其他数组/返回值/新机制；原有CompoundAssignments stop事务和调用者都核。最终公开whole-class必须Structured，保留之前真实失败，不能删原decrement或降低质量断言。
5. 与父片共用最终CLI完整对照：原件/JADX基线精确复用，八API/16完整编译/32VM×8case=256观察及16wrong-root；数组、f、异常和stdout逐条匹配，默认/all与真实maps保持。两个OpenSpec共享一次fresh native与同一产品自身CI全部步骤/双seed/继承具名门，分别同步main spec并严格验证。不把本窄片当CF09/EM或71/612整体关闭。
6. root独占共享/Git/工具链，Luna仅私有tmp/只读；Cargo5GiB/target1GiB/defaultstack/每秒guard/finally exactclean，期间不共享写/Gitmutation/network/manualJDK/CLI。fixture复用，无Reader增量或重复指纹regen。

## Risks / Trade-offs

减法不满足交换律，必须保留old-rhs方向；op必须随计划发射，不得沿用硬编码+=。既有返回值+=不得意外解释减法。完整原件用例能区分减一和加一，旧数组/字段compound及returned-array回归须保留。
