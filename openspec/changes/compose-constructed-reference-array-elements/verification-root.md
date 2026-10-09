# Root Verification — Constructed Reference Array Elements

本片7/8任务已验收：架构、实现、双javac完整源集、负控制与本地全门禁完成；提交推送及确切代码SHA CI仍待验收。两轮编译失败保留；第三轮init16/16、完整家族集成1/1及CLI构建exit0。前置类型片代码已推送main `29dcd5e892696e9f6b5adbb657ffa0a7db576c27`；CI37926854875已四job、48steps全部成功，不能把此处原程序或JADX的成功记作新候选成功。

## Frozen Baseline

前片direct-v3完整六类双compiler输入、冻结CLI与永久JADX证据均保留。第一次baseline索引误把六份Jarde生成源码hash写作原始输入源码hash；root独立核查33项发现6项错误，原记录、v1文档与独立勘误不覆盖。修正后33项全部匹配，见 `results/baseline-freeze-root-verification-v2.json`；原始source与生成source身份已分开。

冻结CLI不导出数字ValueId，历史JSON只证明具体new/dup/init/aastore BCI与唯一reader BCI。精确存值身份需在候选内部使用本轮真实SSA核验，不能把BCI集合当成已经证明的ValueId。

## Complete New Family

新fixture自创建起保留七类闭集：Main、Base、Mid、DirectA、DirectB、TwoHop、LocalInterface。六个目标方法覆盖平台CharSequence、Collection、Throwable和自有直接/两跳/接口元素，全部直接new，两个元素的参数mark及自有构造效果可观察。Collection还保留构造参数内的String[]与Arrays.asList依赖链。

Corretto8使用source8/target8，OpenJDK23使用release8；均显式空classpath/sourcepath、g:none，原程序仅在本腿新编译目录以-Xverify:all运行。run-003 manifest为 `98e8f014b4cb7f8a4efca50ff54e7fee992805f27387f95d4c76db1f9b5bc713`，原两腿stdout相同，stderr为空。早期脚本计数失败保留。source-plan“十源文件”是旧计数文字错误，README独立勘误；真实manifest与闭集均为七类。

| 同一新冻结输入 | 完整编译/验证运行 | 原始双流一致 | 完整语义验收 |
|---|---|---|---|
| 原程序，两compiler腿 | 2/2 | 2/2 | 2/2 |
| 上一片冻结Jarde CLI | 编译均exit1 | 不运行失败源码 | 0/2 |
| JADX rename-flags none | 2/2 | 2/2 | 2/2 |
| JADX默认重命名 | 2/2 | 0/2 | 0/2 |

root分别生成全部七类源码，任何额外源也参与编译。运行classpath仅是该次新编译目录；未借原类、原helper源或剥离成员。JADX默认重命名改变可观察类名，原始输出不归一化。最终原始回放为 `results/baseline-cli1-complete-family-v1/manifest.json`，SHA `a3e835ff98f3c34a886d0ee024732c3967eaa54acbbb2e58cadf73cccbbefb65`；root核对136文件、56命令双流、七类完整集合和隔离argv均无问题，见 `baseline-cli1-complete-family-root-verification.json`。

## Architecture Review

当前代码审计见 `results/current-proof-seam-audit.md`。保留arrays→sites顺序，candidate只读已闭合child facts；精确store BCI/存值身份许可与完整构造验证共同闭合，整条结构链成功才联合提交ownership，Sites census单次move移交。结构证据与Builder类型/呈现成功分别验收，后者拒绝不能丢弃真实结构来源。

Frame::convert_token依据Uninitialized的NewSite身份转换alias并记录WRITE；SSA在constructor BCI创建完成值。SsaValue::replaced_by属于消除trivial phi，不能拿来替代初始化身份。受限candidate应核对真正dup opcode、准确reads/writes、constructor receiver与保留stack slot的完成输出、paired aastore实际ValueId。内部复制正常可能零use；只豁免已证明构造身份脚手架和已闭合child-array结构，参数生产值仍需单次求值及闭包。

普通init verifier此前无Budget；本片新增验证必须在真实scan/use/递归处计费并传播StopReason，不使用全SSA预扫收费或长度估价。五wrapper primitive conversions及外围concat/field/arraylength保持独立未覆盖控制。

## Remaining Acceptance

