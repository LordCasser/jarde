## 1. 取证与基线
- [x] 1.1 复跑 patrol fixture（SL/SM/SN），确认矩阵与 README 一致；2026-10-18 root对双真实javac8/23完整SN/SM/SL/LD执行137命令/274raw，16原oracle、SM八重编/16运行通过，SN/SL/LD仍拒绝；JADX LD八次真实错码9≠10、SL四次编译失败保留，见results/baseline-root-v1。这是红基线验收，不关闭实现任务。
- [x] 1.2 回答 Q-i/Q-ii；root实际临时构建/八all请求取得同次canonical/SSA与Region，40方法配置核Code区间与顺序，26raw pin、整个非usage JSON同605基线，恢复源码/target absent；真实原因和同一声明门的不同上游见instrumentation.md、results/first-refusal-root-v1。SL forward-join内部未日志的具体拒绝条件不冒称已知。

## 2. 实现
- [x] 2.1 MVP：复用现有返回叶证明恢复case内return；root final-focused-v5通过668项，final-api-v2的24目标配置全部700物理BCI覆盖、身份/UTF8/map闭合，SN57/60来源缺口经原switch-arm/LoopContinue修正。真实外来入口、证书缺失/错target/歧义target、异常边、clone edge身份及晚AnalysisSteps/分析后取消/期限入口通过；公开请求late stop无部分发布。clone仅边证书，不冒称整历史loop恢复，见独立债务。
- [x] 2.2 第二步：沿现有switch_loop_join与Frame恢复break outer及SL mixed；root接受单candidate遍历与严格全arm closure，join68/continue71分离；465库、九return/label/stop、12既有switch边界和一公开request停止通过。双编译器label三方法与SL mixed整类原样重编/双JVM driver通过，无新公共AST/solver；异常类两leg均由真实Reader行验证拒绝，所有权和停止边界见final-focused-v5。
- [x] 2.3 对照零回退：最终10a73 CLI实际16请求，六目标的24配置fallback→structured，其余56稳定方法投影同605基线；SM五形全部正文/质量/fallback/map不变，default/all全文/map同，完整80方法的声明顺序、flags、owner及2152区间/3024来源引用复核，见final-api-root-v2。

## 3. 验证
- [x] 3.1 最终10a73 CLI三方完整源码矩阵已root接受：225命令/450raw，Jarde32次整类原样重编、64 driver运行1744case及64 main全等原oracle；javac8 source/target8与javac23 --release8，双JVM -Xverify:all与CodeSource/双root隔离。含SN四锚、LD {9,1,5}→10、SL {1,2,4,3}→124及各边界；JADX SL四编译失败、SN/LD各八ledgerBreak driver失败、LD八main9≠10均原样保留，后续未执行case不计。见final-matrix-root-v2；原始driver另436case，不混算为rebuilt。
- [x] 3.2 最终生产源码fmt/CI同款Clippy/MSRV、必要focused/reader/P5、两fresh seed workspace与strict通过，5GiB/target1GiB每秒守卫/default stack/finally exact clean；精确产品cf857abaabc4fcff82689812226f9906b8d932b2已推送；自身CI39072531393由root实际148raw/69来源pin核验，4jobs/52步骤成功、两seed各382targets/3716通过/0失败/98忽略、27关键门各一次通过；本地两fresh seed同计数/逐target/5016来源及九新门，证据见workspace-acceptance-root-v1与own-product-ci-root-v1。交接与文档收尾后的最终clean-main外置收据见handoff，不增计71/612。
