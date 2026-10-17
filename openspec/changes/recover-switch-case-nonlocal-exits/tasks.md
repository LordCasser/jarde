## 1. 取证与基线
- [x] 1.1 复跑 patrol fixture（SL/SM/SN），确认矩阵与 README 一致；2026-10-18 root对双真实javac8/23完整SN/SM/SL/LD执行137命令/274raw，16原oracle、SM八重编/16运行通过，SN/SL/LD仍拒绝；JADX LD八次真实错码9≠10、SL四次编译失败保留，见results/baseline-root-v1。这是红基线验收，不关闭实现任务。
- [x] 1.2 回答 Q-i/Q-ii；root实际临时构建/八all请求取得同次canonical/SSA与Region，40方法配置核Code区间与顺序，26raw pin、整个非usage JSON同605基线，恢复源码/target absent；真实原因和同一声明门的不同上游见instrumentation.md、results/first-refusal-root-v1。SL forward-join内部未日志的具体拒绝条件不冒称已知。

## 2. 实现
- [ ] 2.1 MVP：复用现有返回叶证明恢复case内return；真实SN双编译器公库/default-all/CLI须Structured、全部BCI/身份/map闭合，错误入口/出口/row/clone等邻变体拒绝，晚budget/cancel/deadline无部分发布，root运行测试后关闭。
- [ ] 2.2 第二步：沿现有switch_loop_join与Frame证明case内break outer及SL mixed；证明无局部join时所有路径闭合、局部join和continue不同身份、块恰好一次，真实正例/错边/外来入口/跨case/异常边与停止测试通过后关闭，不新增公共AST/solver。
- [ ] 2.3 对照零回退：最终CLI对八个原始class的default/all完整方法正文与当前基线比较；SM五对照形及非目标方法逐字节不变，来源/质量/fallback均按实际范围核对。

## 3. 验证
- [ ] 3.1 三方完整源码原样双javac重编、双JVM rebuilt-only运行：SN四锚、LD判别输入{9,1,5}→10、SL独立driver {1,2,4,3}→124及空数组/重复迭代/各出口，核CodeSource隔离、效果次数与结果；JADX实际错码/编译失败原样保留，不删方法或修源码凑绿。
- [ ] 3.2 最终生产源码fmt/CI同款Clippy/MSRV、必要focused/reader/P5、两fresh seed workspace与strict通过，5GiB/target1GiB每秒守卫/default stack/finally exact clean；精确产品提交推送并独立接受其自身CI、更新handoff和clean main后关闭，不增计71/612。
