# CF12 十 fixture 同一新 CLI 回放准备

这是任务 3.2 的只读准备清单，不是 replay 结果。10 个类的原始上游源码与实际 capture input/JADX source 均沿用其已冻结路径和 SHA；正式回放必须另冻结同一个新 CLI 的文件 SHA，并在此 CLI 下重新生成四组对照。旧 `complete-source-root-v1` 和 `cf12-upstream-remaining-root-v1` 仅作基线与断言来源，历史 pass/fail 不能计作新 CLI 的通过。

清单和逐文件 SHA 在 [`jarde-cf12-ten-fixture-replay-plan-luna-v1.json`](/private/tmp/jarde-cf12-ten-fixture-replay-plan-luna-v1.json)。它逐类记录源测试、输入 class 集合、完整 JADX 输出、实际输入路径/参数、JUnit 状态；`TestSwitchLabels` 的 `TestCls` 与 `TestCls$Inner` 必须作为同一测试的两个物理 class 记录。原始源码来自固定 JADX HEAD `2fb1b16386941660fda07e9017285aec40fcb37f` 的 Java tests；captures 位于 `cf12-upstream-java-root-v1/harness-v3-java`（原五类）与 `cf12-upstream-remaining-root-v1/direct-root-v1`（新五类）。

建议从现有 `jarde-cf12-full-replay-root-v1.py` 的 39 命令流程做最小、有限的 fixture 列表改造；新五类可参考 `jarde-cf12-remaining-full-replay-root-v1.py` 的 check 和矩阵定义。不要再跑 upstream JUnit/Gradle 来造输入：十类已有冻结 input class 和完整 JADX source，直接 hash 验证后重放即可。把两个五类列表合成一个显式十项 manifest，并让 runner 接受新 CLI 参数、同一结果根目录、两个 JDK profile；保留原四腿：original class、JADX 完整源码、Jarde default、Jarde all。每腿保存完整编译 class 集、编译/运行 exit、stdout/stderr、`-Xverify:all`、上游 check 状态；两项修改方法还要按物理输入 class 记录 source origins。上游 check 失败时记录原异常，按既有规则不跑该类的 check-dependent probe；无 check 的类使用对应 frozen runtime probe。无须增加通用 runner 框架或重新生成 upstream 字节。

需要的共同 pins（SDK jar、JADX runtime jar、JDK8/JDK23 工具、observer 和 probe）已经写入 JSON 的 `shared_pins`。历史完整回放的 Jarde CLI 为 `/private/tmp/jarde-proved-if-join-cli-v1`，SHA `7b751758…`；另一个 five-class snapshot 使用 `/private/tmp/jarde-proved-conditional-switch-cli-v1`，SHA `39d5699c…`。两者都是旧基线，不能作为任务 3.2 的 new CLI。

三项必须原样体现的对抗性验收条件：

- `TestSwitchLabels` 比 class-set 时 outer 和 inner 生成 class 都须存在，不可只看 `$TestCls.class`。
- `TestSwitch4` 的历史 JADX 真实 check 为 `2234 != 1234`，完整源可以编译但运行断言失败；新 replay 必须保存该失败，不能只因源码可编译就标成通过。
- `TestSwitchWithFallThroughCase2` 的 JADX 真实 JUnit 因 `Code duplicated` warning 失败；完整 JADX 源的 check 与 68 输入矩阵仍同原。warning / JUnit failure 和后续 full-source observation 要分列保存，不能抹掉前者，也不能把 warning 误报为语义差异。

JDK profile 至少是 Corretto 1.8.0_432 与 OpenJDK 23.0.1；两侧统一 source/target 8 完整类重编和 `-Xverify:all`。observer 的官方测试 SDK 与 21 个 helper class 应精确 hash；完整源码重编的 classpath 不可含原 fixture classes。每个 profile 分开记录结果，不把 default 与 all 合并。最终 JSON 的验收状态在十类、两 JDK、四腿及新 CLI hash 全齐前保持 incomplete。