第一次串行门禁保存于 `results/focused-v1/index.json`。fmt exit0，init focused编译exit101、24项错误：只读child view的impl边界错置、切片使用错误、SSA block身份接口及candidate返回类型迁移未闭合。stderr SHA为 `384acb2ba284a888fb413f4efa03dcd681e5298706c0e3064708e2f2cc09b7fc`；runner立即停止，未执行集成测试或CLI构建。修复与后续重试使用新目录，保留本次原始失败。root额外指出，aastore完成值定义于constructor，不能要求其definition为Allocate；new身份须依赖真实NewSite及本元素闭合区间核验。

冻结CLI1 SHA `69a4a4bf86ca3cda412ea5ca6fba4dda24e84253daca781ddfec2da31b4d8495` 已在真实javac8/23两条compiler腿生成全部七类源、隔离编译和验证运行，双流匹配2/2。完整manifest SHA `e97f6299cf785fdc15bd9b02b1027ada422088055617aadf858d1f685fcbe247`，root独立核对257项、24个唯一构造记录、100个source BCI，零问题，见 `results/candidate-cli1-complete-family-v2-root-verification.json`。CLI预检错误路径导致的未执行记录保留在v1目录。两轮编译失败及第三轮成功均保留，不把当前继续修改的源码当作此冻结CLI。任务2.3完成；focused负控制与全门禁尚未验收。集成草稿只使用七份Jarde生成源编译，原类独立目录作oracle；宿主JDK执行不等于跨JDK候选验收。还需对既有constructor-array/nested/char[]/varargs、来源唯一性、结构失败零提交与类型失败完整正文拒绝、预算/取消、P5与全门禁分别保存真实结果，随后提交推送并检查该代码SHA的CI。

## Candidate CLI1 Additional Comparisons

原18冻结输入保持同一jar/oracle，完整成功由8/18增至16/18：新增CharSequence、Collection、Throwable、自有子类各两compiler腿。BigDecimal两腿仍有拒绝正文，虽然compile exit0但输出不符，均不计成功。旧v3 factory全六类仍2/2；direct全六类仍0/2，primitive conversion及nested covariant child-array保留边界。root核对652项hash、完整源集/隔离argv和上一片全部成功保留，零问题，见 `results/regression-replays-root-verification-v1.json`。复用历史runner时已删除的附加fixture路径预检失败留在legacy-18-v1，不覆盖，实际18腿为v2。

独立NestedControls完整单类使用 `new StringBuilder(new StringBuilder(mark("nested")))` 作为Object[]元素。两原JDK的Jarde、JADX none、JADX default六条完整compile/verified-run均双流匹配；没有反射类名观察，不需默认重命名归一化。root核对94项hash零问题，见 `nested-controls-cli1-root-verification.json`。它不替代七类分量兼容family。

`focused-v4` 新测试缺少Type导入导致编译失败，保留原stderr。`focused-v5` 实际fmt/init19/19/完整及控制集成2/2/CLI构建均exit0且无编译警告。嵌套outer/inner独立ownership、准确存值、错误array/index ValueId/head/position/dup、第二i2l元素失败零candidate/零pending通过。预算测试改成直接受限verifier入口：outer scan BCI10收费后递归，第二AnalysisSteps收费在inner BCI14精确停止；不能用整个ArrayInitializers预扫描停止时的同BCI碰撞充当递归证据。BigDecimal→Number控制结构记录保留而完整正文拒绝，与结构失败零提交分别验证。额外stack reader、fresh直接索引变体与真实handler控制尚缺，task2.2不勾选。

## Completed Adversarial Controls

`focused-v6` 的init22/22、新完整及控制集成2/2、旧数组集成4/4全部exit0。两compiler腿fresh sequence派生重复/逆序索引，零parent与pending提交；extra-reader在aastore前插入dup_x2再store/pop，真实SSA核对两个Ref别名分别被store19/pop20消费，准确构造完成值guard拒绝，原码对照guard成功，完整候选不提交。原方法max_stack=6足够，不盲目增加。HandlerControls两腿仅改合法编译结果异常表start0→12，source仍同block、store实例验证仍通过，array allocation与mark/constructor/store的真实SSA handler向量不同，组合在效果闭包拒绝、零pending。全部输入仅在测试内派生，不改冻结源/class、不执行故意变义负例。task2.2完成。

reader-census真实读取后因旧固定计数失败保留在focused-v6，实测为1055/4532/459/2711/8，即本片20class、64Code、2handler、10branch、零subroutine；已更新固定断言，尚需重普查与指纹/P5及全门禁。

## 首次全仓回归与公共区间闭包修复

