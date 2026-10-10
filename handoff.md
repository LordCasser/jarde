# HANDOFF — jarde 主线接续入口

用户已明确回复「继续在当前聊天推进」。持续目标 active；root 负责架构、OpenSpec、真实 Java/JADX 对照和对抗验收，确定性脚本/实现由 Luna 完成。以 JADX 71 单元、612 测试文件账本为依据，先逐片追平，再探索额外场景。

## 当前主线和已验收产品

已验收产品 **6ddc77d5cc2a6fcf098aad40998f6d2602731c59** 已推送 main。[确切 CI 38001720578](https://github.com/LordCasser/jarde/actions/runs/38001720578) 四 job/52 steps 全部成功；两个 fresh workspace seed 各 354 test-result records、3353 passed/0 failed/97 ignored。真 Temurin 25.0.4+7.0.LTS 的 returned ignored 1pass、nested ignored 1pass、BigDecimal ignored 2pass，MSRV/Clippy/fuzz/API/tree/OpenSpec 及 cargo-deny 0.20.2 root/fuzz 四政策全部成功。

root 已保存完整 API JSON、stable/supply 原始 gzip，实际运行独立 verifier v2 核对不可变 Git blob 的 10 产品源、5 测试源、22 canonical 文件和冻结 CLI。入口为 [returned root 验收](openspec/changes/recover-returned-int-array-compound-updates/verification-root.md) 及 results/ci-product-root-acceptance-v2.json。v1 因 gh 日志字面颜色格式匹配失败，原失败保留；v2 仅处理颜色表示，不改验收条件或 raw。

冻结当前 CLI：`/private/tmp/jarde-returned-array-cli-v2`，SHA **71f0a864243e7c4155789e05a0c5021ec06e87ac8a8bf9dfb38b38acb4906110**。最新提交可能是随后验收/测试检查点，以 `git log -1` 为准；产品源仍与 6dd 相同。新增 CF16 公共测试属于后续检查点，不借 6dd CI。

```sh
git status --short
git log -1 --oneline
git rev-parse HEAD origin/main
git branch -av
git worktree list
```

## 最新实现闭环

- [returned int 数组 +=](openspec/changes/recover-returned-int-array-compound-updates/verification-root.md) **7/7**：最小 ArrayAssign 值表达式，仅准确认领 int 返回的 dup2/iaload/iadd/dup_x2/iastore/ireturn 形状，沿既有类型/消费/顺序/依赖/所有权证明；没有新增全局 copy/type/pass。新完整类双 JDK 2/2、root 425 checks；旧 nested 4/4/479 checks、旧 24 腿 24/24/1231 checks，全部完整源码空 CP/SP 重编、仅新 classes -Xverify:all 执行，raw 与原一致。原程序/JADX 历史 hash 复用与 fresh candidate 执行分开声明。
- [nested int 数组更新](openspec/changes/recover-nested-int-array-compound-updates/verification-root.md) **7/7**：DT26 P02 helper 的 `t[0][0] += i`，复用原始 `[[I`/aaload 行类型和四份 copy 证明，去掉冗余 store-copy 类型 gate。
- [BigDecimal→Number](openspec/changes/recover-bigdecimal-number-widening/verification-root.md) **7/7**：准确 release-8 关系，旧 24 腿由 22/24 到 24/24；AtomicInteger 继续拒绝。旧失败 CI/raw 保留，561 组合 CI 已独立继承验收。
- 子数组协变 **7/7**、reifiable wildcard 数组返回 **6/6**、构造 primitive 转换 **7/7** 的历史确切 CI 和完整类验收均已完成。不要以窄片或回归腿成功冒称整个 EM18/DT26 单元完成。

当前 reader 实测 pin **(1087,4830,463,2715,8)**，2061 fingerprint 输入；官方 fingerprint/P5 各 5pass/1ignored。当前 Java-lib 325pass，邻近四 integration 15pass/1ignored，returned ordinary 8pass/2ignored。细节与历史失败均在各片 verification-root/results。

## CF16：主体已实现，剩余验收 10/11

[recover-proved-finally-cleanup](openspec/changes/recover-proved-finally-cleanup/verification-root-2026-10-10.md) 的 ImplicitCleanup 分支/throw/saved return 主体并非缺失；旧 task2.3 已按真实状态纠正。当前 CLI 双 JDK × all/essential 四份完整类、16 路径 fresh 执行通过，root **167 checks/0 errors**，cleanup 覆盖返回的 trace 为 **29**；历史 JADX source 本次重编仍为 **299**，不能当 oracle，未声称 fresh 提取。

新增 `p3_patterns.rs::structured_finally_stop_never_publishes_partial_class_source` 覆盖 zero IR/zero output/预取消具体停止码与无半正文发布，并要求提交后证据受限 **Produced/Partial 且完整正文不变**。root 格式化后完整 p3_patterns **84/84**，构建峰值 187129829 字节。公共总预算不能定位 child 构造中途；静态复审既有 checkpoint/restore，不添加生产 hook，不宣称动态内部 rollback。新测试需由自己的 CI 验收。

深度探针 N=2/N=33 原程序双 JDK 共四份 fresh 编译/验证/执行通过，但 Jarde 四份 run 都 explanation_only/fallback，诊断为 finally copy、loop shape、uncovered blocks；execution=complete，未命中 recursion_bound。**这份早期探针不能算深度限制通过。** 证据在 results/depth-boundary-root-v1。N=2 是 protected-body 嵌套循环 + 单 saved int return + cleanup 的明确浅层组合待分析，不是 cleanup 内部循环。

N=2 fresh 补验已完成：JADX/default/none 四完整腿4/4，Jarde原样完整重编0/2，root232checks/0errors。入口调度先接loop、generic finally起点随后已过晚；静态最小建议仍须实测，见 results/shallow-loop-finally-analysis-root.md。该自写组合先列候选，优先完成现有 JADX CF16 剩余验收；后续 field/branch 探针已到达既有 Region 真实深度限界，接着完成整类回归。已有 void-loop Test2 与 multi-return Test5 是不同完成形状，不能借它们算该单保护段组合已覆盖；不继续盲目加深输入，不先新增机制，也不混进当前公共停止测试。

最新 field/branch N=33 双 JDK 在 BCI450 确实停止：jre_recursion_bound/NotProduced/空正文与来源/Partial，root **204 checks/0 errors**。双原 class 复制为永久 cf16-region-depth fixture，现有 harness 新增 exact stop 测试。v6 在 libtest 默认小栈溢出失败；v7 明确8 MiB测试线程栈后最终完整 p3_patterns **85/85**。不改生产机制，不声称小栈安全或内部 Builder 动态 rollback；v5其实只有旧84项，不借其命名算新深度测试。任务2.4/3.1已勾选，3.2待新检查点CI。

最终完整类对照已独立验收：Normal双JDK当前Jarde2/2完整重编/执行/raw匹配；Completion覆盖型仍2/2完整类编译拒绝，历史JADX重编错误566/8保留。26命令/12腿/94闭合文件，四份实际javap -p核对全部物理成员与Normal全BCI，root **536 checks/0 errors**。历史3/6 Code遗漏private mark，实际完整4/7，历史不篡改。原源码4/4、JDX固定历史source4/4重编（Completion语义错）、JardeNormal2/2成功/Completion2/2拒绝分开计；不是总体12/12语义通过。fmt/all OpenSpec strict/diff check通过。新测试检查点CI尚待验收，不借产品6dd CI。

## EM18：最新完整基线无需实现

[array-field-store](openspec/evidence/java-syntax-2026-10-10/array-field-store/README.md) 以 JADX TestArrayInit.test2 为依据补 fresh byte[] 方法字段写：原双 JDK **2/2**、fresh JADX default/none **4/4**、当前 Jarde **2/2**，root **294 checks/0 errors**。完整七方法/两字段与 allocation/element/call/putfield/return 真实 BCI 保留；新数组身份、失败时旧数组对象身份、null RHS 顺序/异常优先级四条 raw 路径正确。现有 ArrayInitializers+FieldWrite 已闭合，不建重复实施 spec。

EM18 下一片已锁定 JADX `TestArrayInitField.test()`（root核过本地测试原文），见 [基线计划](openspec/evidence/java-syntax-2026-10-10/array-field-store/next-static-constructor-plan-luna-v1.md)：先验 static/constructor 初始化顺序与完整类语义，若仅呈现差异不称机制缺失。后续有明确队列：static/constructor 数组字段呈现、TestArrays2 四种 primitive array 分支、signed byte/long 与 ConstantValue。先 fresh 完整类基线，确认缺口再立 spec；上项不证明 static promotion 已支持数组。[71 单元账本](openspec/evidence/jadx-feature-inventory-2026-09-27/summary.md) 的分母和整单元分类不变。

## 工作树、磁盘与执行纪律

只有 main/origin/main，无未合入或占用分支。14 辅助 worktree 均 detached、干净、main 祖先、无 target；Codex 保护副本保留，无待合入工作。

root 串行执行 Git/Cargo/rustfmt/JDK/JADX/CLI，一次一个 Cargo；Luna 只读/patch/script，不共享运行编译。全 workspace **20 GiB** 机器余量停止线；nested/returned test+CLI 实测约309 MiB，p3_patterns 实测约178 MiB，只有明确窄目标采用 **2 GiB 机器余量/1 GiB 本仓 target** 双守卫。全仓冷构建两次在20 GiB前停止且未进测试，原记录保留，不重复同条件；完整双 seed 已由确切 6dd CI 补验。

只清本仓 target；不动其它项目。冻结 CLI、canonical fixture、原始 source/class/raw 全部保留。机器空间会随其他进程快速变化，以实时 df/runner 为准。以前提交清理467.2 MiB/722文件的真实记录在 returned/results/root-clean-submission-v1；当前检查点又清本仓178.5 MiB/348文件，记录在 CF16/results/root-clean-checkpoint-v1；最终85项通过后再次清理178.5 MiB/348文件，见 root-clean-checkpoint-v2。

全部 generated source 空 CP/SP 重编，runtime 仅新 classes 且 -Xverify:all；不借原 class/helper、不删除失败成员、不手修生成文本。原 exit/stdout/stderr 逐字比较，完整日志不裁剪。71单元完成与窄形状完成分开计数。

独立债务：既有 fallback 对拒绝 Arithmetic producer 未映射 iadd@5、平坦 Signature generic arity、receiver-tail 收尾计费；有记录但不混入当前窄片。
