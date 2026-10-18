## Context

动机及真实三方基线见proposal。原始Code十指令已被区域树accounted；BCI8是if臂前导、单指令goto到内层循环header11。Transfer无独立语句，普通If只继承空臂及末尾join跳转，故此入口没有映射。BCI17退出跳转已有映射。parse/45.3 dialect有效、ClassIdentity/方法匹配、源码恢复Structured、完整类八次重编成功分别是不同证据；没有目标运行或字节码执行验证结论。

## Goals / Non-Goals

**Goals:** 只补该已恢复结构的真实派生origin；受证变体按实际块/边/指令匹配，不依赖类名/BCI8/固定常量。原Java及物理owner完全不变，缺证继续原路径。

**Non-Goals:** 不增加Region/CFG节点、pending-origin状态机、通用goto归属器、solver或外部库；不改常量折叠/loop风格、SourceMap协议或Evidence Complete的含义；不改任意其它来源债务。

## Decisions

1. **选择父If来源补齐。** 复用现有OriginSet::plus_derived与if来源收集。只检查非空臂最前面的单块、单条直接goto/goto_w及紧邻Loop实际header；完整canonical outgoing集合必须恰一normal边到该header，物理块/指令身份闭合且无上下文clone。已有区域owner仍由最终唯一所有权门负责。不可仅凭终点BCI或Operation::Transfer猜归属。可提取并复用既有末尾join跳转的exact-edge证明，小helper只在消除重复时添加，不建立新抽象。
2. **不向Loop.gateway_origins追加外部入口。** 其一处消费者把向量解释为受证latch列表；扩大含义会污染其它闭合证明。也不改Transfer发射为假Java语句、不做遍历全方法并把所有goto挂在最外层。
3. **预算与事务复用。** 仅对前导单块/紧邻Loop形状进一步检查；昂贵边表扫描前按原维度收费并poll，origin节点按既有IrItems收费，Stop走原Builder事务，不refund或降为成功。缺证不追加origin，不影响已恢复语法。禁止对每个普通if做无谓全图扫描。
4. **原始fixture与最小对抗测试。** 永久保存148B真实class、原raung及JADX许可/manifest；原件库存增一class/一Code/四直接branch targets；另两个明确标记constructed的goto_w及effectprefix控制各增一class/一Code/四target，完整新增树预计三class/三Code/十二targets，无handler/jsr增量，须root实际核物理清单与真实IR；指纹由root受守卫显式重建。原class/JAR×default/all完整映射要求10/10且文本与基线逐字相同；窄真实IR测试覆盖goto_w/可变位置及不满足single-transfer/normal-target/clone约束的边界，不能用伪造摘要代替真实IR。加入实际预算/取消停止门，不镜像实现写无意义测试。
5. **完整验收保留范围。** 最终CLI四API重新采集map；Java全文不变时精确绑定并复用已接受八次完整编译，不重复JDK/目标运行。JADX两次原始unreachable失败原样保留。适用If/loop来源、EntrySplit、finThrow/nestedFin回归由最终fresh元数据派生workspace运行一次，root核具名门及自身CI全部步骤/双seed。机器余量5GiB、target1GiB、defaultstack、每秒guard及finally精确clean；Cargo期间不共享写/Gitmutation/网络/manualJDK或CLI。

当前双入口产品3fa自身CI及源码pin采集结束前，Luna只读共享树并在私有tmp实现；root独占共享写、Git/网络/工具链。工具依赖Raung只用于冻结基线，不加入Jarde依赖；官方0.1.1 MIT及JADX LICENSE/NOTICE已随源证据保留。既有标准库/Runtime足够。

## Risks / Trade-offs

- [臂内存在真实副作用或目标歧义] → 只接受单指令前导块与唯一normal边，不收集其它未映射指令。
- [来源修正改变循环闭合/owner] → 只修改Builder父If origin，canonical/Region结构逐字不变，现有结束门继续运行。
- [映射materialization Complete被误读为全BCI保证] → 只对本已证形态断言完整来源，不改全局Evidence语义；正文停止与后置optional映射停止分开验收。

## Migration Plan

无数据迁移或兼容适配。root审私有patch后集成最小差异；编译/对照/停止或归属门失败则修复候选并保留真实失败，产品提交仅在实际验收后推main。其它债务独立登记。
