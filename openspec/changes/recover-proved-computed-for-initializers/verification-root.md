# Root 当前验收

当前2/10，仅1.1/1.2诊断与完整基线已接受；computed-for 的 region/build 生产源码仍与已接受自身CI的bafb产品一致；CF12 的独立 facade gate 已提交为74b，自身CI尚待，2.x未实施。任务顺序仍为CF12同类常量名gate串行集成后再应用computed-init for。

同一次真实IR/SSA/CFG/Builder测试实际1/0，通过5GiB free/target1GiB一秒守卫，132条输入只临时两测试源变化，之后完全恢复和cargo clean。原Store@3的准确Assign(arg3-1)与空声明已生成；既有rposition消费可复用，所有return leaf/frame/phi/use/update及调用上下文在同raw核验。私有类型错误与runpy namespace绑定修正原样保存，最终root独立exit0。见 [same-IR诊断](results/same-ir-builder-diagnostic-root-v1/README.md)。

双JDK原/pending class合法且九条行为相同，另无handler调用初值四条原行为已实跑。fresh三方完整基线59命令独立核验：JADX普通javac输入四组前四行错误，pending和direct八组正确；Jarde复杂控制八整类编译失败/no runtime，direct四组完整重编验证执行同原、仍while。故新增for不能跨g提升初始化；效果边界的准确拒绝也是验收路径。冻结LoopCases又fresh6JADX命令，两JDK完整类与当前原/Jarde八行相同。见 [完整对照](results/complete-controls-root-v1/README.md)。

不新增表达式walker、purity表或IR/pass的方案仍成立：复用原初始化语句及唯一SSA Store消费者，提前一次现有返回叶集合，扩展准确ForHeader回边来源证书。后续2.x必须实际永久边界/Stop/全类对照/产品自身CI验收，不凭这些基线关闭实现任务，不称整个CF07/CF13/71单元完成。

新增无 handler 的副作用控制已通过 root 独立验收：真实36命令、双JDK、8份完整原样Jarde源码重编、12运行侧60行同原；pending class 在独立g之前保存f，初始化仍while。没有复用induction参数槽或handler导致的早期拒绝；全部19条物理BCI、UTF8来源和default/all一致。新增class全量hash只在运行后观察，未冒称事前/事后双pin。见 [无handler控制基线](results/no-handler-call-order-baseline-root-v1/README.md)。该结果支持复用已有deferred binding，但尚未应用的for候选必须重新接受同一控制，2.x任务不关闭。
