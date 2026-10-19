# TestBreakWithLabel 原始对照

原始标签跳出已经恢复，问题在局部布尔标志：`found` 被呈现为 int，`append(Z)` 因缺类型证明而拒绝，整条打印语句进入 fallback。原始 debug/no-debug 的 80B Code 相同，分别有、无 `found:Z` 调试记录，因此不能靠调试表补结论。

实际采集 94 命令，完整 outer/child/check 源码均保留。原件 4 VM、24 用例观测一致；Jarde 八 API、16 次完整编译全部成功，但 32 VM 的 192 用例观测中有 128 次 stdout 不同，返回值和异常类别相符。JADX 两次 JDK23 完整编译后 4 VM、24 观测均与原件相同。原件及 JADX 的 JDK8 框架 major55/52 编译失败原样保留，没有删外层源码换取成功；未调用 check/JUnit。

六个有限用例包括 `{{7}, null}` 命中后退出，能区分退出外层与只退出内层。Jarde 的 test 为 fallback、构造和 check 为 structured；映射已有锚点有效但 test 只覆盖 34/42 指令，缺 44/56/59/60/63/65/69/72。类型修复与来源补齐分别处理，不把编译成功当作语义或来源完备。

`capture.tar.gz` SHA `958fe71efa1a384000d638f2cd89a77dcc64834b007471a840b645e319a6f2da`，全部文件/目录库存见 `capture.tar.gz.inventory.json`，root 已逐字和 mode 复开，结果见 `root-review.json`。归档保留原 collector 的 pending 状态，由其独立 review 和 root 接受记录绑定；审查器的真实 schema 失败、修正和只读复审均保留，没有为修审查器重跑目标。

归档还含固定 CF09 五原始类范围审计及 SequentialLoops 数学用例计划；后者没有执行目标，不代表 CF09 完成。JADX LICENSE/NOTICE 随包保存，外部 JDK、CLI 和工具依赖仅保留原始精确 pin。

下一 [recover-proved-literal-boolean-locals](../../../changes/recover-proved-literal-boolean-locals/design.md) 先核真实 SSA，再复用声明规划的全写/全读证明。当前尚未实施或验收该修复。
