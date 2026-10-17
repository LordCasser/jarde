# Final complete-source replay

最终产品30afa7c9ab2818f41959f9ea6a8ca6f896714dc1与implementation-v1的29命令native/4945 source/Git blob与mode一致。CLI60514880B/0555/SHA256 05e0272eb8cbc65acb87418cecda6d15571ddaea84c041d30436c1238e60b89d。所有生成Java逐字来自本次完整class-source输出，保留全部方法；没有修Java、删方法或把原class放入rebuilt classpath。

| 实际执行 | 命令与结果 | 完整源码和运行 |
|---|---|---|
| final-full-source-v1 | 10命令/20raw均0 | 两独立LoopTransfer类各3方法，完整2150B源码；真javac8与javac23 release8重编、未改942B driver仅对rebuilt编译；双JVM共4次168B oracle逐字等 |
| war-full-source-v1 | 12命令/24raw均0 | 两个973B/三STORE成员WAR；真实enumerate_artifact_tree取得container origin，再以WEB-INF/classes声明显式root/layoutwar；完整Java逐字等对应JAR，再重编/4次隔离JVM逐字等 |
| final-loop-transfer parent screen | 24命令/48raw均0 | FY5/FX6/FO5方法均保留，两腿共6完整类均Java/Structured/空fallback；6次真javac重编和12次固定main输出逐字等 |
| pinned JADX comparison | 21命令/42raw：19个0、2个预期FX javac1 | JADX1.5.6原样FY/FO两腿4源码重编与8main匹配；FX两源码throw th未定义(line56)重编失败，无FX rebuilt运行。包名defpackage按实际调用，不改源码 |

JAR/WAR两transfer方法各35物理BCI均存在于正确PhysicalMethodId的source map，包括For13派生的handler45/47/50/52与正常转移42/56。两条转移目标仍独立：continue到53，break到59。原控制baseline的2412B生成类本来就能重编；本次效果是两方法去除handler45 fallback/注释、完整来源闭合，并首次以这组完整恢复结果执行独立11观察driver。不能把基线仅重编成功说成原源码运行失败。

父screen与CallReturn最终screen逐方法比对：两腿28个非transfer方法text逐字不变；4个finContinue/finBreak腿记录从mixed/fallback变为Java/Structured并有已证handler45记录。固定main对照与独立11观察driver分开，原保护范围没有可观察同步throw，不把main或JADX失败当作handler动态证明。handler无throwing成员与异常对象/清理值链由私有静态证书覆盖；不宣称恢复任意finally。

两个最终实验独审：JAR/WAR 5275checks/0errors，报告SHA273b905ce99659845c790205ce0018633736fd0ee996c07ad21f2207d8a71987；parent/JADX报告SHA4b42fedea2e25b234e954fef3b6e5f65ffb620f6cc21d4bb3575e2b4d2e2383b。中间10命令及其147checks/0errors报告也保留，不与最终重复运行相加为新测试FQN。

包内301 payload+MANIFEST共302安全唯一regular成员，原始runtime/class/JAR/WAR/Java、原控制完整目录、driver/oracle、runner/support pins与上述独审均保留。外部manifest逐字等包内MANIFEST，archive字节/hash在独立summary；工具CLI/JDK/JADX二进制不入包。绝对路径只是原始provenance，按manifest路径重映射；native及Git原始绑定从implementation-v1包对应payload/路径读取。该包是实验取证，不是完整离线工具环境。自身双seed CI和父未完成instrumentation/规划验收另列。
