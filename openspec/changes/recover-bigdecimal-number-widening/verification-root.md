# Root验收：BigDecimal→Number（进行中）

产品只在现有release-8 NUMBER_FAMILY增加准确BigDecimal→Number行；原release gate、六装箱行与表外拒绝保留，concat规则未改。Luna只写实现与测试，以下命令均由root实际执行。当前任务5/7；完整24腿已验收，严格门禁及确切本片CI未全部完成。

## 已执行

- results/root-focused-integration-v1：生产Engine两original Main与两NumberArgument类输入的结构测试2pass；两Java执行测试默认ignored。Main必须完整structured/java、BigDecimal构造一次、Number数组一次、三append/一次toString、实际System.out与来源BCI1/6/9/12/15/16/17/28/29/34/40/43/46。NumberArgument以准确identity(Number)Number descriptor、调用BCI12及完整构造/调用/return来源验证。
- results/root-java8-execution-v1、root-java23-execution-v1：分别显式启用ignored测试，各2pass；每个测试都覆盖两编译器原class，合计八次全类空CP/SP重编与只运行新classes的-Xverify:all执行，逐字匹配冻结stdout/stderr。Main oracle为1:1.25\n，NumberArgument为2.50\n。四个JVM注入环境变量全部剔除。它们不能替代下一步新CLI24腿回放。
- results/root-all-java-lib-v1：325passed/0failed，包含the_number_rows_reach_exactly_their_pairs、direct_child_array_builder_uses_exact_store_type_proofs_and_stops_atomically、nested_composition_budget_can_stop_inside_recursive_constructor_proof，以及既有共享预算/取消/原子拒绝控制。只加常量行，没有新增计费循环。
- 新NumberArgument的真实双JDK原编译、原执行、javap物理调用与工具/hash见results/number-argument-original-v1/manifest.json；原Main字节与上一片冻结baseline一致，不另造成功输入。

## 磁盘中断

results/root-candidate-cli-build-v1实际exit-15；空间低于21474836480bytes时守卫仅终止自己的构建进程组，不能计成功或冻结为候选。root-clean-disk-stop-v1随后cargo clean只清本仓446MiB左右编译产物，原日志/冻结CLI未删除。CLI v2是否成功须读其result.json，不能借v1结果。

## 待完成

本地完整回放和严格门禁已完成。尚待推送后的确切本片产品CI双seed全workspace及全部四job；71单元与EM18部分分类不变。

## 新CLI完整24腿验收

root-candidate-cli-build-v2 exit0，冻结/private/tmp/jarde-bigdecimal-number-cli-v1 SHA **b79520629443a0211cd656d1374e415f76ac6adf400d79cb86154954caba9cb0**。五源身份与build raw结果见candidate-cli-v1.json；初次错误查找不存在的binary名未复制任何文件，核对Cargo manifest实际binary为jarde-cli后再冻结，记录保留。

root-complete-candidate-replay-v1实际102commands、384closedfiles、24/24成功：原18矩阵18/18（两BigDecimal现为正例）、完整六类factory2/2、direct2/2、数值2/2。两个真实JDK空CP/SP重编所有生成source，runtime只有新classes且-Xverify:all，完整原exit/raw双流逐字匹配。original/JADX采用逐hash核对历史baseline，没有fresh提取/执行这些历史基线。

root-independent-candidate-verification-v1：**1226checks/0errors**，manifest SHA **7e56b01f68d9240f0441a7ade433909815cdb83817b41d617990c9c7ba8893c2**。原始成员/完整source闭包、物理anewarray Number、BigDecimal准确new/dup/ctor、aastore15及三append/一次toString/实际System.out/source-map锚点全部确认。collectionGridDirect仍保留准确Collection<?>[][]声明与成功Signature marker；没有删除失败成员或放宽原22腿控制。此24是冻结控制腿分母，不是71语法单元完成率。

## 当前finally队列复核（非本片新增实现）

current-finally-v1实际19commands，固定原ImplicitCleanup.class，fresh渲染完整类并在双真实JDK/all和essential四腿重编/执行，四路径raw流与原程序全部一致。all/essential完整文本同hash；all模式包含原17个保护体/cleanup/return BCI来源。essential CLI按请求省略可选source-map，初版verifier错误要求其存在而失败；v1失败日志保留，依据现有CLI证据选择语义修为v2后独立通过，不修改产品。widened仍引用run且未执行mutant。

