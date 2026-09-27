# DT-31 grouped enum switch acceptance

日期：2026-09-27。命令入口见同级 `../replay.py`；执行产物的来源快照、编译/运行日志、证明 JSON 和汇总位于本目录。所有临时 `.class` 只存在 replay 的 `TemporaryDirectory` 中，没有纳入证据。

- JDK：OpenJDK 23.0.1；输入与三方源码均以 `javac --release 8 -g:none` 编译，运行均使用 `java -Xverify:all`。
- 固定 JADX：`2fb1b16386941660fda07e9017285aec40fcb37f`；通过 `jadx -d <temp> grouped-input.jar` 重建源码。
- 当前 Jarde CLI SHA-256：`009a23c4f7b5bd13563f5cc561203e234b3c9a29a79df452737fe9fc8759f5d0`。
- 冻结单站点对比 CLI（DT-25 已接受二进制）SHA-256：`7bd7ebe1fd0837cd8ca552380361c23a006d4bb3a7818ed21850c66f5e290678`。
- 双表正例 JAR SHA-256：`8df2088c16499b1086ad9e65f516d166603be26d67c229083f5bbefd87fd3ea5`；改坏 Animal DOG key 的证明负例 JAR SHA-256：`949fb81a621fcf57f2ccc2e2e564fb97793f3d87e9bd50de4abbff9ef1dce4c5`。

原始、固定 JADX 和 Jarde 的完整 grouped 源码都通过 Java 8 编译及 `-Xverify:all`，输出与副作用 trace 完全相同。正例涵盖六种 `Count × Animal` 组合以及分别为 null 的两个 selector。负例只改 Animal 映射的 key，使它与 CAT 重复；两条证明记录均标为未投影，且 Count 记录说明其因 Animal proof 被拒绝而 withheld。单站点稳定性用冻结审计输入重建 JAR 与已接受 CLI 比对；SnapshotId 归一后源码逐字节相同。

预算/单表回归：对 `single.Subject` 输入，以 `output_bytes` 二分搜索完整 class-source 请求的成功下界。旧接受 CLI 与当前 CLI 均为 4467 bytes；两者在 4467 成功、4466 拒绝，高预算完整文本 SHA-256 均为 `fdf7c3fb3b17eb8b5f270acbe3162ca9e13ad598e8701c64bb59c8be3337eef2`。旧/新 proof 都按完整 helper `<clinit>` 的物理指令数收取 `IrItems`，联合实现未增加单表扫描预算。拒绝消息由现有 class-source 测试覆盖；本次阈值实验比较的是请求是否成功以及成功文本，不把易变耗时字段作为逐字节证据。
