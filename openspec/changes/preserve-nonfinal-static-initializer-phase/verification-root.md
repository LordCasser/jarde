# 非 final 静态初始化阶段修正：root 验收

当前完成 6/7；本片代码/证据提交推送 main，最终完整产品 CI 待验收，不借历史产品绿灯。生产只有一处准入条件：已有字段身份/flags 校验后，只有 final 字段保留常量 RHS 的阶段拒绝。未知外部静态字段、整组写入、类型、读写次序、副作用、异常边、预算与投影原子性继续复用既有证明，没有新增机制。

## 实际完整类对照

literal/ordered 各自保留双 JDK 原程序 2/2、fresh JADX 1.5.6 default/none 4/4、旧冻结 Jarde 2/2。两组共 62 条真实命令、16 腿，[基线 root 验收](../../evidence/java-syntax-2026-10-10/array-field-initializers-root-verification-v2.json) **2093 checks/0 errors**。原 Jarde 语义正确：literal static数组已提升，ordered因 nonfinal trace=0 被阶段条件拒绝而留static块；实例b均留构造器。

新 CLI 在原固定 class 上完成四份 fresh 候选全源码重编和 -Xverify 执行，共16条命令，四腿 **4/4**。源码逐字等于 class-source JSON，无手工修正文；空 CP/SP，不借原/helper class。原 exit/stdout/stderr 是 oracle，包含第二次构造后的 Java int 溢出结果。

[root 实际独立验收](results/candidate-root-verification-v1.json) **2245 checks/0 errors**：闭合 inventory/hash、工具与产品 pins、真实原 class 物理成员/flags 和 source-map 完整 owner/method/BCI，proof write_bci 对应原 putstatic 字段。新编 class 的独立 javap 检查 nonfinal/no ConstantValue，不能用新编 BCI 冒充原指令来源。ordered现按 trace/before/a/after 提升整组，根源码无重复static块，物理clinit及原来源完整保留；literal静态数组仍正确，实例字段不在本片范围。

冻结 CLI `/private/tmp/jarde-nonfinal-static-cli-v1` SHA dda511224111d8f7fe3e22b6e2800e5b2e1311dd75f899a69233f19fe6f2b36e，metadata SHA d5cfcd4f60eb3f241dc913cd56a2aefa3740e167ee3cfca98150b8a9c2416c5a，见 results/candidate-cli-v1.json。旧71f CLI与基线保留，不覆盖历史。

## Rust 与架构边界

静态投影 **6/6**、接口证明 **4/4**；新增 nonfinal runtime常量+有序数组正例、无ConstantValue的blank-final常量负例。漏写/重复/未认领效果/前向读/ConstantValue混合/异常与预算取消回归通过。相关 Clippy（沿CI既有债务豁免）、fmt、全部OpenSpec strict及diff检查通过，raw见 results/root-static-tests-v2、root-interface-tests-v1、root-scoped-clippy-v1、root-validation-v1。

Luna patch v1上下文遗漏既有unknown-static拒绝分支，apply check失败未写产品；root v2仅生成准确上下文。首轮测试编译误用SourceMap私有segments字段，root改用公开segments()后通过；失败raw保留。候选prepared verifier未执行，root v2修正原BCI来源、完整method身份和javap省略owner格式，才实际执行接受。

实例跨构造器提升另见 [架构分析](../../evidence/java-syntax-2026-10-10/array-field-initializers/instance-promotion-architecture-root.md)，未运行的下一片探针另存 instance-field-init-next。不能将现有clinit候选扩义为构造器集合；CF16小栈边界也不混入。此片不代表 EM18 整单元完成。

## 最终产品门禁与磁盘

新产品须提交推送后验自己的确切CI，两个固定seed全仓、真实Temurin25完整对照、MSRV、全仓Clippy/fuzz/supply及reader/fingerprint不借 acd/6dd 旧结果。results/verify-ci-product-root-v2.py 已审阅准备但未执行；准备版错误的22 canonical计数和候选结果字段名已按实际16文件/真实schema修正。

本机限定20 GiB余量/1 GiB target守卫，实际测试/build/Clippy均未停；冻结CLI后只clean本仓target **658441395字节**，target已不存在，全部输入/raw/CLI hash未变，见 results/root-clean-v1。后续其它进程用量以实时空间为准。

