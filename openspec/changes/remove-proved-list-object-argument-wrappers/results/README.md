# Object 参数展示包装：证据和执行状态

当前 **6/6**，实现、最终广门、完整源码/JVM对照、fresh workspace及产品自身CI均已独立接受。finThrow/nestedFin 及前片已经交付，不重做；本片不计 CF11 或 71/612 整项完成。

root 已独立接受 controls-baseline-root-v3：141 实际命令、282 原始流；12 原始目标/配置编译，8 VM/64 case，12 错 CodeSource 拒绝，24 Jarde default/all、12 JADX、72 完整生成源码编译。40 编译成功，32 次 Jarde single-class 输出因未提供 nested Receiver 定义而编译失败，不借用原始 inner class、不算 runtime 通过。初始采集不含生成运行；后续实际运行见下节。78 份 class 已由 root 重读双解析器全部 Code/BCI，原始 12 份额外独立 CP 引用绑定调用目标；3305 来源与所有输入/产物/原始流 hash 重核，无 target。

两编译器 debug/none 和双 VM 原始 oracle 一致：Integer 引用选择 remove(Object)，primitive int 选择 remove(int)，同值1分别留下[2]/[1]；自定义 equals 的 Object/String 路径不同；真实参数 String CHECKCAST 保留错误类型异常与 null=false。早期 Integer 拆箱反例撤回，v1/v2 更正均保留。

平台包仅包含4实际 javap命令及清理，root接受 String final/唯一 equals(Object)、List两种remove的 SDK 定义，不单独当重载选择证明。controls review 独立脚本与 JSON 单列归档。

规划已 strict 通过。root 已读源码、Luna复用设计及对抗审查，明确采用任意**已证引用局部**的 remove(Object) 窄规则，不采用 memo 中不必要的 List<String> 限制；String.equals 则要求精确 String receiver 和调用。沿用 opaque AST 精确包装事务与同次 SSA，不加公开机制；可选缺证不得撤回核心投影。

实施已通过 collaboration.followup_task 派发现有 Luna `typed_foreach_impl`：读取本 change 全部规划，私有补丁输出 `/private/tmp/jarde-object-argument-implementation-luna-v1-20261018`；根独占共享/Git/工具链。此为历史派发记录；当前已集成并验证 v1/v2/v3，最终结果以下节为准。另派 Luna 准备成功编译产物的 mixed-root JVM 重放工具，root 已实际执行并独立验收（见下节）。所有4归档已 root 逐成员按原 bytes 复开，见各 capture-review.json。

准备失败保留：v1 driver field 顺序解析不符；v2 实际预检因 driver 已改 Base64 却仍 pin 旧 bytes 而拒绝，未运行 JDK；v2 parser 返回 dict 与下游字符串契约不符。root v3 仅修 pin/返回契约，保持原始 Java targets 和全部编译失败，最终实际采集成功。

## 基线生成运行独立接受

`controls-generated-replay-root-v1`：root 实际120命令/240raw，40成功整类编译树双VM共80次，224重建目标case全部同原（Jarde32VM96case；JADX48VM128case），40wrong-root拒绝；伴随的2个原始target不计重建case。全部3305来源、3838基线引用、raw/argv/根角色/CodeSource重核。此为旧26eb基线，不冒称新实现已通过；32旧整类编译失败仍未运行。

## 最终实现与完整对照，5/6

最终 CLI56a589ef9a170a1bb2911ed5597c20b2b4e06abf78da845f8fd3946099b2fa51/63624704B；`final-broad-root-v2` 18命令/36raw、812通过/0失败/6旧忽略，fmt/CI Clippy/MSRV1.88 通过。3194来源不变，target absent。较早 e22 广门811及focused均为中间结果。

`final-string-comparison-root-v1`：完整固定原始4VM32case；完整生成20编译18成功，36VM288case/18wrong-root全接受。2次JADX outer在JDK8的major55框架边界保留。最终56 default/all、debug/none、child/bundle-child与旧基线精确差分只删两处Object参数包装，其余非usage字段/raw/map不变。

`integer-comparison-root-v1`：e22实际10完整源码编译/20VM120case，原始2VM12case；CP核remove(Object)，同编译器产物Code一致。两次原始源编译仅inventory、未运行。`final-api-binding-root-v1` 最终56实际26API：24控制非usage JSON逐字相同于旧26eb，2Integer相同于e22，完整Java相同使既有真实VM可复用；56这里只跑API，不冒称重跑VM。

