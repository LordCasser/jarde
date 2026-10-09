# HANDOFF — jarde 主线接续入口

用户明确要求继续在当前聊天推进，不暂停持续目标。root负责架构、OpenSpec、真实Java/JADX对照和对抗验收；确定性实现由Luna完成。按JADX的71单元账本逐片推进，先追平既有能力，再探索额外场景。

## 当前状态

已验收产品为 **561de209531c021a9d8adb7c979bae5a57d6c3fb**，已推送main。[确切CI37994276707](https://github.com/LordCasser/jarde/actions/runs/37994276707)四job/51steps全部成功；两轮fresh workspace seed各354test-result记录、3348passed/0failed/96ignored，新增nested/BigDecimal真JDK25完整类执行、MSRV、Clippy、fuzz及固定cargo-deny0.20.2 root/fuzz四政策全部通过。原始JSON、完整日志及冻结source/CLI身份已由root独立核对，入口见[nested root验收](openspec/changes/recover-nested-int-array-compound-updates/verification-root.md)。

当前产品之后的验收/规划提交不改变产品源；新的returned片尚未验收，不借本次CI。

```sh
git status --short
git log -1 --oneline
git rev-parse HEAD origin/main
git branch -av
git worktree list
```

当前WIP是已完成验收记录、下一片OpenSpec和patch-only准备；主树产品/测试/workflow仍与561和冻结CLI相同。先提交这一验收检查点，再应用完整下一片补丁。

## 已完成：nested int数组复合更新，7/7

[recover-nested-int-array-compound-updates](openspec/changes/recover-nested-int-array-compound-updates/verification-root.md)恢复DT-26 P02 lambda helper的`t[0][0] += i`。原始`[[I`沿既有`array_of_value`经aaload可证行为int[]；删除对dup2 store-copy的冗余类型gate，保留原始行类型、四份准确copy identity、唯一消费、顺序/依赖、预算/Stop。不新增pass、AST或类型传播服务。

冻结CLI为`/private/tmp/jarde-nested-int-array-cli-v1`，SHA **8745071312981d4eafb8fbb738e6a1fa1edbb55753062bfae057e879c494bb7a**。新完整类四腿真实JDK8/23空CP/SP重编、只运行新classes且-Xverify:all，4/4与原始raw流一致，root独立474checks/0errors；旧24控制腿fresh24/24，root独立1226checks/0errors。original/JADX对旧24腿采用逐hash历史基线，不声称本次fresh。

最终Rust边界/public预算/预取消及邻近回归14pass/2ignored，官方fingerprint5pass/1ignored，P5严格pins5pass/1ignored；325 Java-lib、新reader census(1071,4654,463,2715,8)、本机root/fuzz cargo-deny四政策也通过。fingerprint只新增11输入，原2029条不变。详细source/命令/raw/hash在results/local-root-acceptance-v1.json及verification-root。以前17GiB未执行记录保留为历史；上述是空间回升后实际补验。

3.2/3.3已完成：results/ci-product-root-acceptance-v1.json接受确切四job/全部steps、准确双seed、新nested ignored1pass与BigDecimal ignored2pass、真Temurin25.0.4+7.0.LTS与冻结source身份。supply独立CLI0.20.2安装与root/fuzz四政策成功；日志中的数字429只是时间戳，修正见ci-supply-http-status-correction-v2.json。历史未跑/失败记录保留。

## 已完成：BigDecimal→Number，7/7

[recover-bigdecimal-number-widening](openspec/changes/recover-bigdecimal-number-widening/verification-root.md)只增加release-8 NUMBER_FAMILY中的准确BigDecimal→Number行，不改concat机制。完整24腿从22/24增至24/24，生产/双JDK/full-class与本地严格门禁已过；两个旧负例的过时预期已精准更新，AtomicInteger仍拒绝且来源保持。

旧6997产品CI37989319644确实失败于过时断言，原日志保留。修正提交8cd8c4f的CI37991578328 stable/MSRV/fuzz成功，supply在checkout前DockerHub HTTP429失败；仅重试该失败任务后仍429，不继续循环重试。ci-run-v2/v3、完整stable及重试supply日志在本片results。不能把旧run称为success。

561组合产品四job全绿，root已实际运行本片results/verify-superseding-product-v1.py，superseding-product-root-acceptance-v1.json以**561身份、新CLI的24/24/1226checks及确切新CI**完成继承验收和3.2/3.3。原准确BigDecimal关系所在init/report/class_source/lock及原两个生产测试身份保持；当前build.rs重新随nested产品验收。不能冒称旧8cd CI成功。历史BigDecimal CLI SHA b79520629443a0211cd656d1374e415f76ac6adf400d79cb86154954caba9cb0只绑定历史blob，不代表当前产品。

## 下一片：返回int数组+=新值，2/7

[recover-returned-int-array-compound-updates](openspec/changes/recover-returned-int-array-compound-updates/)规划4/4、strict已通过。完整ReturnedIntArrayUpdates共11成员，原两真实JDK2/2、fresh JADX default/none四腿4/4，当前Jarde两份完整source均compile失败0/2；root独立156checks/0errors。所有返回值、十条异常/求值顺序轨迹和原始失败保留在前片results/returned-next-baseline-v1；不得借原classes编译恢复source。

原形状为dup2→iaload→iadd→dup_x2→iastore→ireturn。返回新值与现有返回旧值PostfixUpdate不同；现有IndexAssign只有语句形式。设计仅加最小值语义数组赋值Expr，复用AssignOp及已有左值/RHS证明，准确四份dup_x2消费者、同块紧邻形状与一次原子claim，不新增全局copy/type/pass或合成局部变量层。

必须核对Java类型，category-1不是Java int证明；byte/short/char/int可合法提升，boolean或未知类型不得伪装为int。所有AST walker、来源/字段/命名/类型及emitter同步处理，赋值有副作用，lambda donor不能复制该写入。精准升级旧returned负例，merged row-Phi继续拒绝并保留BCI；补额外消费、错误复制、非int及停止控制。

Luna只在新片results产出完整未编译patch和notes，主树冻结。root已验收前片CI，接下来负责apply、fmt、实际focused及完整对照；新CLI身份必须扩大到所有实际受影响产品源，不能沿用旧五源集合。canonical fixture字节从已核验baseline复制；其加入后的reader/指纹计费按实际测量更新。

## 已完成与队列边界

[71单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md)的71是验收单元数、612是JADX测试文件数，不是成功率。EM18/DT26仍部分已测；窄片或24控制腿不等于整单元完成。

- 子数组协变7/7、wildcard数组返回6/6，确切45b CI已验收。
- 构造参数primitive转换7/7，完整双JDK2/2；JADX四腿删除中间舍入，虽能编译但语义0/4，Jarde以原程序为oracle。详见[数值转换验收](openspec/changes/recover-constructor-primitive-conversion-arguments/verification-root.md)。
- constructed reference array组合8/8，owner/value/store及Site原子闭合已完成；完整历史失败保留，后续数组与BigDecimal补片已解除当时剩余限制。
- CF16 ImplicitCleanup主体已恢复：当前CLI双JDK/all与essential四腿16路径通过，原cleanup-over-return trace29，历史JADX重编仍trace299。不能重复实施主体；子Region停止/回滚边界须单独核对。
- DT26原生int[] capture已恢复；P02二维helper正文已由当前nested片补齐。平坦Signature generic arity和receiver-tail收尾漏计是独立债务，不混入本片。

## 工作树、磁盘与执行纪律

只有main/origin/main，无未合入分支或占用分支。14辅助worktree均detached、干净、main祖先、无target；受Codex保护的副本保留，没有遗留工作待合并。

只由root串行执行Cargo/Git/rustfmt/Java/JADX/Jarde CLI，一次一个Cargo。使用CARGO_BUILD_JOBS=1、CARGO_INCREMENTAL=0、RUST_TEST_THREADS=1，关闭debug信息但保留assertions；**20GiB为构建停止线**，不碰其它项目target。2026-10-10机器空间又降到约14GiB，root只清本仓421MiB缓存，恢复约434MB，仍低于停止线；命令及实测在nested results/root-clean-current-low-space-v1。随后机器空闲恢复25GiB；当前本仓target不存在，冻结CLI和raw证据均保留。实时空间以df为准，按20GiB守卫执行下一片Cargo，不能把未跑命令称作通过。

全部生成source参与空CP/SP重编，runtime仅新classes且-Xverify:all；原exit/stdout/stderr逐字核对。不得借原class/helper、剥离失败成员或修剪raw日志。原始日志的whitespace提示与源码diff检查分开记录，不设全局豁免。以前详细阶段可从Git历史及各片verification-root/results检索。
