## 1. 冻结输入与架构

- [x] 1.1 冻结未改写 TestFinally3 debug/noDebug、依赖及两工具，复核七方法 Code/异常表、原程序固定 driver、Jarde/JADX 原样整类编译基线；归档 raw、hash 和失败 no-runtime。证据：results/tf3-original-inputs-root-v1、tf3-original-baseline-root-v2、tf3-baseline-matrix-root-v1；100方法配置/8原运行64cases，Jarde20/JADX6真实编译失败。
- [x] 1.2 执行实际 SSA/array/site 探针覆盖裸数组、动态长度、初始化、子数组、multianewarray、构造元素与多参数；root 核物理 argument/final value、uses、sources、pending sites 和首拒，形成架构审查收据。证据：results/tf3-ssa-probe-root-v1、direct-array-inputs-root-v3、direct-array-ssa-probe-root-v1及architecture-review-root-v1.json；临时源码已恢复、target不存在。

## 2. 最小组合实现

- [x] 2.1 Luna 按 design 提交最小私有补丁，root 集成精确直接消费者判据并复用既有 array/Builder，替换被统一证明覆盖的 String 窄特判，闭合后置实参/sibling数组 reader 和同owner精确receiver调用选择；通过针对性正例、旧间接 factory 回归及完整 java 库测试。最终final-gates-root-v5实际461库/0失败，公共正例/停止及P5旧计量通过；早期focused/final-gates整体失败收据另存，不冒称完整门通过。
- [x] 2.2 增加精确错误身份、共享/alias、无关效果、异常覆盖变化、所有权及预算/取消/期限控制；以合法反序swap变体验证求值顺序，验证原子拒绝/Stop、不发布半正文、来源完整，不降低旧语义断言凑绿。最终4公共测试、461库测试通过；reverse-final-root-v1及order-probe-root-v1真实合法反序控制拒绝；通用fallback来源债务已单列并由旧/新CLI对照确认。

## 3. 完整对照与交付

- [x] 3.1 最终 CLI 对 TestFinally3 两 profile/default-all/独立 child及bundle-child/owner 原样整类重编；允许的 javac8/23、Java8/23 隔离 driver 全同原 oracle，核所有物理方法/BCI/UTF8来源及非目标正文；JADX 实际失败保留。tf3-original-final-root-v1核100配置全BCI，tf3-final-matrix-root-v1实际20重编/40隔离运行320case，88非目标配置不变，JADX6真实编译失败/no-runtime。矩阵CLI为e01；计量修正后的605 CLI由output-replay-root-v1及独立审查核完整源码和非usage报告恒同，仅ir_items/elapsed_millis数值变化，按相同源码复用实际JDK结果，不冒称重新编译运行。
- [x] 3.2 独立通用数组家族双 javac/JVM 完整源对照，观察长度/元素异常、多个参数和内外构造顺序、身份；拒绝变体保留完整类和真实原因，结果由 root 独立验收。complete-array-final-root-v1的88方法配置全BCI/8重编16运行224case由root与Luna独立复核；same-owner-final-root-v1为8重编16case且class Complete；reverse-final-root-v1为8自然重编16运行、8拒绝编译no-runtime，JADX反序8运行全失败trace21。
- [x] 3.3 验证 reader/指纹、fmt/当前CI Clippy/MSRV、双 fresh workspace seed及产品自身 CI；归档全部失败/最终 raw、来源绑定，strict OpenSpec 通过，提交推送并核 clean 主线/target absent，更新 handoff 且不增加71/612完成数。产品81b0684d5自身CI39062097681实际4jobs/52steps，两seed各381targets/3707通过/0失败/98忽略、18关键门及逐target计数恒同；本地两fresh seed同计数、8关键门、412raw、默认栈/守卫/exact clean接受，见workspace-acceptance-root-v3与own-product-ci-root-v1。构建3037/每轮4990来源绑定产品，最终文档/clean-main收据单列，不将文档head冒称产品CI。
