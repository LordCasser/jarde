# 普通 while 隐式回跳来源：root 验收入口

当前 tasks **2/6**。已接受控制基线及私有源码证明；候选未应用、未编译，不能勾选动态验证任务。原始基线31命令/111文件，原2/JADX4完整类成功，Jarde4编译失败/零运行；`noPrefix` 的 goto@14→6 缺 map 已独立接受为真实来源缺口。详见 one-arm-loop-controls/verification-root.md。

root 全文读审 Luna v2 与相邻 region/build/emit 后，发现新 helper 在 canonical edge 计费前重复调用无计量的 `leaving_edge`。root 私有 v3 只删除该调用；后面的计费、逐边 poll、唯一 Normal→本 header 门禁已涵盖异常/未知/多余边，不改变 shape。header-test-chain 与单条件两个成功构造点均已接入 helper，审查初期的 chain 漏接判断已撤回，不据未应用 main 误判候选。

**当前应用候选是 results/loop-latch-origins-root-v6.patch**，SHA-256 `aedfdfbfa16d6d7c1feb8e7a9be72e242893917bbc433af1ff087a4a2b59f425`。生产证明沿用已审 v4；v5/v6 再增强已有短路循环测试，分别绑定 `andWhile(II)I` 的 derived goto@20 与 `orWhile(II)I` 的 derived goto@19，检查准确 BCI、该物理方法身份和实际非空 while 语句跨度。root 对 v5/v6 实际 apply-check 均退出0，生产/永久测试仍与 main 恒同；记录 results/validation-runner-helper-preflight-root-v3.json。

复用冻结 fixture；完整候选新增函数只有两个 noPrefix 测试，其余均增强既有测试。涉及 region.rs 和四个已有测试文件，无 public IR、Frame、pass 或新增机制。v6 收紧了 v5 的身份断言：不能用同一 segment 的其他派生 BCI 来替准确 latch 的 physical identity 作证。历史 v4/v5 保留。

根 `tests/p3_loop_arm_join.rs` 属于 **jarde** 包，准确命令是 `cargo test -p jarde --test p3_loop_arm_join --locked`。此前只看 jarde-java crate 而误记不存在的结论已纠正。其完整结构化正例直接覆盖同一 latch-origin gate，second-entry/different-exit/额外 break、return、exception 负例保留；p3_effectful_exits 是相邻回归，不能替代它。

root 以旧冻结乘法 CLI 实际执行4条 class-source，核三组既有 transfer anchors 在 if/break span、LoopIfJoin goto@23 无 map。随后用真实 BLAKE3 与 raw SHA 检查物理类、方法 owner、全部来源和准确 span。原始字节及只读接受在 results/transfer-map-old-cli-root-v1；它们只用于设计对抗断言，不是新候选或完整类运行接受。源级尾部 continue 若与隐式 latch 编译成相同字节码不可区分；门禁排除的是已由 Region 显式呈现的 transfer，不虚构不可观测事实。

root 以固定 JDK23 javap 及旧冻结 CLI default/all 实际执行3条命令，确认 AND/OR 两准确 latch 缺 map、两 profile 正文/map相同、现有来源绑定正确。证据 results/header-chain-old-cli-root-v1，只是测试设计观察；已有 LoopBool runtime 测试提取了 AND/OR 两方法，不是含 mixedWhile 的整类接受。

root 全文读审 runner v1、v2/v3 完整差异，实际只读 preflight 接受 **results/run-validation-build-root-v3.py**：17产品/8测试/23当前字面 include pins、CI29lint、**12条命令**、noPrefix 两测试和 Boolean-loop 三测试准确名字/非空 summary。新 NO_PREFIX include 会在生产 patch 应用后再加入闭包。Java 测试固定 controls-v1 JDK23 的三个工具 SHA，清除四个 Java 覆盖环境变量；其余旧11项验证保留。runner v2 的三个分开 exact 调用被收敛成一次完整 target，历史保留。未运行 runner main/Cargo；机器余量仍低于20GiB，root/fuzz target均不存在，新CLI/meta尚未生成。

相邻 CF-07 整类已由 root 用旧冻结乘法 CLI 实际重放29命令/118文件：原2/JADX4/Jarde4整类编译运行exit0，raw相同。输入不删成员，生成源不修改，固定双JDK、fresh classes/empty CP-SP/-Xverify:all。andWhile goto@15→2 缺 map；counted 缺@20→27和@30→6；lastIndexOf 缺@25→5，四profile相同。来源缺口与整类运行各自判断，root 实际独立 verifier-v4 退出0，接受118闭合文件/10完整腿及所有primary+derived物理来源；准确接受文件是 cf07-loop-latch-baseline/independent-verification-root-v3.json；不能据此声称新候选已通过或全部来源完整。

资源满足后的顺序：在干净检查点 root 应用 v6、格式化，以该 HEAD 运行 runner v3；12项真实验证成功才勾2.1/2.2。冻结新CLI及源码pins后执行已审 prepare-candidate-luna-v2.py，双JDK/default-all重新核 noPrefix 准确来源增加、其余正文/map保持及四整类真实编译失败；另以 CF-07 完整类检查相邻正常运行未回退，并核新增来源只落入已证普通直线末尾 latch。捕获精确产品自身CI后完成3.2。单臂片仍1/7，71/612分母和整单元完成数不变。

历史版本均保留：test-only v1 实际 apply-check 退出128（hunk计数损坏），v2从真实base以difflib重建并实际check通过；早期私有READMEs的建议目标以本入口纠正结果为准。v3初次内存准备的整文件计数断言失败发生在写输出之前，之后限定新helper生成，不曾写入生产源码。其它架构债务不混入本片。
