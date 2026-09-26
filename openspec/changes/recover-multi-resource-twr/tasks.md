## 1. 冻结形状与三方基线

- [x] 1.1 自写两资源 Java 8 fixture（正例：读取返回/内层抛出/外层 close 抛出/正常关闭四条路径；负例：等宽 patch 主行结束或伴随行的范围/目标），javac --release 8 与 9+ 各编译一份，冻结 SHA；记录 javap 行几何、原 class 执行输出（含 suppressed 顺序）、jadx 1.5.6 输出（展开为嵌套 try，记为其偏离）、jarde 修前输出（RangeEnd 引用），并对照旧 `Guarded.two/three` 的单行布局。[证据与 Root 重放](../../evidence/java-syntax-2026-09-26/multi-resource-twr/README.md)

## 2. 层级几何与呈现

- [x] 2.1 `twr` 在正常 close 逆序证明后核对每层主行止于该层 close 起点；旧单行布局与新分段布局同测，单资源路径不退化。root 独立复跑几何 4/4、既有 guard 13/13、null resource 5/5、TWR catch 1/1（另 1 项既有 ignored）。
- [x] 2.2 当外层主行未覆盖内层 handler 时，只接受范围精确相等、同类型、同目标的伴随保护行，并纳入已解释行及来源；缺失、额外、交叠、错误类型/目标的 patched 控制拒绝保持。新增样本进入预期的 continuation 拒绝而非 RangeEnd；verifier-valid patched 行与重复伴随行保持拒绝，来源含伴随行首末 BCI；root 审读预算停止传播与行集合。
- [x] 2.3a 对 close 后纯 `load; return` 的体内已求值结果，用终端 CFG 块与 SSA 唯一 store 证明值来源和尾部无效果；已证 return 由 Builder 放入 TWR 体内。不同值或额外效果仍拒绝，旧 guard 定向回归通过。[Root 验收与未闭合的高层作用域边界](verification-2.3a.md)
- [x] 2.3b 完整方法验证 return 与资源体局部声明同一词法作用域：已证 TWR 清理 handler 的非连续块由 Guard 精确认领；仅按这些指令 BCI 从源局部声明规划排除 synthetic 读写，不按全方法物理 slot 跳过声明。竞争异常表行保持拒绝。其它用户 handler/跨异常区作用域仍归 [异常区局部作用域任务](../preserve-local-scope-across-exception-regions/tasks.md)。[Root 验收](verification-2.3b.md)
- [x] 2.4 多资源一条头的呈现：资源按初始化顺序、close 逆序证明逐层复用、suppressed 链锚点保留；精确认领每个资源初始化的 `new; dup` 生产者（冻结样例 BCI 0/10）。**已实现并复跑正例**：field/concat/site 在 `region::recover` 前规划，已证 Sites 只读交给 Region/Guard；资源 Store 的 SSA 值与站点结果和完整指令集合一致后才领入头。文本含两个资源声明、体语句及体内 return，无 `@bytecode`，头、close/suppression BCI 来源齐全；`Guarded.two/three` 不退化，几何/伴随行负例仍引用。2.3b 的 handler 局部作用域已闭合。**新增验收**：`new`/`dup`/`<init>` 的 Store 值缺匹配 Site 时不再落入通用单语句路径；冻结双资源的外层 BCI 9、内层 BCI 19 分别撤销站点或使用空 Sites，均精确拒绝，匹配站点则保留两项正例。**补齐验收**：[effectful-header-negative 与 constructor-identity-negative](../../evidence/java-syntax-2026-09-26/multi-resource-twr/README.md) 分别在 Store BCI 12/13 精确拒绝且整方法保留引用；两类样例均通过五模式 `-Xverify:all`，Rust 负例与正例、Java 8 重编执行对照通过。旧 `jre_guard_span` 输出仅是 [2.3b 阶段证据](../../evidence/java-syntax-2026-09-26/multi-resource-twr/after-2.3b.md)，不是当前主干状态。

## 3. 对照与门禁

- [x] 3.1 当前 Engine 精确输出的 `run()` 方法体在受控 wrapper 中经 `javac --release 8` 原样重编，原冻结 class 家族与恢复 wrapper 均以 `java -Xverify:all` 限时执行；正常、体异常、内层 close 异常、外层 close 异常、抑制链共五种模式逐字等于冻结的 `release8/runtime.txt`。完整 class-source 的其它成员尚有独立 fallback，因此本项的重编译范围明确为已恢复的 `run()`，不声称全类可编译。JADX 展开嵌套 try 且重复 close、抑制顺序偏离的对照在 [三方证据](../../evidence/java-syntax-2026-09-26/multi-resource-twr/README.md)。自动回归为 `p3_multi_resource_twr_geometry::recovered_run_compiles_as_java_8_and_matches_original_five_mode_trace`（需 JDK，显式 `--ignored` 运行）。
- [ ] 3.2 复跑 guard、twr、typed_catch、execution_comparison 回归；`cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-multi-resource-twr --strict`；golden/语料计数若变重录并说明。定向 guard/TWR/typed-catch 与 OpenSpec strict 已通过；strict Clippy 被既存 lint 阻断。`p3_execution_comparison` 的 `p3-handlers/v8` `syncBody()V` 仍被引用而对照声明为 Java，在 TWR 合入前的 `87f29dae` 上独立重跑同样失败，归独立基线债务；本项门禁因此仍未完成。
