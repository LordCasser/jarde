# 普通 while 隐式回跳来源：root 验收入口

当前 tasks **2/6**。已接受控制基线及私有源码证明；候选未应用、未编译，不能勾选动态验证任务。原始基线31命令/111文件，原2/JADX4完整类成功，Jarde4编译失败/零运行；`noPrefix` 的 goto@14→6 缺 map 已独立接受为真实来源缺口。详见 one-arm-loop-controls/verification-root.md。

root 全文读审 Luna v2 与相邻 region/build/emit 后，发现新 helper 在 canonical edge 计费前重复调用无计量的 `leaving_edge`。root 私有 v3 只删除该调用；后面的计费、逐边 poll、唯一 Normal→本 header 门禁已涵盖异常/未知/多余边，不改变 shape。header-test-chain 与单条件两个成功构造点均已接入 helper，审查初期的 chain 漏接判断已撤回，不据未应用 main 误判候选。

**当前应用候选是 results/loop-latch-origins-root-v4.patch**，SHA-256 `c42c69662cb8206f702d8d682cfd5567f1eaabc017faa2fd9bc0241d30b9d8df`。它组合 v3 源码证明及 Luna test-only v2：保留 noPrefix 的全部11个准确 BCI/physical owner/default-all/非空 while span/Stop 正例；在既有 LoopIfJoin 正例增加 goto@23 的来源断言，并在既有 double-jump anchors 上补非空来源及不得误挂 loop span 的断言。只涉及 region.rs 与三个已有测试文件，不增加 fixture、测试函数、public IR、Frame、pass 或机制。root 实际 git apply --check 退出0，生产/测试 diff 为空；记录 results/private-review-root-v3.json。

根 `tests/p3_loop_arm_join.rs` 属于 **jarde** 包，准确命令是 `cargo test -p jarde --test p3_loop_arm_join --locked`。此前只看 jarde-java crate 而误记不存在的结论已纠正。其完整结构化正例直接覆盖同一 latch-origin gate，second-entry/different-exit/额外 break、return、exception 负例保留；p3_effectful_exits 是相邻回归，不能替代它。

root 以旧冻结乘法 CLI 实际执行4条 class-source，核三组既有 transfer anchors 在 if/break span、LoopIfJoin goto@23 无 map。随后用真实 BLAKE3 与 raw SHA 检查物理类、方法 owner、全部来源和准确 span。原始字节及只读接受在 results/transfer-map-old-cli-root-v1；它们只用于设计对抗断言，不是新候选或完整类运行接受。源级尾部 continue 若与隐式 latch 编译成相同字节码不可区分；门禁排除的是已由 Region 显式呈现的 transfer，不虚构不可观测事实。

root 全文读审 results/run-validation-build-luna-v1.py，并实际只读 preflight 核17产品/7测试/21当前字面 include pins、CI29lint、11命令及精确新测试名/空测试拒绝，结果 validation-runner-helper-preflight-root-v1.json。新 NO_PREFIX include 会在 patch 应用后纳入闭包。未运行 runner main/Cargo；机器可用空间约11GiB，低于20GiB机器/1GiB target守卫，root/fuzz target均不存在。候选 CLI 路径是 /private/tmp/jarde-loop-latch-cli-v1，metadata 是 results/candidate-cli-v1.json，尚未生成。

资源满足后的顺序：在干净检查点 root 应用 v4、格式化，再以该 HEAD 运行上述 runner；11项验证成功才勾2.1/2.2。冻结新CLI及完整源码 pins后执行已读审 prepare-candidate-luna-v2.py，用双JDK/default-all重新验收 noPrefix 唯一准确来源增加、其余正文/map保持及四整类真实编译失败。再捕获精确产品自身CI完成3.2。只有后续独立单臂续接片成功，才可声称这些完整类运行接受。71/612分母和 EM23 整单元完成数保持不变。

历史版本均保留：test-only v1 实际 apply-check 退出128（hunk计数损坏），v2从真实base以difflib重建并实际check通过；早期私有READMEs的建议目标以本入口纠正结果为准。v3初次内存准备的整文件计数断言失败发生在写输出之前，之后限定新helper生成，不曾写入生产源码。其它架构债务不混入本片。