`corpus-gates-v1` 的reader重普查、指纹生成与核验、P5均exit0：fingerprint 5 passed/1 ignored，P5 19 passed/1 ignored。首轮 `gates-v1/workspace-seed1` 实际351 targets、3327 passed/1 failed/93 ignored，exit101；原始stdout SHA `0ff2751766a5bebfa76cd8897cc51f612184f7ef46470fa222b73bbce877e223`。唯一失败是已有 `p3_local_postfix_array_elements::plus_two_and_other_slot_are_not_local_postfix_values`，未执行第二seed及后续stage。失败日志、summary与index均保留，不迁移负例断言。

组合重构把旧公共 `interval_is_expression` 检查移入构造Site分支，造成generic和child-array分支遗漏。DifferentSlot场景中的无关iinc因此从元素区间移到整个initializer之前；这个具体局部值未继续读取并不能授权一般重排。root与独立Luna审计一致，恢复三分支汇合处的公共完整区间检查，删除Site分支重复扫描，逐条计费只执行一次，无新机制。说明见 `results/generic-element-effect-regression-fix-v1.md`。

`focused-v7` 七stage均exit0：init22/22、新完整及控制2/2、旧array4/4、原postfix3 passed/2 ignored、reader census与CLI构建通过。公共闭包修复后CLI2 SHA `78cfb53212489c017a8ac64292df2531d65300739cdc3297435d76993b2fa273`；完整源集回放和新的双seed全门禁继续验收。尚不勾选3.1/3.2/3.3。

## CLI2 完整源集复验

公共interval修复后，`candidate-cli2-complete-family-v1` 两真实JDK腿完整七类隔离编译/验证运行成功2/2；manifest SHA `3c44a58483928ee5ac2db4d37ba20dcdc9e547138b764bc9e2ea318723440582`，root257项hash/argv/24唯一new/100source BCI核验零问题。旧18腿仍16/18，旧factory仍2/2，旧direct完整六类仍0/2。原BigDecimal失败仍compile0但正文拒绝且双流不符，不以可编译空体计成功；额外656项hash/完整源/隔离argv及与CLI1成功集一致检查零问题，manifest hashes见 `results/cli2-additional-root-verification.json`。历史类型片回归核验另652项零问题，见regression-replays-root-verification-v2.json。NestedControls两真实JDK的原/Jarde/JADX none/default均完整双流一致，六比较腿全通过。

`focused-v8` 七stage均exit0，曾包含一个新增child-array间隔负控；root发现插入无关更新会在已有array_initializer_reader先拒绝，无法定位公共parent interval guard。因此该测试/helper不采纳，不把23个init测试的临时通过当最终22项验收。v8原日志永久保留，现有DifferentSlot/postfix回归保护公共postlude。恢复公共guard是原不变量，不能靠额外测试机制制造不存在的独立证明。

## 最终本地门禁与清理

`gates-v2` 实际12 stage全部exit0。两独立seed各351targets/3328passed/0failed/93ignored；现有varargs3/3、P5 19passed/1ignored、MSRV1.88、fmt、CI-exact Clippy、ignored P3 3/3、functional-constructor双输入完整对照1/1、全OpenSpec strict及diff均通过。root核对93项exit/原始streams/hash/CI参数及最终产品源码与冻结CLI2一致，零问题。index SHA `ff4e8ed40a6e3f9e9b7a85556e962d24e6c7eefb476e5c3807a01bf5c8962da4`，见results/local-gates-root-verification.json。不是把历史通过结果用于当前源码。

两seed后按本项目磁盘纪律清理3060文件/13.2GiB，再完成余下门禁；最终再次清理5862文件/814.8MiB，主仓和fuzz均无target。冻结CLI2完整保留hash不变，最终可用32877355008bytes（约30.6GiB）；外部项目占用实时变化，不触动其它项目target。原始清理双流/hash见results/cargo-clean-final.json与gates-v2/cargo-clean-mid-gates.stderr。14辅助worktrees仍detached/clean/main祖先/无target，只有main/origin/main，无剩余分支占用。

3.1/3.2已勾选。3.3必须等提交推送后实际代码SHA CI全部成功，不能以本地green预先勾选。下一数值转换OpenSpec规划4/4、0/7实现任务，仅源码草稿；独立保留BigDecimal、nested covariance、member/statement与一般alias边界，不把当前片当整个EM18追平。

原始门禁日志在首次staged diff检查因Cargo stdout终尾空行exit2；不修剪原始stream或重算历史hash，沿用相邻片已有的results/.gitattributes（stdout/stderr/log仅关闭源码空白检查）。失败与正确配置后的exit0分别保留staged-diff-check-v1/v2.json；源码与文档空白仍正常检查。
