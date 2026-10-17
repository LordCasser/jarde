# exception-only finally 实际验收索引

当前产品 **3fa506df0ccd7da3d38ab769bfd65235817508fe** 已推送，自身CI39051492999已由root独立接受，最终两版本strict各366/0，计划11/11；最终交付见final-delivery-root-v1。持续目标active，71/612不增加。

- initial-facts-root-v1：两907B封存原class的完整CFG/SSA/effects及真实generic首拒，临时源码恢复和清理收据。
- frozen-inputs-root-v1 / baseline-matrix-root-v1：独立双编译器682B输入及九组合driver、完整原七方法、旧CLI/JADX真实红基线。
- neighbor-jvm-verification-root-v1：原七个完整邻变体双JVM严格加载14/14；六个负例加一个绑定row边界正例，不执行负例目标。
- entry-parameter-phi-facts-root-v2 / local-read-fixture-root-v1 / local-parameter-reassigned-input-root-v1：真实367B入口参数及565B重赋值拒绝控制；首拒是冗余Local0 phi的旧基数要求。窄证书只放行已消去、unused且各输入同一Entry槽参数的phi，不增求解器。
- builder-test-assembly-failures-root-v1：缺真实field plan、缺生产prepare及旧phi首拒的失败均保留；未降低断言凑绿。
- final-focused-root-v3：最终458库/181reader/2公共正例/2公共停止/指纹再生1/指纹检查5，共649 passing appearances、0失败、1忽略；fmt和CI同款Clippy通过，默认栈/资源守卫/exact clean。早期focused-root-v3仅647及342d CLI，不作为最终CLI。
- final-matrix-root-v2：最终7287CLI，75命令/150raw、44方法配置；两输入compiler×default/all×两重编compiler×双JVM，8整类编译、16重建运行144case等4原始运行36case；完整七方法仅tryThrows改善，两显式finally missing-return红项保留，8次sealed编译失败且无sealed runtime。JADX四编译成功、八运行首case trace8而非78。
- local-runtime-root-v1：入口参数独立类default/all×两重编compiler×双JVM，8运行24case等4原oracle12case；null传播NPE，非null保持异常对象身份，trace78一次。全部3方法/6配置的身份/index/BCI/map闭合。JADX两编译成功、四运行首case trace8失败；不同compiler输出字节不同，同compiler default/all相同。
- neighbor-public-root-v2：最终CLI对七完整拒绝类的28命令/56raw、184方法配置，全类正文/map/quality/fallback同旧4811；包括重赋值phi拒绝。
- existing-finally-public-root-v2：旧15class、60命令/120raw、236方法配置全类正文/map/quality/fallback同旧4811；仅源码/来源验收，不冒称重跑旧产品576runtime。
- local-gates-root-v1 / msrv-root-v1：相邻P5/CF16/finally本地门和两版strict，最终MSRV1.88全workspace/all-targets check通过。旧阶段30门531/0/8不替代最终workspace。
- product-source-binding-root-v1：最终CLI SHA7287e856c5ce256c127fc6b33bcc3c663b058e913bf50de5d82ed9c9373b95c8，3002构建来源及4955native来源逐文件绑定精确产品；标准指纹仅增加19项，原2256项不变。
- workspace-seed1-root-v1 / workspace-seed2-root-v1：两个fresh seed各378targets/99批/3700通过/0失败/98忽略，两轮来源、inventory及逐target计数一致；9新关键测试每seed各一次ok。408raw复核，默认栈、5GiB/1GiB每秒守卫及exact clean。补充接受器首次缺re.MULTILINE导致的parser失败已保留并修正，生产源码未改。

所有capture按manifest逐成员复开，实际scope/归档SHA见各root-review。own-product-ci-root-v1实际4jobs/52API步骤通过，两seed各378targets/3700通过/0失败/98忽略，逐target结果相同，9关键测试各一次ok；20命令raw流及全部工作流步骤核验。FinallyCompletion.finallyReturns/finallyThrows仍是独立欠账；不删原方法，也不把本leaf推广成所有finally形态已恢复。

- final-document-strict-root-v1：最终规划/交接/CF16增补后，实际1.11.0和1.14.0各366/0，4955产品来源未变。
