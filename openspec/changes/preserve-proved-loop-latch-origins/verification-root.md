# 普通 while 隐式回跳来源：root 验收入口

当前 tasks **5/6**。局部来源实现、永久正反例、fresh CLI 对照与精确产品CI均由root实际独立验收；剩余3.2的主线检查点交付。单臂续接仍是另一项工作，不计 CF-07/EM-23 整单元完成。

2026-10-10 10:31 UTC，root 完整执行 `results/run-validation-build-root-v9.py` 的12条命令，fmt、CI同范围Clippy及552 passed/0 failed/1 ignored全部通过。根 arm-join 4项、double-jumps 7项、Boolean-loop 3项均实跑，gateway 5项含 noPrefix 来源和预算/取消。源前后49个pins恒同，target峰值760681447 bytes；机器5GiB/target1GiB守卫全程未触发。fresh CLI `/private/tmp/jarde-loop-latch-cli-v1` SHA-256 `1728ef3fa1a3a9e8ed56d6f4384a1c54c3e63400e6495c750a0bd139d7623b5c`，metadata SHA `6b0c59d452d8d5a1fb157c60e81a6602f9cc78c26479f2638e98bdfda54df9e3`，准确身份以 `results/candidate-cli-v1.json` 为准。

10:35 UTC root 实跑独立 verifier-v5退出0，接受结果 `results/candidate-acceptance-root-v5.json`，完整argv/raw/hash在 `results/independent-execution-root-v5`。Plain真实31命令/111闭合文件：noPrefix仅增加derived@14的完整while来源，所有已有来源、全部方法正文、其他方法map与基线恒同；全部物理owner/方法/BCI、default/all均核。三个旧单臂拒绝仍存在，Jarde四份完整类编译失败、零运行，未删除成员或修改生成源。CF-07真实29命令/119闭合文件：原2/JADX4/Jarde4完整类全部原样重编和验证运行成功，raw与原oracle相同；仅andWhile@15和counted@30新增准确while派生来源，旧来源/default-all保持。counted@20与lastIndexOf@25仍在范围外，不能据运行成功冒称全BCI完整。

实际应用的五文件patch为 `results/loop-latch-origins-root-v7.patch`，SHA `3b5aa370bc52193eda4ea7c491930fc59d13ebfee3203d33e30192f7e6dc4c1c`。生产证明沿用已审v6；root依据真实失败把测试helper的参数槽数改为官方descriptor_facts，而非硬编码1，并在essential请求显式加入SourceMap。optional evidence语义与生产算法未因此改变。新增实体仅一个私有来源证明helper，复用现有自然循环、canonical/SSA、gateway_origins和emitter，不加public IR、Frame字段、pass或依赖。

资源下限由用户明确授权20→5GiB，实时中止与完成清理保留。v4/v6真实因机器余量不足中止；v5因测试facts错误失败；v7摘要tuple/list假阴性及v8清理后空target错误均保存，未冒称接受。v9从空target重新完整执行，没有借旧成功行补齐结果。v4独立verifier的BLAKE3 callable/module类型错误raw保留，v5仅修调用形式后重新完整验收。CF首次root调用缺SHA参数在任何重放前拒绝，修正调用后实际完成。所有历史版本和原始证据保留。

10:37 UTC root仅cargo clean本仓，释放725.4MiB/3436files，target不存在，冻结CLI保留；记录results/root-clean-five-gib-v2。OpenSpec全量strict实际337 passed/0 failed，git diff --check通过。辅助14工作树仍detached，无分支占用，受应用固定保护的副本保留。

11:34 UTC root独立CI verifier-v3退出0，准确接受404b422e141e200f0eea000f64937becedbad664自己的CI38045578457：4jobs/52steps，双固定seed各3379 passed/0 failed/97 ignored、354记录；每seed核2个来源测试与11个旧整数回归，Temurin25完整类/MSRV/fuzz/supply全部成功。API、完整GH stable/supply日志、raw/hash与acceptance-loop-latch-v3.json均在results/ci-product-v1。验收时root逐字恢复两文件到404b Git blobs、重新核49live pins；完成后逐字恢复下一片candidate并重新核50pins，执行记录verification-execution-root-v4。

CI verifier-v1实际因stdout/stderr交错把interface summary截断而失败；v2只局部以stdout running marker至首summary并核四个exact test names，旧SHA-pinned helper文件未改。首次v2调用argv索引错误及随后gateway路径采用仓库路径而非Cargo实际tests/路径的失败均保留。root v3只修两处日志label后重新完整验收；总计数、精确产品blob、工具及每seed门禁未豁免。

## 历史分析和准备（以下状态由上面的实际验收结果更新）

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

CF-07 fresh candidate 重放入口现已准备：results/README-cf07-candidate-root-v4.md。root 全文审查 v2 及 v3/v4 差异，v4 SHA `91426f23deebdb5be099f02f25fa5d2c1f5f321114db4397b3532b299722f823`，复用已审 collector 完整 main 流程，仅覆盖新 CLI/meta/output。root 实际 helper 预检退出0，闭合旧118文件；真实旧基线的观察校验准确拒绝缺失 andWhile@15。v3 helper 观察曾因 tuple/string method key 错配实际退出1，历史保留。全部是工具准备/只读控制，未跑新候选29命令；新 CLI、candidate output、Cargo target 仍不存在，tasks仍2/6。
