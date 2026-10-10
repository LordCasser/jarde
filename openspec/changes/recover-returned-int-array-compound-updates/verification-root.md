# Root验收：返回 int 数组复合更新（7/7完成）

当前任务7/7。产品 `6ddc77d5cc2a6fcf098aad40998f6d2602731c59` 已推送 main，确切 CI 38001720578 已由 root 按完整日志、冻结源码/CLI/canonical 身份独立验收。下面保留此前实际失败、局部通过和待验收阶段，最终结果以末节为准；后续新增 CF16 公共测试不借此产品 CI。

## 已实际执行

- v1 patch SHA13c4aa858adf316be2170154f03bd6e880da6cda3799d58b5eb7fbed0f1bb74f与apply check已核对；保持原patch不改。root纠正降序Slot模式（stack_operands升序）、sum_input借用、私有frame模块引用、遗漏的report返回锚点及facade initializer遍历；后者拒绝移动赋值表达式。dup_x2在现有decoder为Other，只有0x59为Duplicate；移除错误语义标签gate，仍按物理opcode/SSA输出身份和准确consumer证明，不扩decode。
- 新依赖集合合并/clone/ownership/quote收集按实际规模预先计费并传播Stop，旧ordinary单用途路径保持；scope限定int返回与准确同块最终窗口。workflow两条ignored执行各指向准确测试，避免重复运行returned。
- root-focused-v1/v2是真实compile失败（私有模块/两个遗漏walker），root-focused-v3实际3pass/3fail/2ignored（错误Duplicate标签阻断新形状）；全部保留。root-focused-v4实际6pass/0fail/2ignored，包含旧statement/merged与新returned完整结构/真实来源、bool拒绝/char接纳、公开late budget和pre-cancel。只证明公开边界，不称internal checkpoint-pressure通过。
- 空间再次低于20GiB后，root-java8-v1的Cargo命令在preflight未启动，不计成功。root另执行已构建的root-focused-v4测试二进制，无新Cargo/Rust构建：root-prebuilt-java8-v1和root-prebuilt-java23-v1各2pass，旧nested和新returned完整类、两版class输入均全source空CP/SP重编，只新classes-Xverify:all执行且raw oracle一致，合计8次完整类比较。二进制SHA、构建记录SHA和对应真实JDK工具身份均记录；这是该历史编译artifact的证据，不能当当前新CLI冻结/3.1完成。artifact已保存在/private/tmp/jarde-returned-array-test-root-v4，见preserved-test-artifact-v4.json。
- canonical完整source/Runner/oracle及10 class输入按字节复制自root验证baseline与typed controls，来源见tests fixture/copy-manifest-v1.json。常驻测试不依赖OpenSpec结果布局。
- root只清本仓309MiB编译缓存，保留上述测试artifact、新旧CLI、源码与全部raw；机器仍约16GiB，以实时df和20GiB构建守卫决策。

## 最终门禁待完成

2.2和3.1现已完成，后续章节记载实际对抗和完整重编结果。3.2/3.3等待当前产品提交的确切CI：workspace两个完整seed、MSRV、CI-exact Clippy、真JDK25、fuzz/supply全部必要job/steps须实际成功。局部通过和历史561绿灯不替代它；本机全仓磁盘中止/预检未启动不计通过。

## 额外消费者来源复审

root已真实执行consumer-controls-root-v1六次双JDK-Xverify:all加载/反射验证，仅加载不调用target；53独立变更/成员不变checks见consumer-controls-root-verification-v1.json。root-focused-v5实际6pass/1fail/2ignored，失败为Luna新增断言错误要求每条BCI映射。root另用冻结561 CLI和新候选CLI对六份完整class进行逐字对照，scalar拒绝文本均字节一致，见consumer-source-map-history-v1/current-v1。两个额外consumer控制实际锚点2,3,6,7,8,9,10，long-return为2,3,6,7,8,9；明确保存copy/read/store/return/额外消费。稳定参数load0/1/4由方法声明命名，既有deferred_producers不报价，不能要求source_map覆盖全部Code。

独立债务：既有fallback的writes_no_statement不收Arithmetic，拒绝链的iadd@5未被映射。历史561已有此行为，本片失败证明不创建ArrayAssign，未回退；本片不扩大通用fallback producer机制。成功returned compound仍必须保留iadd/read/copy/store/return及左值/RHS全部真实来源，既有正例断言保持。新negative仅改为明确既有锚点集合，保留禁止return +=及显式@bytecode检查；不把v5失败擦除或计为通过。

局部Cargo约束已明确修订：实测focused冷构建约309MiB，仅该test及CLI目标用1GiB target/2GiB机器余量守卫run-root-gate-v3.py；全仓依旧20GiB。root-candidate-build-v1实际成功，10相关产品源与新CLI冻结在candidate-cli-v1.json；新CLI尚待完整重编/执行和最终门禁。

root-focused-v6实际7pass/0fail/2ignored，随后加入不合法dup2_x2类别控制并fmt，root-focused-v7实际8pass/0fail/2ignored。无合适类别输入的dup2_x2由两套JVM都报告VerifyError，Jarde准确停止于ir_frame_inconsistent/frames表缺失，无Java/source-map发布；它只证明失败路径，不冒称合法Java语义负例。成功/额外consumer/boolean与char/merged/不同左值/long返回及公开预算预取消均已覆盖，2.2完成。

root-reader-census-v1实际运行全canonical class读取后旧pin失败，实测(1087,4830,463,2715,8)：16份完整11成员控制共176 Code增量、其它计数不变。只按实测更新reader测试pin及增量说明，root-reader-census-v2实际1pass。没有按猜测修改读者计数，历史失败保留。

