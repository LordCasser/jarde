# HANDOFF — jarde 主线接续入口

用户明确要求继续在当前聊天推进，不暂停持续目标。root负责架构、OpenSpec、真实Java/JADX对照和对抗验收；确定性实现由Luna完成。按JADX的71单元账本逐片推进，先追平既有能力，再探索额外场景。

## 当前状态

已验收产品为 **561de209531c021a9d8adb7c979bae5a57d6c3fb**，已推送main。[确切CI37994276707](https://github.com/LordCasser/jarde/actions/runs/37994276707)四job/51steps全部成功；两轮fresh workspace seed各354test-result记录、3348passed/0failed/96ignored，新增nested/BigDecimal真JDK25完整类执行、MSRV、Clippy、fuzz及固定cargo-deny0.20.2 root/fuzz四政策全部通过。原始JSON、完整日志及冻结source/CLI身份已由root独立核对，入口见[nested root验收](openspec/changes/recover-nested-int-array-compound-updates/verification-root.md)。

当前产品之后的验收/规划提交不改变产品源；新的returned片尚待最终CI验收，不借本次CI。

```sh
git status --short
git log -1 --oneline
git rev-parse HEAD origin/main
git branch -av
git worktree list
```

验收/规划检查点38a9a142a已推送main。returned产品、测试和canonical fixture现已完成局部验收并进入产品提交检查点，工作树状态以git status为准；新CLI及局部门禁已过，确切新CI未验收。上一片561冻结CLI仅代表已验收历史产品。

## 已完成：nested int数组复合更新，7/7

[recover-nested-int-array-compound-updates](openspec/changes/recover-nested-int-array-compound-updates/verification-root.md)恢复DT-26 P02 lambda helper的`t[0][0] += i`。原始`[[I`沿既有`array_of_value`经aaload可证行为int[]；删除对dup2 store-copy的冗余类型gate，保留原始行类型、四份准确copy identity、唯一消费、顺序/依赖、预算/Stop。不新增pass、AST或类型传播服务。

冻结CLI为`/private/tmp/jarde-nested-int-array-cli-v1`，SHA **8745071312981d4eafb8fbb738e6a1fa1edbb55753062bfae057e879c494bb7a**。新完整类四腿真实JDK8/23空CP/SP重编、只运行新classes且-Xverify:all，4/4与原始raw流一致，root独立474checks/0errors；旧24控制腿fresh24/24，root独立1226checks/0errors。original/JADX对旧24腿采用逐hash历史基线，不声称本次fresh。

最终Rust边界/public预算/预取消及邻近回归14pass/2ignored，官方fingerprint5pass/1ignored，P5严格pins5pass/1ignored；325 Java-lib、新reader census(1071,4654,463,2715,8)、本机root/fuzz cargo-deny四政策也通过。fingerprint只新增11输入，原2029条不变。详细source/命令/raw/hash在results/local-root-acceptance-v1.json及verification-root。以前17GiB未执行记录保留为历史；上述是空间回升后实际补验。

3.2/3.3已完成：results/ci-product-root-acceptance-v1.json接受确切四job/全部steps、准确双seed、新nested ignored1pass与BigDecimal ignored2pass、真Temurin25.0.4+7.0.LTS与冻结source身份。supply独立CLI0.20.2安装与root/fuzz四政策成功；日志中的数字429只是时间戳，修正见ci-supply-http-status-correction-v2.json。历史未跑/失败记录保留。

## 已完成：BigDecimal→Number，7/7

[recover-bigdecimal-number-widening](openspec/changes/recover-bigdecimal-number-widening/verification-root.md)只增加release-8 NUMBER_FAMILY中的准确BigDecimal→Number行，不改concat机制。完整24腿从22/24增至24/24，生产/双JDK/full-class与本地严格门禁已过；两个旧负例的过时预期已精准更新，AtomicInteger仍拒绝且来源保持。

旧6997产品CI37989319644确实失败于过时断言，原日志保留。修正提交8cd8c4f的CI37991578328 stable/MSRV/fuzz成功，supply在checkout前DockerHub HTTP429失败；仅重试该失败任务后仍429，不继续循环重试。ci-run-v2/v3、完整stable及重试supply日志在本片results。不能把旧run称为success。

561组合产品四job全绿，root已实际运行本片results/verify-superseding-product-v1.py，superseding-product-root-acceptance-v1.json以**561身份、新CLI的24/24/1226checks及确切新CI**完成继承验收和3.2/3.3。原准确BigDecimal关系所在init/report/class_source/lock及原两个生产测试身份保持；当前build.rs重新随nested产品验收。不能冒称旧8cd CI成功。历史BigDecimal CLI SHA b79520629443a0211cd656d1374e415f76ac6adf400d79cb86154954caba9cb0只绑定历史blob，不代表当前产品。

## 正在验收：返回int数组+=新值，5/7

[recover-returned-int-array-compound-updates](openspec/changes/recover-returned-int-array-compound-updates/)规划4/4、strict已通过，实施5/7。完整ReturnedIntArrayUpdates共11成员，原两真实JDK2/2、fresh JADX default/none四腿4/4，当前Jarde两份完整source均compile失败0/2；root独立156checks/0errors。所有返回值、十条异常/求值顺序轨迹和原始失败保留在前片results/returned-next-baseline-v1；不得借原classes编译恢复source。

原形状为dup2→iaload→iadd→dup_x2→iastore→ireturn。返回新值与现有返回旧值PostfixUpdate不同；现有IndexAssign只有语句形式。设计仅加最小值语义数组赋值Expr，复用AssignOp及已有左值/RHS证明，准确四份dup_x2消费者、同块紧邻形状与一次原子claim，不新增全局copy/type/pass或合成局部变量层。

必须核对Java类型，category-1不是Java int证明；byte/short/char/int可合法提升，boolean或未知类型不得伪装为int。所有AST walker、来源/字段/命名/类型及emitter同步处理，赋值有副作用，lambda donor不能复制该写入。精准升级旧returned负例，merged row-Phi继续拒绝并保留BCI；补额外消费、错误复制、非int及停止控制。

Luna完整v1 patch已经冻结并应用，root修正栈槽顺序、错误Duplicate标签/私有模块引用和漏掉的report/facade遍历，并补新增集合复制/提交计费。新AST仅表达已证明returned int +=，不扩frame/decode。第一次真实focused最终6pass/2ignored；首三轮compile/结构失败全部保留。完整v4测试artifact在低磁盘时直接执行，双真实JDK各2ignored pass、合计8次旧nested与新returned完整类比较成功；明确没有新Cargo构建，不代表当前新CLI或最终产品验收。

2.2与3.1已完成：定向ordinary 8pass/2ignored，双JDK当前test各2ignored pass，额外sum/返回副本consumer及long返回六份控制都经真实JVM加载，非法dup2_x2两份输入被JVM拒绝且Jarde停止于frames表，无Java发布。新CLI v2 SHA71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110，全部10相关产品源/5测试源/canonical22身份冻结；新完整双JDK2/2、独立425checks，旧nested4/4/479checks、旧24/24/1231checks。所有源码完整空CP/SP重编且只新classes-Xverify/raw一致。原JADX/原程序按逐hash历史复用，不称fresh。

受影响Java-lib325pass、邻近四integration目标15pass/1ignored；官方fingerprint与P5各5pass/1ignored，reader实测(1087,4830,463,2715,8)并复验1pass，fmt/OpenSpec全strict/source diff通过。全仓seed1冷构建因机器剩余低于20GiB中止，没进入测试；清本仓1.4GiB后再次下降，MSRV在preflight未启动。空间回升后本机CI-exact Clippy及明确rustup run 1.88.0 MSRV workspace/all-targets已实际通过；之前失败/未跑记录保留。本机第二次全仓构建再触20GiB且未进入测试，已清本仓4.1GiB。最终当前产品确切四job CI、完整双seed/真JDK25等仍待验收，3.2/3.3未勾。详见[returned root验收](openspec/changes/recover-returned-int-array-compound-updates/verification-root.md)。

独立债务：历史fallback拒绝Arithmetic producer时未映射iadd@5，当前六个负例与冻结561拒绝正文及来源逐字一致；参数load由声明命名。负例明确保留copy/read/store/return/额外消费，成功compound仍要求全部真实来源。本片不扩大通用fallback机制。

## 已完成与队列边界

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数、612是JADX测试文件数，不是成功率。EM18/DT26仍部分已测；窄片或24控制腿不等于整单元完成。

- 子数组协变7/7、wildcard数组返回6/6，确切45b CI已验收。
- 构造参数primitive转换7/7，完整双JDK2/2；JADX四腿删除中间舍入，虽能编译但语义0/4，Jarde以原程序为oracle。详见[数值转换验收](openspec/changes/recover-constructor-primitive-conversion-arguments/verification-root.md)。
- constructed reference array组合8/8，owner/value/store及Site原子闭合已完成；完整历史失败保留，后续数组与BigDecimal补片已解除当时剩余限制。
- CF16 ImplicitCleanup主体已恢复：当前CLI双JDK/all与essential四腿16路径通过，原cleanup-over-return trace29，历史JADX重编仍trace299。不能重复实施主体；子Region停止/回滚边界须单独核对。
- DT26原生int[] capture已恢复；P02二维helper正文已由当前nested片补齐。平坦Signature generic arity和receiver-tail收尾漏计是独立债务，不混入本片。

## 工作树、磁盘与执行纪律

只有main/origin/main，无未合入分支或占用分支。14辅助worktree均detached、干净、main祖先、无target；受Codex保护的副本保留，没有遗留工作待合并。

只由root串行执行Cargo/Git/rustfmt/Java/JADX/Jarde CLI，一次一个Cargo。CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1，关闭debug信息但保留assertions。全workspace20GiB停止线；实测局部nested/returned test与CLI冷构建约309MiB，明确采用本仓1GiB target/机器2GiB余量双守卫（当前change run-root-gate-v3.py只接受这两类命令）。不碰其它项目target。第二次workspace冷构建增长到4.1GiB、再触守卫且未进入测试，已清本仓4.1GiB；不重复同条件冷构建。最新清本仓部分workspace构建残留，冻结CLI/raw均保留；机器空间快速变化，以实时df和实际守卫为准。未跑/停止记录不得计为通过。

全部生成source参与空CP/SP重编，runtime仅新classes且-Xverify:all；原exit/stdout/stderr逐字核对。不得借原class/helper、剥离失败成员或修剪raw日志。原始日志的whitespace提示与源码diff检查分开记录，不设全局豁免。以前详细阶段可从Git历史及各片verification-root/results检索。

## 下一明确方向：CF-16剩余边界

只读事实复审确认ImplicitCleanup主体不是实现缺口：历史root167checks、双JDK/all+essential四完整腿16路径已有。recover-proved-finally-cleanup task2.3仍写缺分支主体，是旧任务记录；剩余独立边界是structured-subregion budget/cancellation/rollback。guard层proof预算/预取消、p3_patterns完整输出及其它finally端到端原子性已有，不能替代ImplicitCleanup Builder子Region内部恢复状态验证。现有finally_checkpoint/restore_finally机制应优先复用；在returned产品确切CI验收后再同步OpenSpec/准备精准测试，不重复主体、不混CF17，也不凭未验证假设新增机制。
