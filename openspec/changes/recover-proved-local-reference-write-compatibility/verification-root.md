# 局部引用写兼容验收

当前2/8，实际旧产品同IR观察、架构设计及完整控制基线已接受。foreach前片8/8/自身CI及最终main审计已真实交付。生产未改，候选应用和实际新产品对照均尚待；见proposal/design/specs/tasks。固定CF11/71/612完成数不变。

独立Luna静态对抗审查发现slot_uses忽略unassigned reuse访问及elided_cleanup；root核1934/1954/1967及reuse::Plan::variable_at None语义后将exact SSA access census补入design，保持原完整性要求而非新增feature。候选尚未应用，任务2.1未勾；NoType旧“no Java declaration can hold both”绝对文案必须纠正。

实际[完整基线](results/complete-source-baseline-root-v1/README.md)：debug93/none97命令、380原始stdout/stderr，两个独立checker实际exit0。七目标类与29case Runner原JDK8/23一致；JADX两profile双JDK各四条完整生成运行腿同原。Jarde两profile各四次combined compile1，无完整生成类运行；debug只有参数控制四条isolated运行腿通过，none另有CharSequence四条通过。四条source map缺口/profile单列，失败不计行为通过。准确旧CLI526e2e5a/最终build receipt b3ea224绑定，4744源码前后恒同、target absent、free12281155584B，资源守卫未触发。

真实slot控制使用slot2：astore2/10/22与aload3/23；debug LVT first=String范围3..8、target=Object范围11..25，none所有目标Code均无LVT/LVTT/LineNumberTable。此处只接受物理slot/属性，不据此认定reuse identity；候选永久测试仍须从实际复用计划核身份。私有v1被root审查要求修订handler、Return额外writes、逐扫描计费及冗余实体/index，未应用/编译；v2私有修订中。
