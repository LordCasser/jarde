# Object 参数展示包装：证据和执行状态

当前 **5/6**，实现、最终广门与完整源码/JVM对照已独立接受；fresh workspace、产品提交推送及自身 CI 待完成。finThrow/nestedFin 及前片已经交付，不重做；本片不计 CF11 或 71/612 整项完成。

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
