# 局部引用写兼容验收

最新状态（2026-10-12）：`recover-proved-local-reference-write-compatibility` **6/8**。完整类双profile对照、永久正反例、精确late Stop和所有本地验收已接受；fresh全仓root-v5 **96批/363targets/3489通过/0失败/97忽略/3586具名结果/4784输入**，独立acceptor exit0。最终fmt/当前CI完整Clippy/global strict **356-0**/CLI字节一致/clean以及独立acceptor均exit0、target不存在。最新CLIroot-v5 61486096B/0555/SHA4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95。源码与证据尚待提交推送，最后已推送main为9a381cc1；本片自身CI和干净主线仍待，不得借前片CI接受。四次全仓及首次定向失败完整保留，成功重跑未拼接；新增扫描精确计费已核，finally紧预算77步通过。

/tmp清理20个无占用旧Cargo缓存，释放3.12GiB，最新101代和验收文件保留。CF11/固定71单元612文件完成数不扩大；下一项是同轮结构化void候选与List<String>参数用途诊断，当前仅私有准备，待本片交付后执行。以下段落为历史检查点。

以下为本片先前断点，历史保留：

当前3/8：生产v4已应用，仍未提交或验收全仓/自身CI。公开API fresh-v3三测试3/0/0、独立验收exit0；fresh core-v4四测试4/0/0并再次API3/0/0，精确Returnconsumer预算Stop与预取消通过。真实旧/新LVT范围都映射slot2/index0，旧String receiver读阻止Object扩大；两次错误测试假设的3/1/0失败均保留。见[core证据](results/core-focused-root-v1/README.md)。无新类型pass/identity/LUB/cast，CF11/71/612完成数不变。

新CLI实际构建0555/61486096B/SHA4df9ff1c96addfcae2814b72687e35f2883b5aba1d19b0b42cdfa9959d091f95，[构建](results/candidate-build-root-v1/README.md)。完整源码debug97/none101命令，每profile两JDK×default/all四条isolated生成目标整类+原控制/Runner编译0、运行0、29行同原。通用观察checker均exit0，专门验收待执行；七个全生成类combined仍compile1/no runtime，不能称完整项目通过。见[对照](results/complete-source-candidate-root-v1/README.md)。采集时4784源码恒同；后续只测试、reader人口及golden/fingerprint改动，最终CLI须按最终真实闭包重新构建绑定。

reader真实测得(1133,4995,473,3048,10)，仅替换旧准确人口预期；handler黄金只三个conflict文案有变化。意向fingerprint重录实测2096→2134、38新增文件、仅该golden变动，分类/旧输入未变化；reader1/0/0、fingerprint5/0/1、API4/0/0、reference lifetimes8/0/0并clean。API新增中止测试虽通过无部分文本/map检查，但实际at2、未抵达新proof末尾；root不接受为late Stop，正在改为准确单方法请求总预算。任务2.2保持未勾。见[指纹及公开中止记录](results/corpus-and-public-stop-root-v1/README.md)。

以下为提交的旧产品基线断点，保留历史，不代表当前生产状态：

当前2/8，实际旧产品同IR观察、架构设计及完整控制基线已接受。foreach前片8/8/自身CI及最终main审计已真实交付。生产未改，候选应用和实际新产品对照均尚待；见proposal/design/specs/tasks。固定CF11/71/612完成数不变。

独立Luna静态对抗审查发现slot_uses忽略unassigned reuse访问及elided_cleanup；root核1934/1954/1967及reuse::Plan::variable_at None语义后将exact SSA access census补入design，保持原完整性要求而非新增feature。候选尚未应用，任务2.1未勾；NoType旧“no Java declaration can hold both”绝对文案必须纠正。

实际[完整基线](results/complete-source-baseline-root-v1/README.md)：debug93/none97命令、380原始stdout/stderr，两个独立checker实际exit0。七目标类与29case Runner原JDK8/23一致；JADX两profile双JDK各四条完整生成运行腿同原。Jarde两profile各四次combined compile1，无完整生成类运行；debug只有参数控制四条isolated运行腿通过，none另有CharSequence四条通过。四条source map缺口/profile单列，失败不计行为通过。准确旧CLI526e2e5a/最终build receipt b3ea224绑定，4744源码前后恒同、target absent、free12281155584B，资源守卫未触发。

真实slot控制使用slot2：astore2/10/22与aload3/23；debug LVT first=String范围3..8、target=Object范围11..25，none所有目标Code均无LVT/LVTT/LineNumberTable。此处只接受物理slot/属性，不据此认定reuse identity；候选永久测试仍须从实际复用计划核身份。私有v1被root审查要求修订handler、Return额外writes、逐扫描计费及冗余实体/index，未应用/编译；v2私有修订中。
