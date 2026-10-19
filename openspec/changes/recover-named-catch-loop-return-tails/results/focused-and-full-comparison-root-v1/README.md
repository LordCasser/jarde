# Loop4/5 定向与完整三方对照

产品 `412245d514896b1d0eb9a8fe24388f6a70668637` 的两个真实缺口由Luna私有实现，root集成、架构审查及对抗性验收。基线和四原件IR已独立归档；这里保留从失败定位到最终完整生成类的实际历史，不能把单个方法或内部helper通过当成整类验收。

Loop4在自然循环外有17/23/29分支DAG。闭合证明绑定唯一入口、同循环出口和精确owner后，复用既有sequence/If/LoopBreak；保留ForHeader init3/update36及break26/33。Loop5保留effectful latch88的While，仅在精确命名行row0/[0,123)/handler124/type54下组合prefix+完整loop+tail106/return123。Builder复用既有catch身份，识别handler入口自身异常绑定，避免同物理槽的正常double被当作异常引用赋值。没有新增AST、CFG或通用槽拆分；外来入口/出口/异常行、双向Call/Return、重复owner以及预算/取消/晚期Stop均有拒绝与回滚验证。

最终CLI64,631,664 bytes、0555、SHA-256 `d29b6fee723d8f59fbecd9fd8be9f15a8ec73de3a90f5a2a29b24ff91e4af607`。最终定向CF12原预算门9/9、lib540、公API4通过；fixture阶段Reader census1、fingerprint5通过/1既有忽略。旧Clippy四测试lint失败及等价修正保留；root误用Rust2021排版导致fmt失败也保留，随后项目格式检查和CLI构建通过。首轮全工作区及首版a4产品CI在旧CF12精确预算门实际失败：单出口循环已能固定形状拒绝，却先被预付完整候选扫描预算。Luna最小补丁前置既有两出口条件，不改旧365@29/130@1测试、不减少真实候选收费。最终新CLI重新构建并完整重复三方/扩展对照，不能借用首版产物。最终源码的完整具名结果以fresh workspace和产品自身CI入口为准。

| 每原件完整矩阵 | Loop4 | Loop5 |
| --- | ---: | ---: |
| 实际命令/raw流 |70/140|70/140|
| API组合/structured方法 |8/24|8/24|
| 完整生成源码编译尝试 |20|20|
| JDK23成功：Jarde/JADX |8/2|8/2|
| JDK8框架55/52编译边界 |10|10|
| 原件VM/生成VM |4/20|4/20|
| 生成错误CodeSource拒绝控制 |10|10|
| test来源映射 |23/23|59/60|

各原件ctor/test/check与字段完整库存一致，debug/no-debug、class/完整JAR、default/all均覆盖；default/all文本、identity、quality及映射恒同，所有已引用来源均有效。完整源码不裁剪outer/check；所有生成VM的case与实际check结果按相同profile/runtime同原件对照。JDK23 check通过，JDK8 check因框架class55/52不兼容如实记录。独立重开280 raw流及全部引用pin。Loop5缺失来源103，实际为`goto14`，原字节a7ffa7；只登记为独立映射债务，不声称完整来源覆盖或与旧69/78同根因。

Loop5另外复用冻结12例driver与四原件VM oracle：10份成功完整编译产物×双runtime=20 VM，20stdout+20stderr=40 raw、240 CASE及20 CHECK；所有CASE字段、CHECK类型/消息/stdout与对应原件一致，保留stack而不要求CHECK行号相等。实际含null→-1、15M→15360、2g→2097152、1..2→-1、0M→0、超大十进制→Long.MAX_VALUE。driver-only应用classpath、parentless loader、精确CodeSource、空cwd和超时均有证据；没有去重或漏掉相同源码的不同API产物。独立审计早期文字曾错误翻倍计数，最终逐run审计与root独立原流解析已更正为20 VM/240 CASE，重复reopen不计新运行。

归档计数、SHA及成员索引在capture.json/inventory.json。CLI与target不入库；跨机器按归档成员前缀定位原始绝对路径。native/自身CI及Git blob来源绑定另见相邻验收入口；CF09/71/612仍partial。

实际归档共1377个payload、121,403,194未压缩bytes；[归档](evidence.tar.gz) 19,621,166 bytes，SHA-256 `4ebf0886dd49eb3c7d2430efbf23232a982ae008844990436e6f1ef51023ebe0`。[库存](inventory.json)记录每成员原路径/bytes/mode/SHA，[重开回执](capture.json)确认私有包和持久副本均逐成员复核。第一轮native失败与两版产品自身CI的原流在[native/自身CI验收](../native-and-own-ci-root-v1/README.md)另存。
