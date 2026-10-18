# CF08：双入口前缀循环的实际恢复差距

独立构造 Java8 classfile `cf08.MultiEntryCounter.test(IZI)I`，刻意保留 prefix@4、header@7 两个 SCC 外部入口。它来自对 JADX FixMultiEntryLoops 两种块复制算法的检查；不是原始 TestMultiEntryLoop.smali 的等价转换，不代表 DEX 前端或整个CF08覆盖。

| 对照腿 | 实际结果 |
| --- | --- |
| 原始 class | JDK8/23 verifier+运行：2 VM / 24观察 |
| Jarde | child/bundle-child × default/all共4API；目标均fallback，原因 jre_region_irreducible，块[4,7,11] |
| Jarde完整源码 | 两编译器共8次均因missing return失败；0 VM |
| JADX完整源码 | 两编译器均成功；4 VM / 48观察同原始 |

实际26命令/52raw；12有限用例覆盖steps0..2、skip双值、普通int及溢出初值。Root独立使用32bit模运算公式核对stdout，完整复开3416路径/10301pin记录；并核实际19B Code、分支偏移、StackMapTable、原始及生成类身份/字段/方法/flags和parentless CodeSource。一个错根控制拒绝。Jarde失败文本完整保留，没有为重编成功修改源码。

JADX实际输出是：先 if(!skip) 执行prefix，再 while(steps!=0) 在减步后执行prefix，最后返回值。Jarde当前的whole-method不可约拒绝挡在Walker前；仅放宽natural-loop分类无法表达这两个入口。Root裁决：需要窄的区域所有权证书，真实物理prefix拥有一次、受证明的纯int前缀可在初始条件和循环尾各展示一次。复用现有If/While/Assign/Return AST、局部类型/SSA和SourceMap；不改CanonicalBlockId.path的jsr语义，不增加通用CFG clone pass、state machine或solver。精确边界、Stop/预算和反例需先写OpenSpec，再派Luna实现；此证据提交不含生产实现。

[capture.tar.gz](capture.tar.gz)、[库存](inventory.json)、[root复开](capture-review.json)保存完整输入writer/driver、raw/API/源码/class、四版私有准备、root独审和架构memo。Luna架构memo早于终态JADX结果，最终裁决以root architecture-verdict与review为准。CLI仍为已验收96e534b0；receipt SHA `da22ca92c26800ff92f7a3dfaf54007c3c315e01d9a335e58b52392dc0255f9a`。71/612与CF08整项计数不变。

后继计划：[recover-proved-entry-split-int-loops](../../../changes/recover-proved-entry-split-int-loops/tasks.md) 已strict通过、当前1/6；实现/真实IR探针/对抗性tests由Luna私有并行准备，尚未集成或验收。包内root裁决与Luna静态memo保留其写作时点。
