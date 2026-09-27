# CF-08：单出口循环臂的局部汇合实施证据

本分支基于主线 `31586ffc`。固定 `LoopIfJoin.java` 用 `javac --release 8 -g:none` 重建后，class SHA-256 为 `542c856a156103f8c4e20372fcbeb3bc767b0c706f3270d73d5ec6dddfefdc2a`，与 [基线](../join-baseline/report.md)相同。实施前的 Region 探针显示：外层 BCI 0 的 join 为 34；内层 BCI 6 的 join 为 26，父 arm 边界为 34；内层一臂是 `Straight [10]`，另一臂是完整的 `Loop { header: 15, body: [20], exit: Some(26), gateway_origins: [] }`。因此可以在已有 Region 上验证唯一出口，不需要放宽局部声明规划。

实现只在一个 Loop 臂与一个 Straight 臂的形态下检查循环 Region 出口、natural-loop 块与 Region owner 完全相等、各 owner 已访问且在当前 scope 内、循环只有来自内层分支的一个入口和通往内层 join 的一个正常出口。额外转移、非正常边及不同出口不取得该证明。原有两个 Straight 臂路径仍由同一尾部检查处理。后续直线尾部的 canonical 入边、逐块出口、来源和父边界检查仍适用。构建器的中间 join 条件值候选现在只匹配它原有证明实际支持的双 Straight 子臂，因此普通循环语句不会被误报为条件值桥。

当前 CLI SHA-256 为 `7de17430b2995643b377821f89cda38f0a2aa6a05177e48b849ad9a0292419a7`。完整 [Jarde 源码](LoopIfJoin.jarde.java) SHA-256 为 `b249d4214bccd4b3968cd66582d26cdb61bed40f8d231e95b128291b06fd2999`。原、固定 JADX 与本次 Jarde 三份完整类分别用 `javac --release 8 -g:none` 重编，经 `java -Xverify:all` 均输出 `4 / 13 / 11`；`run` 无 `@bytecode`。源码中内层判断、BCI 15 循环条件、BCI 20 更新、BCI 26 外层 `+10` 和 BCI 34 返回各出现一次；相应非结构性转移 BCI 12/23/29 的目标由固定 `javap` 和 Region owner 证明，未伪造可见语句。

定向 `p3_loop_arm_join` 4/4 通过：正例检查来源及语句次数；Verifier 有效的第二循环入口与不同出口、额外 break/return/异常边保持引用；预算与取消不发表半份完整类。`p3_forward_join` 3/3、`p3_switch_forward_join` 3/3、`p3_loop_transfers` 5/5、`jarde-java` crate 测试通过。已验收 CF-08 简单双出口、CF-07 与 CF-09 的原/JADX/Jarde 完整源码再次 Java 8 重编且 `-Xverify:all` 输出分别一致：`0 / 1 / 2 / 3 / 3 / 3`、`0 / 10 / 39 / 63 / 9 / 3 / -1 / 3`、`Grid` 五行和 `OuterContinue` 五行。原 `NotIndexedLoop.test` 仍带 `@bytecode`，未纳入本首片。

此文件记录分支实施态；主线合并后的检查由接收分支者补录。