原cleanup-over-return trace29，保存的历史JADX提取本次双JDK重编/runtime均trace299；不声称fresh提取JADX。它已实现，不能据旧unchecked任务重复立项。structured子Region递归预算/取消/事务回滚专项覆盖仍须独立核对，四条语义正例不能当作全部CF16或其任务均完成。

## Corpus变化

root-reader-census-v1旧pin真实失败：新增4个canonical class/10个直线Code body。按测量与物理fixtures改为(1067,4626,463,2711,8)，root-reader-census-v2通过；handler/branch/subroutine未变。root-fingerprint-old-v1真实因11个新fixture输入未登记而失败；预期扩容的regenerate-v1完成，后续verify和P5严格pins结果按各result读取，不删除历史失败或放宽阈值。

## 本地严格门禁完成，CI待验收

local-gate-plan-v1的八项实际exit0：fingerprint verify、P5严格pins、finally两专项、MSRV1.88、CI-exact全workspace/all-targets/all-features Clippy、fmt、OpenSpec all strict、diff。完整命令/环境/双流/runner/source身份见各root-* result。P5 pins无变化，fingerprint只新增11条控制输入。没有在本地重复全workspace双seed，须由新产品确切CI执行补齐。

root-clean-after-local-v1只清本仓target；冻结新旧CLI和所有raw证据保留。新workflow新增显式BigDecimal ignored比较步，默认无JDK的结构测试仍可运行。3.2/3.3等确切新CI全绿后才能完成，不借前产品45b的CI。

下一条P02_multianewarray已由当前新CLI fresh确认：两原class真实运行stdout6\n；完整候选两真实JDK均compile1，helper fallback，不能沿用旧README“compile0/打印0”的行为说法。capture没有拒绝，失败在二维元素compound更新。新baseline仅定位问题，尚无JADX新对照/实现spec/实现；单独推进，不混入本片产品。

## 发布与实际CI状态

产品6997c9f8b2515cb18c359fe55559a486fe47fff1已推送main；CI37989319644已结束为failure：stable seed1在旧closedNumberBoundary负例断言失败，actual Structured/expected Fallback；seed2与显式BigDecimal步骤未执行。MSRV/supply/fuzz成功但不能计产品CI验收，3.2/3.3仍不勾。原JSON与完整gzip stable日志、root失败判定保存在results/ci-run-v1.json、ci-stable-job-v1.log.gz、ci-failure-root-v1.json。

提交后完整diff whitespace检查发现11个原始Cargo stdout的末尾空行；raw bytes必须保留，不能trim。root-committed-whitespace-v1记录全raw检查exit2和排除这些stdout的source检查exit0；所有真实失败保留。此前root-diff-check-v1是工作区源码检查，不能拿它冒称全新raw日志没有whitespace提示。无需修改产品或Git全局规则。

## CI旧边界修正（待新CI）

root用同一冻结产品CLI对两个实际输入重新提取BoundaryControls与BNX，四个完整报告与原双流保存在ci-stale-boundary-current-v2。closedNumberBoundary现为完整Number initializer，每个Integer/BigDecimal构造及mark参数各一次，两个NewRecord及原全部BCI保持；旧拒绝预期需转正。BNX仍wholebody fallback，只有AtomicInteger@46拒绝；BigDecimal@21拒绝消失，完整effect来源闭包仍保留，不能把其正文称已编译成功。v1两BNX成功后root误写Boundary路径中止，partial失败保留。

Luna提供两个旧test更新，root应用/rustfmt并fresh20checks/0errors确认准确预期、Atomic拒绝与全来源；五产品源及原本片测试/workflow都不改。无新增恢复规则。由于其它项目Cargo占用后空闲约19GiB、本仓target已无，root遵守20GiB停线，未运行本地focused Cargo；该限制写入ci-stale-boundary-root-verification-v1.json，必须等新确切CI执行补齐，不能以CLI核验冒称测试已跑。此次CI修正独立提交，不混入下一片nested产品/fixture。
