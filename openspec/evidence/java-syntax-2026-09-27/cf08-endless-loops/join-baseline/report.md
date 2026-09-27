# 外层 if 中循环臂的局部汇合：独立首片

这是 [CF-08 剩余缺口定位](../remaining-gap-architecture-2026-09-27.md)的第一个隔离样例，不含多出口或带效果的循环退出。固定 JADX checkout 是 `2fb1b16386941660fda07e9017285aec40fcb37f`，Jarde 主线是 `727e744f`，CLI SHA-256 为 `9518aa8e0d5e34ab6d5ce71b6222db5a38b009ebea6467a228553d3db44cc4f9`。输入 [LoopIfJoin.java](../input/join/cf08join/LoopIfJoin.java) 的 SHA-256 为 `a383cab3b0a8d4ee36ca71e7120b2f2914729f161008f0d3c398d1a9b26ae5d3`；用 `javac --release 8 -g:none` 编译出的 class SHA-256 为 `542c856a156103f8c4e20372fcbeb3bc767b0c706f3270d73d5ec6dddfefdc2a`。

原 class 与固定 [JADX 完整源码](LoopIfJoin.jadx.java)分别以 `javac --release 8 -g:none` 重编，`java -Xverify:all` 均输出三行 `4 / 13 / 11`。Jarde [完整源码](LoopIfJoin.jarde.java)在 `run` 被引用并缺少返回语句，Java 8 编译失败。`JRE_JOIN_PROBE` 显示外层分支 BCI 0 的 join 是 BCI 34，内层 BCI 6 的 join 是 BCI 26；[javap](LoopIfJoin.javap.txt)显示内层真臂 BCI 10–12 跳至 26，假臂是头部 BCI 15、更新 BCI 20 的单出口循环，也在 26 结束；BCI 26 的 `iinc 2,10` 再跳到外层 join 34。Jarde 报告 `jre_region_arms_do_not_meet`（block 0），而非循环出口网关失败；因此这份样例只隔离出嵌套 `If` 的循环臂续接缺口。

`continue_inner_join_arm` 目前只接受内层 `Region::If` 的两个 `Region::Straight` 臂，且尾部也必须是直线段。循环臂的物理正常出口虽到 BCI 26，却不能通过此门；实施时仍须检查现有 `Region::Loop.exit` 是否保存了同一目标。修复应在现有 Region 与 canonical CFG 事实上证明内层两个臂的唯一出口、BCI 26 的全部正常入口和 BCI 26→34 的独占直线尾部，再将尾部留在外层臂中；不能直接忽略拒绝或放宽局部变量绑定。原 `NotIndexedLoop` 的带效果双出口仍属另一个待审边界，不作为本首片的通过条件。
