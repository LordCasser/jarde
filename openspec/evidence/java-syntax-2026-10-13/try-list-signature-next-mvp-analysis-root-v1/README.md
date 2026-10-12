# 下一大颗粒 MVP 的架构落点

root 已完整审读本地 JADX 7 个相关测试、SignatureProcessor/LoopRegionVisitor/ProcessTryCatchRegions/DebugInfo/CodeVar/调用泛型算法的只读分析，并逐文件核对35份源码和账本 pins；具体行号及静态限制见 reference-analysis-luna-v2.md。没有执行新的 JADX 或 Jarde 投影，不把本记录当运行验收。

下一方向是既有“已认证普通 Try 前缀+foreach”与“有限 List 泛型用途证明”的交集。现有 generic-return/VoidBody census 对任意 handler 早拒，导致正文已完整恢复而方法参数仍为 raw List。应复用同次 Region 精确 named row/所有权证书，对整个 Code/SSA（含 handler-owned 指令）做原用途 census，再将声明投影、调用 wrapper 和物理来源原子发布。不删除原 no-handler 路径的证明，也不新建通用类型 solver 或 CFG pass。

root 修正私有分析中的元素类型缩限：正式实现必须对现有 List proof 已支持的精确 E 统一处理，至少验证 String 和 Integer；不能因 String 更易通过就固定类型、方法名或fixture名称。拓扑仍以当前真实证书能力界定：ordinary单range catch、完整prefix+loop、已证纯出口；不能借泛型投影扩大到任意多循环/nested try/finally或未证明异常范围。

raw alias/heap pollution、参数重赋值、未知调用、wildcard写入和重载边界继续由现有闭合证明决定。JADX将raw alias写入折叠到List<String>导致整类编译失败的实际证据保留，源码与运行原类才是行为oracle。新spec需将支持的有限List调用和已支持控制体形状交叉，双JDK/debug-none/default-all整类比较并加入Stop/缺失owner等负例。

当前两change的最终工作区/产品自身CI/clean-main交付完成后再立项实施。此分析不改变71单元/612测试或CF11计数。
