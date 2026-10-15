# 实施与实际源码效果

当前5/6完成；修复自身commit CI尚待，旧c871 CI不作为本片验收。归档、同步和最终main源码绑定均须在自身CI实际成功并独审后完成。

## 生产边界

只改src/facade.rs中的两条direct-parent消费者、共享关系读取与两个owner读取、既有proof closure传播。复用DependencyProofFailure原Error/Report及既有恢复前工厂，不改reference-overload、全局Budget/merge/publication或正常访问predicate，无新依赖。其余源码修改均为永久回归和既有cfg(test)观测的扩展。

## 实际测试

| 范围 | 实际结果 | 边界 |
| --- | --- | --- |
| 旧生产public | 2通过/2行为失败 | 外部C公共Partial原已绿，不作早停证明 |
| 旧生产CLI | 1通过/2行为失败 | 两cap实际exit0，预期4；后继断言未执行 |
| 旧生产observer | 5原有H1通过/7新增失败 | 5例先在错误Cancelled状态失败，2取消例在后继Checkpoint tail失败；后继断言不算执行 |
| 旧生产普通缺失/歧义 | 2通过 | 原普通拒绝基线 |
| 修复首验 | 12+6+3，全21通过 | 七新增in-flight均达原channel、诊断、账目及空tail断言 |
| 最终focused | 18 summaries，457通过/0失败/1默认忽略 | 指定忽略运行控制另已作为1个通过计入457；全部16新增具名门通过 |
| 既有身份/flags/来源专项 | 4 summaries，21通过/0失败/0忽略 | recover_write_accessor_field_types、p3_accessor_edges、raw_receiver_field_selection、p3_field_presentations |
| 格式/Clippy/OpenSpec | 全通过 | 当前CI相同Clippy参数；OpenSpec1.11和1.14各364/0 |

首次停止的两渠道、真实收费取消及过期ledger checkpoint由既有最窄observer实际观测。DT-29两个消费者各有三类控制；真实外部C receiver B的source关系读取Report控制确实尝试B，停止后不读A或进入field。未声称每种外部source/owner渠道组合均有动态覆盖。停点之后全部15个counted维度无新增，保留已获准用量，不以预取消/sleep/伪造Error替代。sticky ledger允许仅保留已付费构造器；plain Budget低cap则要求当前Stopped setter record已合法准入。

## 完整源码对照

最终CLI SHA256 c2db4721a879c47493b537e946b0ff827986e1e58e7b021222df1e66a8c9c3cd，59676240B，0555，绑定4905源。实际12命令保留24份raw：前三次取得root/A/B报告，仅编译root中完整嵌套A/B与原Runner，standalone A/B不作为额外编译单元；javac23 --release8的rebuilt classpath没有原JAR。whole-root生成文本与旧成功基线逐字相同，未改生成Java、Runner或oracle。

Java23和Java8分别运行原JAR及重编结果，共四次-Xverify:all输出均逐字为true:true:false:false\n，stderr空。原Runner只调用set(true,false)，不声称穷尽输入。此为完整生成源码的编译及固定输入行为证据，不把verifier成功当一般语义等价，也不增加既有全局覆盖计数。

同CLI另三次JSON/all预算4096/1/2真实exit0/4/4。宽预算保留三条父owner cast和物理access$002调用；低预算保留analysis Complete、原ClassHeaders维度/诊断、constructor文本和source map，setter为Stopped/not_produced且无正文及map。低预算报告未拿去编译。

所有执行源码/输入before与after相同；最终focused和专项均exact cargo clean成功、target缺失。外置CLI/JDK原生二进制以收据pin绑定；可移交证据包保留原始绝对路径及历史状态，不将旧red、预备脚本或包生成成功改写为产品全绿。