`implementation-luna-v3` 按唯一equals key分别授权，新增真实fixture两项缺证/错key变形门。v1对抗审查的remove AST blocker已在v2撤回：缺核心receiver closure不能通过可选参数规则放宽，保留原审查及更正。所有最终包均由root逐成员按原始bytes复开，capture-review记录SHA。64MB CLI排除出归档但完整pin及真实路径保留。

主java8-recovery已同步本片一个行为requirement（5场景），18/18 strict通过。`acceptance-preparations-root-v2` 仅工具准备，尚非native/CI接受。下一固定Sequential的旧冻结baseline已接受；最新8API及既有statementful header路径分析在`sequential-fixed-analysis-root-v2`，仍待实际独立review，不能增加71/612覆盖数。

## fresh workspace 与下一固定 Sequential 完整对照

`workspace-native-root-v1`：本轮385目标/101批/3770通过/0失败/98旧忽略，20精确门各一次；默认栈及5GiB/1GiB守卫、source不变、exactclean通过。3194构建与5147native并集5154来源按Git blob/bytes/mode绑定产品096a764dce8bb29b2d7d79e86bef6bea880bc848，CLI56完整pin一致。该产品已推，CI39138950945仍未完成。

`next-fixed-independent-reviews-root-v3` 保留Sequential API checker v1/v2未实际核旧raw却声称核过的缺陷，root两次退回后v3补真实cell→command→argv→input→raw==report绑定，root实际重跑通过；不以旧memo替代执行。下一Seq2既有交付只读备忘，不冒称当前CLI重跑。

`sequential-complete-comparison-root-v1`：最终56已有Seq1八API/25BCI完整map被独立接受，完整原样源码实际70命令/140raw、24编译22成功、44VM264case同原，1wrong-root拒绝。Jarde16编译/32VM192case全部成功，JADX6成功编译/12VM72case同原；2JADXbundle/JDK8 major55旧框架边界保留不运行。collector v1先用后赋evidence的静态阻塞由root发现，Luna v2最小修正后才实际运行。无新恢复机制；CF11仍不整项增加完成数。

## CF11 当前32 API及下一明确缺口

`current-fixed-api-and-next-architecture-root-v1`：原收据32组argv/input/raw已先由root绑定；NL2/NL4旧临时路径迁至归档同SHA class，其余参数不变。最终56实际33命令/66raw（32API+clean）；root重跑独立checker并补全全部raw/guard/cwd/3194live来源及两个独立classparser核验，通过16对default/all全文/raw方法/map一致和全部physical方法/BCI覆盖。NL3/NL5旧baseline fallback已由既有交付解决，不能重做。此为当前API/来源验收，不冒称本轮重跑它们的JDK或整CF11已完。root补充checker一次错误假定Seq2/Seq1 parser同hash而拒绝，已记录并改用两份明确pin的双解析对照，无产品失败。

NL2当前仍有独立泛型质量缺口：结构化循环完整，但`test(List<String>)` Signature被安全拒绝，保留raw List及真实String cast。Luna只读架构备忘定位VoidBody/void-result候选和缺get(I)Object合同两道门；现有result_value_fact可复用，无需新IR。后续先分析primitive/void结果与参数用途census分离、get结果→唯一物理cast(E)/AST闭合；保留cast，不凭Signature放开正文或推断E。尚无下一OpenSpec/实现，本片不混入。

## 最终交付 6/6

产品096a764dce8bb29b2d7d79e86bef6bea880bc848已推，自身CI39138950945独立接受4jobs/52官方步骤、两seed各385目标/3770通过/0失败/98既有忽略、65关键门各一次。`own-product-ci-root-v1`保留25采集命令（24原采集+1官方ZIP）和verifier执行；官方ZIP全部52独立step齐全，零aggregate重建。最初缺ZIP的checker正确拒绝不完整证据，补齐后完整验收，并保留拒绝记录。文档后继不声称拥有另外的产品CI。

主规范一个requirement/5场景已同步；最后关闭文档后由外置final-clean-main收据记录实际HEAD/远端、干净main、唯一main分支、aux detached-clean-main祖先、来源/CLI及target终态，避免将自包含文档hash冒称提交事实。下一NL2泛型缺口仅分析，无新Spec/代码；当前leaf不增加整CF11/71/612完成数。

最终strict376/0、apply all_done6/6；24归档/2025成员已root逐字复开并与原capture-review SHA核对，见all-archives-root-review-v2.json。最终planning命令/exit/原始流存于final-planning-validation-root-v1。
