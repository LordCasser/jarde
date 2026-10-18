# CF08 原始 TestLoopRestore2：完整基线与来源缺口

root实际执行官方Raung 0.1.1装配四命令，冻结JADX原始 `.raung`，非手写JVM替代：class 148B、版本45.3、唯一 `test([I[I)V` Code 24B/十指令，SHA `4fd1503499a049efa8e736a98de4339b6504973f80208987ed0774073df9c447`。工具依赖来自官方Maven的POM，来源/下载hash/许可及两独立物理parser保留。

最终产品3fa/CLI f7ea…实际基线16命令：class/JAR×default/all四API均structured；四份完整Jarde源码分别经JDK8/JDK23共八次编译成功。JADX完整源码两次编译均因 `while (1 == 0)` 不可达语句失败，保留原文/错误。上游本就disableCompilation，仅检查该文本片段；本片没有执行目标方法，没有运行等价或精确JADX文本通过结论。Jarde输出 `if(false){do{}while(!true);}`，未修补或裁剪任何生成源码。

**真实待修：四map均只覆盖9/10物理BCI，缺BCI8的goto11。** 全部已有anchor有效且method/class身份匹配；不能把materialization Complete或quality Structured当成全部BCI映射完整。BCI8已归属region、Transfer自身不emit语句；父If只继承空臂/尾部join跳转，循环gateway只涵盖其它受证路径，故进入nested loop的前导transfer漏来源。优先复用父If的derived-origin，不改变区域owner、Java文本或loop gateway的latch证书含义；尚无本片生产修改，不关闭CF08/71/612。

`capture.tar.gz` 包含144 payload及内嵌inventory，SHA `3eea2de4a59149f9bd99d5c762bc04b690cd22ff2845d909a0617a813ba7f3a7`，root逐成员复开bytes/mode；十一官方工具jar/sources.jar为显式外部pin，未重复入仓。包含完整装配/输入/API/raw/源码/编译class、collector/reviewer静态修正、独立review与架构定位；Raung MIT及JADX LICENSE/NOTICE随源片段保留。无CLI、JDK或Rust构建产物。
