# 普通单臂循环对照验收

root 实际执行 collector-v1，31 条命令、111 个闭合文件；原双 JDK 两份完整类与 JADX default/none 四份完整类均原样编译、运行成功，exit/stdout/stderr 与原 class 逐字一致。Jarde default/all 四份完整源码均因三个方法缺 return 而编译退出1，没有候选 runtime。原始 raw、源/class 与文件清单保留在 baseline-root-v1，真实 collector argv/hash/三流见 collector-execution-root-v1。

root 完整读审 verifier-v1 及 v2–v5 全部 delta 后实际执行正式入口。最终 v5 退出0，接受 results/independent-acceptance-luna-v5.json：准确双 JDK class BLAKE3、五个物理方法的身份/flags/BCI、111 文件闭合、31 命令、六份成功运行 raw 和四份真实编译失败。真实执行与三流在 results/independent-execution-root-v5。v1 的 policy 字段、v2 的 JDK schema、v3 的 BCI 字符串键、v4 的 flags 类型错误及其实际失败文件不改写；Luna v5 只读 preflight 不替代 root 正式验收。

`prefixWhile`、`loopAndTail`、`takenArm` 均为 branch0 的 explanation-only / ArmsDoNotMeet，完整 quoted BCI 与 physical method 来源准确。`noPrefix` 的正文已有 if/while/return，方法报告 structured；双 JDK/default-all 映射恰缺原指令 goto@14→6。它尚无成功整类运行证据，不能把 structured 报告当作语义接受。默认/all 的完整文本、逐方法正文与 source map 一致；最终 candidate 来源门禁不能豁免14。

下一片拆为 recover-prefixed-one-arm-loops 与 preserve-proved-loop-latch-origins，两套 OpenSpec 均4/4规划完整并 strict 有效，不代表实现已完成。前者1/7：原形和本控制基线已接受，带 method 身份的临时诊断未应用/编译。后者2/6：基线接受、Luna private-v2 已全文读审与 git apply --check 通过，仅 region.rs 局部 latch proof/双臂 origin gate 与既有 gateway 测试的候选；生产源码仍与 main 恒同，未应用或实跑。原始审计的“源码正确”措辞只指结构观察。

root 相邻读审确认 loop_arm_join_source 的非空 origin 拒绝需精确区分已证单一 latch 与未知/exit gateway；保留原自然循环 owner、closed body、入口/出口和 Frame 门禁。专门 loop-arm-join 回归实际位于根 tests/p3_loop_arm_join.rs，属于 jarde 包；后续必跑 cargo test -p jarde --test p3_loop_arm_join --locked。此前只查 jarde-java crate 而误记不存在的结论已撤回；p3_effectful_exits 仅是相邻 effectful 回归，不能代替根 LoopIfJoin 正反例。其它相关 target 是 jarde-java 的 p3_loop_exit_gateways、p3_loop_body_double_jumps、p3_loop_terminal_return。

本仓 target 不存在；机器空间低于既定20GiB守卫，未执行新的 Rust 工具链。候选来源重放脚本准备与未运行范围分开记录，后续冻结 fresh CLI 才能执行。EM23 仍部分完成，JADX 71 单元/612 测试文件分母与整单元完成数不变。
