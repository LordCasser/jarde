# Loop4/5 完整三方基线

日期：2026-10-19。root已并行执行两份完整原件基线，并接受逐原流、完整源码/类库存和隔离运行复核。两目标均确认恢复缺口，尚未实施修复；相同诊断不代表相同根因。输入CLI来自产品 `618b5e20f27f94ba28ea2f9e4310648718475f82`，64,347,312 bytes/0555，SHA-256 `7e321a7f4fd6a029540ab169f79572c6ad83e74119900ac6d695290c9c055318`。

## 实际观察

| 每个目标 | Loop4 | Loop5 |
| --- | --- | --- |
| 完整上游源 |1068 bytes，SHA `42b0f55b653b41bc69286e1947c8a19d94b0da810cdcc7cf48bb6d2cd6a256e5`|1495 bytes，SHA `4aef48802b7dc7cc47c7c03d6c2639103c2913345cb8be8818545e63ed1fea64`|
| 原件有限输入 |`.`, `a`, `b`, `c`|`1.2`, `12am`, `13m`, `1G4`, 空串|
| 命令/原始流 |46 / 92|46 / 92|
| 原件隔离VM/CASE观察 |4 / 16|4 / 20|
| Jarde完整API单元 |8，debug/no-debug × class/JAR × default/all|8，同左|
| 目标test |fallback，遗漏live块 `[17,23,29]`|fallback，遗漏live块 `[14,23,106,39,88,46,53,60,67,74,81]`|
| ctor/check |均structured|均structured|
| Jarde完整源码编译 |16次尝试、0成功，无生成class/VM|16次尝试、0成功，无生成class/VM|
| JADX完整源码编译 |4次尝试：2次JDK23成功、2次JDK8框架边界失败|4次尝试：2次JDK23成功、2次JDK8框架边界失败|
| JADX隔离VM/CASE观察 |4 / 16，均同原件|4 / 20，均同原件|
| 错CodeSource控制 |原件1+JADX生成2，均拒绝|原件1+JADX生成2，均拒绝|

两个目标均保留完整outer JUnit、原始TestCls、全部check断言与三个方法（公开构造器、test、check），没有target字段/额外helper。完整原件由JDK23 `--release 8`生成debug/no-debug的major52 class；JDK8原源码编译尝试的框架55/52失败原文保留。每个JADX成功class tree含完整outer与target，不裁剪源码。

原件与成功生成JADX类在每个VM均实际调用check：JDK23两profile都通过，JDK8单独记录 `UnsupportedClassVersionError`（框架major55、运行时上限52）；四个/五个test输入仍正常运行。运行比较按同profile/runtime分别核对CASE及check，不把JDK8的框架异常算作断言成功。应用classpath仅driver，经过目标污染检查的pin依赖只加入parentless target loader URLs，目标CodeSource必须匹配精确root；运行目录为空且有超时。

Jarde的JDK23整类编译失败为primary test仅有解释/注释而缺少返回语句；JDK8首先报告JadxAssertions框架55/52边界。default/all的全文、方法身份、quality、正文与maps稳定。构造器/check映射到有效物理指令；primary test只有fallback quote的块头anchor（Loop4 8/23、Loop5 13/60），不能视作结构化恢复或完整来源映射。当前没有Jarde运行等价结论。

## 冻结证据

[完整归档](evidence.tar.gz)：352 payload、15,318,636未压缩bytes，归档2,182,759 bytes，SHA-256 `fda350b692e7999c5fbf69c9e0a2128d6c2288bbb69a95ae42bd965e0ba6619c`。[库存](inventory.json)逐项列出bytes/mode/SHA，[root回执](capture.json)记录全部成员重新打开核对结果。

`loop4/actual/`与`loop5/actual/`包含各自execution.json、全部92原流、原件Java/class/JAR、8API原报告/整源、完整JADX树、完整源码编译尝试、物理方法/BCI、VM与错根控制。`prep-v1/`、`prep-v2/`保留原准备及修订，`audit-review.json`为实际接受的Python审计，`independent-review/`是单独复开的原流/源码/物理事实报告。CLI二进制、外部框架jar和target未入库，身份pin保留。

初次Loop4审计的wrong-root argv固定索引未适配依赖URL尾部而失败；`loop4/audit-v3/`修正该审计接线，实际exit0并独立确认控制有效。旧失败原流与v2冻结材料保留，没有重跑或改写collector。此故障不属于恢复层缺口。

下一步是同轮Reader/CFG/SSA/Region实测与最小架构设计，再创建OpenSpec并派发Luna。固定13原件当前7项完整接受、2项已定位实际gap、4项待完整基线；CF09/71单元/612文件整体仍partial。
