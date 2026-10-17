# switch 非局部出口实际证据

## 2026-10-18 当前产品红基线

`baseline-root-v1` 使用已验收产品81b0684d5的605 CLI，新编译原封不动的SN/SM/SL/LD：四完整源×javac8/23，八class均major52；双JVM共16原始oracle。137命令/274原始流逐份核字节与SHA，工具/输入/源码前后不变，target不存在。完整源码从Jarde JSON或JADX输出原样重编，未删除或修补任何方法。

- Jarde共16配置；SM八次完整类重编、16次隔离运行匹配。SN/SL/LD共24次完整重编失败，无rebuilt运行。执行complete/CLI0表示请求完成，不是源码质量通过；所有六个目标方法均有明确fallback/explanation-only标记。
- SN两个case-return的报告原因是uncovered blocks；SN.labBreak、SL.labeledBreak与LD.lab是switch shape/loop shape/uncovered；SL.mixed是switch arms overlap/loop shape/uncovered。同一local1诊断不能证明相同首拒。
- default/all的八对完整类正文、方法正文/quality/fallback/source map相同；all额外展开evidence/region/rule，非usage整报告不要求相等。40个default方法配置为28 structured/12 fallback；all对应相同，共80方法配置。独立检查了668个default map区间、1120个同类origin BCI引用；这里只证明引用合法及范围合法，不宣称拒绝方法覆盖全部语义指令。
- JADX1.5.6的SN/SM共八次完整重编、16次运行匹配；SL四次编译均因不可达break失败，无runtime；LD四次编译成功，八次运行均输出9，原始双编译器/双JVM均为10。普通SN main未暴露此错，LD判别输入不可省略。

root接受的是当前红基线，不是实现完成。采集器、独立接受器、两份复核结果及所有实际输出均在归档。

## 同次IR首拒已确认

`first-refusal-root-v1`实际应用Luna私有诊断补丁，守卫构建39.34s/峰值372076204B、exact clean；八all请求的整个非usage JSON同605基线。40方法配置按完整stdout方法顺序和canonical Code区间配对；26raw pin复核，源码SHA精确恢复，完整产品来源同基线，target不存在。SN返回44未入loop scope、label break无局部join准入、SL重复尾71与后置声明拒绝已独立复核；未打印的SL forward-join内部条件不推断，见instrumentation.md及归档独立memo。

design已按这些事实补齐并派发两个Luna私有实现：返回叶片及switch join/出口消费。当前2/7，仅取证关闭，产品实现和对照未验收，不按规划或私有补丁计完成。已有LoopBreak/Continue/SwitchBreak和Return足以表示，暂不新增公共表示或solver。
