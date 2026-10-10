# 非空 If arm 汇合来源：root 验收与接续

当前 **5/7**。已接受实际内部诊断、最小产品、永久边界回归和新 CLI 完整源码对照；精确自身 CI 与最终 clean main 交付尚未关闭。71 单元/612 测试文件和 CF07 整单元计数不变。

## 产品与证明

只改 build.rs 普通 If 的 OriginSet 路径。读取既有 If.join，检查 arm 最后直接 Straight 中实际末尾 goto/goto_w，以及完整 canonical outgoing 恰为一条 Normal 到本 If join。region::recover 的完成树检查已保证唯一 ownership，因而不新建归属表、IR、Frame、pass 或依赖。folded/refused conditional-value 和空 arm 规则不变。

counted 的同一次真实 IR 诊断已证明 canonical branch11/physical14、join27、then block17 的 iinc17+goto20，全边唯一 Normal17→27，owner计数1/0/1。新永久测试核全部23物理BCI、准确物理owner/name/descriptor、@20到含缩进和末尾换行的完整If范围、condition14和while30。错误target使用真实外循环exit33；goto20→header6会改变实际join，不能当拒绝反例。

真实 javac 异常反例的 If12/join27 then 是 Fallback(ExceptionEdge15)，SSA为invoke15/iinc18/goto21，全边 Normal27+Exception36。测试核这个实际树、helper拒绝和公开字节码引用，不伪造 Straight。其整方法引用只映射block leaders、physical21缺source的独立债务见 results/exception-case-architecture-root-v1.md。预算测试准确Stop at20且正文/map为空；取消同样不发布部分artifact。私有预期错误、真实失败和root修正均保存。

## 本地与完整源码验收

results/validation-acceptance-root-v1.json：12实际命令、562 passed/0 failed/1 ignored；Clippy使用CI同范围allowlist。52 pins（17产品/8测试/27实际include与显式class输入）前后恒同，target峰值701667300 bytes。所有build原argv与24 raw stream可重新核SHA。

新CLI `/private/tmp/jarde-proved-if-join-cli-v1`，0555，SHA `7b751758ee7f0b49bd61332a1a78412b36ecef898e43171ecf6af620d65cc9a6`。metadata SHA `6f785a03e50565bc5d90bd6a6ae85f1bb789e31647811421e5f21d41ee7030ca`；source base d9855c2764860c7c2e20e40e700abeaa59f00a55 和 uncommitted_if_arm_join_product=true 是真实历史冻结身份，之后不回写。新root runner SHA f139f117354f4b96762737190c03a025f3a02d3efc078062b715c63d83518f6b。

results/cf07-candidate-acceptance-root-v1.json：29命令、119 inventory members、原2/JADX4/Jarde4共10完整类腿；双JDK8/23与default-all恒同，源码与Runner原样重编运行同同JDK原raw。raw javap重解析准确physical methods/BCI与goto target。唯一新增来源为四profile counted@20 derived完整If，counted全部BCI已完整，原while来源/正文/旧来源不变。唯一剩余缺口是lastIndexOf([IIII)I@25，不能宣称整CF07全覆盖。

results/candidate-whole-classes-acceptance-root-v1.json及old-controls-exact-acceptance-root-v1.json：57命令、原4/JADX8/Jarde8共20完整类腿；28方法profile全物理BCI覆盖。两套控制的8份生成类正文与全部map都与已接受For bundle恒同，包括原for@20。v2 verifier的helper作用域NameError失败raw保留，root v3传入实际helper后接受。

## 剩余交付

root提交推送本片全部代码/class/源码/raw后，核提交head自身CI；已审capture-ci-product-root-v2.py和verify-ci-product-root-v2.py将证据保存在results/ci-product-v2。运行参数须使用真实产品40位SHA与run id；预期lib336/gateway12/workspace每seed3389（以真实日志验证，不预先宣称通过），仍要求4jobs/52steps、354结果记录、97ignored和所有新旧测试名。不得借For或文档CI。

完成自身CI后更新账本/handoff、提交推送记录，核main/origin、全部15 worktrees、仅main分支/辅助detached、没有待合入/占用，关闭3.2/3.3。当前cargo clean已实际释放3022files/669.2MiB；冻结CLI、源码/class与raw保留。资源守卫继续按用户批准的5GiB free/本仓target1GiB、一秒进程组中止执行。
