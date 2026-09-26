## 1. 冻结形状与三方基线

- [x] 1.1 自写两资源 Java 8 fixture（正例：读取返回/内层抛出/外层 close 抛出/正常关闭四条路径；负例：等宽 patch 主行结束或伴随行的范围/目标），javac --release 8 与 9+ 各编译一份，冻结 SHA；记录 javap 行几何、原 class 执行输出（含 suppressed 顺序）、jadx 1.5.6 输出（展开为嵌套 try，记为其偏离）、jarde 修前输出（RangeEnd 引用），并对照旧 `Guarded.two/three` 的单行布局。[证据与 Root 重放](../../evidence/java-syntax-2026-09-26/multi-resource-twr/README.md)

## 2. 层级几何与呈现

- [x] 2.1 `twr` 在正常 close 逆序证明后核对每层主行止于该层 close 起点；旧单行布局与新分段布局同测，单资源路径不退化。root 独立复跑几何 4/4、既有 guard 13/13、null resource 5/5、TWR catch 1/1（另 1 项既有 ignored）。
- [x] 2.2 当外层主行未覆盖内层 handler 时，只接受范围精确相等、同类型、同目标的伴随保护行，并纳入已解释行及来源；缺失、额外、交叠、错误类型/目标的 patched 控制拒绝保持。新增样本进入预期的 continuation 拒绝而非 RangeEnd；verifier-valid patched 行与重复伴随行保持拒绝，来源含伴随行首末 BCI；root 审读预算停止传播与行集合。
- [x] 2.3a 对 close 后纯 `load; return` 的体内已求值结果，用终端 CFG 块与 SSA 唯一 store 证明值来源和尾部无效果；已证 return 由 Builder 放入 TWR 体内。不同值或额外效果仍拒绝，旧 guard 定向回归通过。[Root 验收与未闭合的高层作用域边界](verification-2.3a.md)
- [ ] 2.3b 完整方法验证 return 与资源体局部声明同一词法作用域；异常 handler 复用同一物理 slot 的当前拒绝须先由 [异常区局部作用域任务](../preserve-local-scope-across-exception-regions/tasks.md) 的 SSA/Region 身份证明闭合，不得按全方法 slot 跳过声明或降级拒绝。
- [ ] 2.4 多资源一条头的呈现：资源按初始化顺序、close 逆序证明逐层复用、suppressed 链锚点保留；精确认领每个资源初始化的 `new; dup` 生产者（当前冻结样例 BCI 0/10 缺口）；文本含两个资源声明、体语句及必要的体内 return，无 `@bytecode`；旧 `Guarded.two/three` 不退化。2.3b 的 handler 局部作用域仍是完整类验收前置。

## 3. 对照与门禁

- [ ] 3.1 恢复文本 Java 8 重编译，四条路径执行对照与原 class 一致（返回值、异常类型、suppressed 顺序）；jadx 偏离写入对照记录。
- [ ] 3.2 复跑 guard、twr、typed_catch、execution_comparison 回归；`cargo fmt`、`cargo clippy -p jarde-java -p jarde --all-targets -- -D warnings`、`openspec validate recover-multi-resource-twr --strict`；golden/语料计数若变重录并说明。