## 7196 产品 CI 的旧断言与实际修复

确切7196a365 CI38010503464失败于 enum_constants::tests::ordinary_class_fields_and_static_initializer_keep_the_existing_projection，仍断言bare first/second与static块。完整API/stable原始日志在results/ci-7196-failure-v1；另3job成功，不能算产品验收。此次仅更新该测试函数，Proved2顺序/field index/static nonfinal flags、根声明与无重复块、物理clinit原writes及write BCI来源全部校验；enum NotApplicable、default/all正文一致继续保留，无生产变化。root本机精准测试1pass，20GiB/1GiB守卫未停。

同一SOURCE以-g重新编译双JDK，默认/完整证据4份全生成源码原样重编、fresh-Xverify/raw等于原程序，24命令/76闭合文件，root实际独立验收results/ordinary-static-regression-root-acceptance-v1.json。CLI并不serialize Engine enum_constant_proof；首轮v1仅因错误要求此JSON键而判失败，实际四运行均正确，raw保留；rootv3脚本修正视图边界并重新执行v2采集接受。该Engine断言由准确Rust测试验证。冻结CLI产品10pins未变化，cfg(test)的enum断言修复另pin，不改历史metadata。

已清本仓target286902139字节，raw/CLI/pins保留，见results/root-clean-v2。新CI verifier v3已准备额外测试pin/差异仅限目标函数和双seed确切测试名，尚未执行；本片仍6/7，须待修复提交自己的全仓CI。CF16的acd检查点已独立11/11验收，不借其绿灯。

## 25c5 CI 第二个历史整类断言

25c5a6fd09ed8603dbbbf3d217469432f27e5c40确切CI38012927864的MSRV/fuzz/supply成功，stable seed1失败于tests/p3_array_slot_retype_locals.rs旧A1整类golden。API原始JSON在results/ci-25c5-failure-v1；首次日志网络EOF/零byte失败保留，第二次实际完整日志357068 byte在ci-25c5-failure-v2/stable.log.gz。不是新数组正文回退：root冻结CLI实际A1整类结果与oldbaseline仅差static calls=0提升及删除冗余static块，物理6methods/1field、Proved1/fieldindex0/writebci1/order0保留，见a1-static-regression-root-v1。

仅最后A1 test与其已unused const更新；历史baseline/A1.jarde.java不变，新增expected/A1.static-init.jarde.java逐字等于root实际CLI输出。保留整类原/恢复编译运行，补proof顺序、无重复根static块、物理clinit及source-map BCI断言，其余五项原断言不变。root-a1-static-tests-v1实际6/6，通过20GiB/1GiB守卫，最低22362042368字节。本产品仍6/7，必须待再次修复提交自己的完整CI。

A1相关Clippy实际exit0（root-a1-clippy-v1），fmt/all OpenSpec strict/diffcheck通过（root-validation-v3）。只清本仓target362.4MiB（root-clean-v3实际stderr），10产品pins/CLI/meta不变；未声称本机全仓测试通过。

## 2daa CI 的 corpus 登记遗漏

2daa21c99db7e126d91b5ccbe3d016208fd54aba 的确切 CI38014660202在stable第一seed失败于 `corpus_files_match_the_recorded_fingerprint`，仅缺本次新增 `expected/A1.static-init.jarde.java`；A1六项恢复测试已通过，另MSRV/fuzz/supply成功。完整API/stable日志572529 byte及真实采集argv/exit/hash保留在results/ci-2daa-failure-v1。不能验收失败CI，也不回退合法静态提升。

root实际执行register-a1-fingerprint-luna-v1.py，以Python blake3 1.0.11独立扫描与Rust测试相同的两root/排除规则，2062文件全部核对：原2061条字节/digest恒同，唯一新增2210字节A1 expected SHA a8b84e8cfd3fa4610a05ecd9ef3f1204eb267a063aa3fcd0d28dcc6d73cc96ca，BLAKE3 eaac73c5450923dadab849829f1dc2ca1e9503b0c5c3aa14fbd7ab39df6cc7b4与真实CI相同。manifest仅增加这条sorted entry，classification/原数据不改；新manifest SHA6f7eac52f10e231cd45666438a31e9807bd4ba442c8ac4abfb83d56f51a38147。真实结果register-a1-fingerprint-root-v1.json及wrapper raw完整保留。