## 候选v2完整验收（3.1完成）

新CLI /private/tmp/jarde-returned-array-cli-v2 SHA71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110，10产品源/5测试源/canonical22文件身份见candidate-cli-v2.json，root-candidate-build-v2实际exit0。root双JDK真实执行完整11成员全部source与固定Runner，显式空CP/SP，只新classes-Xverify:all运行且exit/stdout/stderr逐字与历史原程序一致，新片2/2。独立root verifier v2实际425checks/0errors，核对完整物理成员身份、source-map的真实dup2/iaload/iadd/dup_x2/store/return及左值/RHS原始BCI。Luna verifier v1错误禁止正文中的普通单词bytecode，误拒绝所有标准来源header；原失败保留，root v2准确检查@bytecode拒绝marker和structured/java状态，并增加物理成员身份校验，不减成功来源要求。

旧nested/P02四完整腿新CLI fresh4/4，独立479checks/0errors；旧24完整控制腿新CLI fresh24/24，独立1231checks/0errors。新旧30腿均实际重编执行；原程序/JADX复用逐hash历史对照，不称fresh。root-java8-v2/root-java23-v2当前test各2ignored pass，旧nested+新returned各两class输入共8次完整类比较，区别于历史v4预编译artifact执行。

官方fingerprint5pass/1ignored、P5严格计费5pass/1ignored，reader实测pin复验1pass。fmt/OpenSpec全strict/source diff均实际exit0。root-workspace-seed1-v1在冷构建时磁盘跌破20GiB，自己的进程组被终止，未进入任何测试，不能记作全仓通过。只清本仓1.4GiB部分target，冻结CLI、测试artifact/raw不删；root-msrv-v1因随后再次低于20GiB在preflight未启动。最终workspace双seed/MSRV/Clippy/fuzz/supply/真JDK25门禁须以本产品确切CI实际成功补验。

脚本范围纠正：Luna适配旧回放时越界修改4份历史脚本；root保存script-agent-out-of-scope-correction-v1.patch/json后恢复准确HEAD。只有当前change新副本执行新CLI，历史证据仍可重放，不改旧验收输入。

最终当前源码focused v8实际8pass/0fail/2ignored，命令前10源/5测试源身份与CLI v2一致。先前v7的格式差异不借作最终身份测试；v8独立编译执行后才建立提交检查点。

空间回升后CI-exact Clippy实际执行成功：root-clippy-v1完整workspace/all-targets/all-features/locked，使用ci.yml当前所有allow flags及-D warnings，exit0，最低余量26267303936字节。区别于之前磁盘未跑MSRV/停止workspace记录。

root-msrv-v2不是代码编译失败：当前PATH cargo不是rustup shim，拒绝+1.88.0指令，exit101原错误保留。root-v3改用明确rustup run 1.88.0 cargo执行相同workspace/all-targets/locked检查，不换MSRV版本。

root-msrv-v3实际workspace/all-targets/locked检查成功，明确rustup run 1.88.0 cargo，exit0，最低余量24528109568字节。先前两个v1未启动/v2错误入口仍保留。当前产品MSRV/Clippy均已本机实测。

第二次全仓root-workspace-seed1-v2实际冷构建120秒，target增长到4.1GiB，再触20GiB守卫且仍未进入测试。root清理本仓6780文件/4.1GiB，恢复4367175680字节；冻结CLI/raw仍保留。不继续第三次同条件全仓冷构建；本机仅补受影响Java-lib和邻近目标，完整双seed由确切新产品CI实测。

受影响Java-lib实际325pass/0fail，邻近四integration目标实际15pass/1ignored；源/类型/字段/输出walker和旧compound/capture/constructed-array/boxed-widening均无回退。本机完整成功门禁与结果hash汇总在local-root-acceptance-v2.json，workspace两个磁盘停止记录明确保留，不能计为通过。

## 确切产品 CI 与提交验收

[CI 38001720578](https://github.com/LordCasser/jarde/actions/runs/38001720578) 准确绑定产品 `6ddc77d5cc2a6fcf098aad40998f6d2602731c59`，四 job、52 steps 全部成功。两个独立固定 seed 各为 354 test-result records、3353 passed/0 failed/97 ignored；日志中八个 nested/returned ordinary 测试均明确成功。真 Temurin 25.0.4+7.0.LTS 上的 returned ignored 1pass、nested ignored 1pass、BigDecimal ignored 2pass，及其余 Java/Clippy/MSRV/fuzz/API/tree/OpenSpec 门禁全部成功。固定 cargo-deny 0.20.2 对 root/fuzz 的 advisories/bans/licenses/sources 四政策各通过。

完整 API JSON、stable/supply 原始 gzip 日志与 `ci-product-root-acceptance-v2.json` 保存在 results。root 实际运行 verifier v2，核对不可变 Git blob 的 10 产品源、5 测试源、22 canonical 文件及新 CLI v2 身份。v1 首次核验因 gh 日志用字面 `^[[32m` 颜色序列而无法匹配 `advisories ok`，失败记录保留在 `root-ci-verifier-v1-failure`；v2 仅准确移除两种颜色表示，全部语义条件不变，原始日志不改。本片 3.2/3.3 完成，后续 CF16 测试检查点须接受自己的 CI。

产品提交后清理本仓 467.2 MiB/722 文件，真实命令/raw 在 `root-clean-submission-v1`；冻结 CLI 与全部证据保留。71 单元、612 JADX 文件分母及 DT26/EM18 整单元分类不变。
