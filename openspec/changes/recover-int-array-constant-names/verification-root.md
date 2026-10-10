# 整数数组常量名称：root 实际验证

产品检查点375dee45a54d22ce960429cf11f0f96d33ca28b8已提交推送，其自身CI38026598963全部成功，05:47 UTC root独立verifier v2实际接受，任务8/8。192b0bc13eca631f44ebaf28a998db52d4f14040是本片CLI构建基线；以下按时间保留历史过程，不借实例或静态CI。

已应用private v4；真实编译分别暴露测试AST新增字段/Range引用、facade Budget enum引用错误，Luna最小修正后重编。历史scoped-rust-root-v1/v2/v3和patch原样保留。

scoped-rust-root-v4的Java AST1/1、body replay1/1通过；facade staged-target独立对抗测试通过，但旧PriorAssert数字拒绝断言失败。root用v5原始诊断确认实际方法保留if(!$assertionsDisabled)/if(!ok)/throw AssertionError及直接return数组；没有先前member_text占用，唯一直接叶合法投影VALUE（bci23）。旧contains("assert")仅命中合成字段名，不能证明assert fold。未修改生产规则；测试按实际正例核完整控制流、token Field/MethodPoint及physical recovery仍数字7。

scoped-rust-root-v6实际Facade8/8通过。独立staged测试先重建同一物理AST/body，未占用control实际投影VALUE，另一相同输入占用真实member index时class/member text与derived集合保持不变，避免被其它拒绝条件掩盖。旧switch/return、字段不完整、预算early-stop与pre-cancel回归通过；不宣称late-stage动态注入。

validation-build-root-v1 实际通过 scoped all-feature Clippy、实例 Java5/facade6、静态6/接口4、reader178、fingerprint5（1 ignored）和 CLI build。CLI /private/tmp/jarde-int-array-names-cli-v1 SHA256146c657cdf5baeaa2b9c31e2715547ff9f1a4e129decad025f517a93400af3bf，metadata candidate-cli-v1.json SHAc97aa02f3af4bab9064e63f9431ecaf61c4c1769e4693529cf5a0d1cdfc72180；root 独立 build-freeze-root-acceptance-v1 核全30文件pins与实际执行记录。04:30 UTC仅清本仓target，释放859.0MiB，冻结CLI及所有源/class/raw保留。

完整类collector v4实际在JADX helper context参数处失败；v5完成75命令后在JSON tuple-key序列化失败。所有已观察raw与历史版本保留，不能把部分收集冒称接受。v6最小修正后root实际执行退出0：75命令、32渲染、10条新完整编译腿、18运行，另导入已独立接受的2个ArrayFill原oracle。Jarde与JADX完整源码均原样重编，运行开启/关闭断言两模式与对应原程序逐字一致；CONST_INT、重复VALUE及PriorAssert VALUE正例成功，歧义/遮蔽/不支持形状无新名称投影。417闭合文件，manifest SHA91917201c2660db52271fe687f3fa37ecc92e31e0fe9875f48021dd3a8a344e8，inventory SHAdea6e75e4fd40e7773fab34cdb7757da2a5a4e94031224713ac6f1c06d99beb8。

当前tasks4/8（1.1/1.2/2.1/2.2）。完整类已执行成功，独立物理来源与精确range验收、产品提交及其自己的CI尚待完成，不借实例或静态片CI。

对抗审查发现Try的resource/catch名称扫描未逐项poll和计AnalysisSteps/IrItems；已有节点总计不包含空catch名称，不能借节点预算声称有界。root已在本片returns_int_array路径补齐克隆前计费，保留旧switch行为，加入16空catch的精确停止用例（IR cap8或analysis cap7、written0、BCI0及使用量）与resource停止用例。04:57 UTC实际启动guarded validation-build-root-v2，增加本片AST2/replay1/facade8，全部原回归保留。旧CLI v1/full-class v6与CI脚本v1只能证明修正前版本；最终冻结、完整类和产品CI必须重新执行。预构建fmt/diff检查及OpenSpec strict 334/0真实通过，记录prebuild-checkpoint-root-v2。

04:59 UTC validation-build-root-v2实际10命令全部通过：AST2/replay1/facade8、实例Java5/facade6、静态6/接口4、reader178、fingerprint5（1 ignored）、Clippy与CLI build；峰值900673477 bytes。CLI v2 SHA51e17991cbf7cb6cebb43d2ea3b66dfe655522e1f8c3da9a69b05ba47f698067，metadata SHAdd2a5a0d5731ca5fa5a12e87c9e344c6faaf269c82bc128f935b57e8681ec282，build SHA b14f28e2e1200665c4a156f0b0f3438795ce7ee455ebcbf8e4b4e247d5543af8。root独立核30文件pins及20rawstreams后冻结接受，结果build-freeze-root-acceptance-v2。

05:00 UTC完整类collector v7实际退出0，重新完成75命令/32渲染/10完整编译腿/18运行，417文件闭合inventory SHA4eb840839ca5190977a3f2946773e4689c2d2a6c68409f7e5a0b5b86b80d6ad8，failures为空。独立verifier v4实际因导入旧oracle仅有stdout/stderr、没有exit字段而失败；原产品编译/运行没有失败，脚本与raw保留。root另逐方法核全部80记录的OriginSet primary+derived并集恰等于原javap指令BCI，完整method/owner对应；不能只看primary把derived aload_0误判成来源缺失。正式完整独立接受仍待修正版本执行。

05:00 UTC仅cargo clean本仓target，删除3216 files/858.9MiB；target不存在，所有冻结CLI、源/class/raw保留，记录root-clean-v3。

05:08 UTC root审阅完整v4→v5 delta后实际运行独立verifier v5，退出0：接受417闭合文件、75命令/150raw streams、32renders、80物理方法完整OriginSet并集、8次正向名称观察与10次负向观察、10条新完整编译腿与18次原raw一致运行。导入的2个oracle整source_case及runtime与已接受原baseline逐字段相等，复制stdout/stderr逐字核原command raw，从原runtime取exit而不默认成功。接受结果candidate-full-class-luna-v5-verification.json，真实执行记录independent-execution-root-v2。Tasks更新6/8；仅产品提交、推送及其自己的CI验收待完成。

05:47 UTC确切375dee自身CI38026598963由root独立verify-int-array-ci-product-luna-v2实际接受：4jobs/52steps，两seed各354records/3374 passed/0 failed/97 ignored，11项required新测试逐项成功；真Temurin25.0.4+7 instruction/P3/full-class比较、MSRV、Clippy、fuzz与supply均通过。接受文件results/ci-product-v1/acceptance-int-array-v2.json核30产品Git blobs及冻结CLI/meta/10命令构建raw，不借其它片CI。完整API/stable/supply gzip(mtime=0)与实际执行记录均已保存。watch TLS超时和初次日志下载EOF保留于ci-capture-network-failure-v1，属于本地网络读取失败，不是CI测试失败。此后检查点只提交文档、规划和证据；不把文档提交新CI冒称已经成功，不递归等待相同产品代码的文档CI。