本机空间低于20GiB且target已不存在，因此此次未执行官方Rust generator或P5 Rust测试；独立Python全量核验不冒称Rust门禁通过，下一修复提交须再次验收自己的双seed全仓/P5完整CI。本片仍6/7。生产/测试逻辑、原fixture和冻结CLI/metadata不变。

## 41b8 CI 的 Unicode 声明旧断言

41b8e17516b21091e68f3c9f03fa5bd6d041cea6 的确切 CI38017574260 已失败：stable job114111169840第一seed在 `the_cjk_declarations_are_presented_as_the_pool_states_them` 要求 `static int 变量;`，实际为 `static int 变量 = 1;`；下一字段也已正确恢复为StringBuilder表达式。原UT.java本就初始化两字段；同一CI中双腿整类编译/运行、ASCII逐字控制、标识符别名测试均通过（Unicode 3pass/1fail）。这不是全仓成功，MSRV/supply/fuzz三个成功也不能代替stable。

root实际捕获完整API及stable stdout644954字节，原hash c5d670659f233c87c42a71099a5119fff5bc7181f66608aee2c1ca9136bc9d18，gzip hash0f368d61fe91c5fa3eaa45ef09c154b0f348908bef6293c9ef080e401288b497；真实argv/exit/stderr/hash在results/ci-41b8-failure-v1/capture.json。私有test-only两字符串修复已准备，root未以此声称Rust测试通过；v7完整CI verifier不得对失败run执行验收。后续修复须自己的完整CI，本片仍6/7。

root随后只应用这两字符串，独立git blob比较确认其余文件文字恒同，source新SHA9776f2f52eb7324fd83e5f5e389181c0577098b419d8773a7b0212e62dfb0890；10产品/4test/16canonical冻结pins逐一仍恒同，详unicode-test-only-root-audit-v1.json。本机约14GiB，未执行Rust测试；fmt/all334 OpenSpec strict/diffcheck实际exit0，raw在root-unicode-validation-v1。

Unicode collector v1使用single-class及两个独立源码输入，不能证明实际两class family，root完整读审拒绝、未执行，原script保留。v2改为原两class恰一PlainJar、只重编outer完整源码（包含nested member）；root完整读delta后实际执行exit0，3.286秒，wrapper在root-unicode-ci-execution-v2。unicode-ci-root-v2保留30命令/60raw streams：原程序双JDK2/2及default/all四完整生成类family4/4全成功，原样源码、空CP/SP、fresh恰UT与UT$内部类两class、-Xverify/UTF8 exit/stdout/stderr等于各自原oracle；8份CLI outer/child完整JSON用于物理表核对。原UT 2fields/4methods（含clinit）、child1field/1ctor，Proved2 writebci1/25，default/all正文相同。独立closed-inventory/source-map verifier尚在准备，不能将collector success替代独立验收；新提交必须自己的完整CI。

### Unicode完整来源验收（2026-10-10 03:40 UTC）

root实际执行 `results/verify-unicode-ci-luna-v4.py`，退出0，接受文件 `results/unicode-ci-acceptance-luna-v4.json`（SHA256 3c55044d39160db928139d7f53e7d06ed7bdb5c7a616fcd58aa2252d0e676bda）。检查30命令、99闭合文件、8renders、6运行腿、原物理成员/BCI及所有映射范围；冻结静态CLI与6fd Git产品pins绑定，当前实例WIP仅作信息记录。v3实际失败于primary-bearing segment计数口径误写，原失败raw不改。CI38019682442仍待全部门禁，任务仍6/7。

### 确切主线CI接受（2026-10-10 03:58 UTC）

root实际执行verify-ci-product-root-v12，退出0，`results/ci-product-v5/acceptance-v12.json`接受6fd51a18dd83d980f2f060246c209fb6fb0afba1 / CI38019682442：4 jobs、52 steps全成功，两seed各354结果记录、3357 passed/0 failed/97 ignored；完整Temurin25.0.4+7对照、MSRV/fuzz/supply通过。完整1752511 byte stable及114221 byte supply raw以gzip保留，API/capture身份/hash已核。此前v10/v11误用Rust源码转义及failure capture字段导致的脚本失败raw全部保留，v12只纠正格式读取，不放松产品验收。结合既有完整源码、来源与准确本地门禁，本change任务7/7；实例片不借该CI。
