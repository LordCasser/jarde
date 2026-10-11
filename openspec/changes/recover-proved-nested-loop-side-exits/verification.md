# 验收记录

## 1.1 固定事实（已验收）

Root重新执行已冻结独立baseline verifier，实际exit0，核25命令/50 raw、双JDK原类和4份JADX完整类返回12，4份旧Jarde完整源码编译失败无runtime。新receipt：results/fixed-baseline-recheck-root-v1.json；原失败未覆盖或修饰。固定常量return0未动态覆盖。

Root重新逐一核已提交同次IR archive-inventory的26文件SHA/bytes、v2真实1-test和40条物理指令/15节点/19正常边，以及执行前/恢复后4700源码inputs逐一同当前原源码。结果results/task-1-1-root-recheck-v1.json。诊断旧v1编译101原记录仍保留。中间Frame截断28与外层branch10拒绝为实际trace；loop_arm_join_source内部closed_body/owners/exit门槛是实际Region形状+当前源码推导，并非独立helper trace。index5准确指BCI62。

对应JADX源码审阅已提交于同次IR evidence下jadx-loop-exits review。参考其先处理loop exits再转换for的问题分解，复用本仓natural/Frame/Region/transfer/return；不复制其可变CFG或宽泛frontier处理。当前产品尚未实现本片，仍保持固定fallback，不关闭CF11或71单元。

已将同一602-byte真实class与适配源码复制至tests/fixtures/java8/cf11；README记录精确出处、编译参数和SHA。fixture只登记输入，不冒称新产品通过。

## 1.2 受控输入（已验收基线观察）

Root v3冻结collector实际25命令通过采集；独立root verifier v2实际exit0。results/controls-baseline-root-v1保留247文件/4065253bytes、所有50raw、两原class/Runner、JAR、两profile完整JADX源及四份完整Jarde源、输入工具前后pins、独立physical class/Javap/maps核验。10固定输入双JDK原类stdout恒同；四JADX完整类双JDK重编-Xverify:all run exit0且stdout/stderr同原。固定ctor/testFor/runCounts物理指令3/40/47；方法序号/flags/Code跨度/准确owner-name-descriptor及合法UTF8来源均核。BLAKE3身份未独立重算。旧CLI绑定computed-for已接受receipt，不是本片新实现。

原testFor正常12，受控earlyreturn0；runCounts正常12020、early13/1/4、zero-J和zero-K12000，核真实normal/break/skip/zero及early行为。四旧Jarde全文render0、javac1、无generatedclass或runtime，target准确Fallback/ExplanationOnly，default/all全文/map恒同。当前测试fixture直接复制v3已运行1046-byte class，API正断言待实现验收。

保留v1真实collector mkdir failure（18命令）；v2为完整不同输入，tmp++位于outer条件内、实际normal6，不冒称符合固定上游continuation。v3仅恢复tmp++到if外，旧失败未覆盖。独立verifier rootv1实际失败因复用旧verify_map硬编码owner；rootv2核准通用classfile事实后通过，原stderr保留。私有静态候选v1/v2均未应用，proof-aware消费/rollback/unit尚未完成；2.1–3.4未验收。

## 语料登记（局部通过，非3.2完成）

Root实际guarded cargo fmt、ignored fingerprint regenerate（1passed）、fingerprint verify（5passed/1ignored）、clean全部exit0；完整4707-source/include/fixture closure登记，格式变更仅新API文件、生成变更仅corpus-fingerprint.json，verify期间同字节。peak303473250bytes，5GiB/1GiB守卫未触发，target已删除。results/fixture-registration-root-v1保留所有命令和raw及runner前后pin。尚未编译新API目标、尚未应用生产实现或验收fresh workspace，不能借本次registry成功关闭2.4或3.2。

## 2.1–2.5及3.1 同版本窄片接受

Root v4/v5/最终budget候选静态审查，保留自然集合，以局部proof扩展body scope/expected并消费完整child loop/终止return/准确goto break；上游method非源码文本推断。默认栈API3 passed、真实Reader/proof/consumer反例1 passed、旧循环36/try14/Java recovery+gateway69（1ignored）以及完整Java库353 passed，4707 closure恒同。详见独立stack change/results/default-stack-acceptance-root-v1；CLI脚本bin拼错原101已保留，正确bin构建单独同closure接受于results/candidate-cli-build-root-v8，不能冒称原runner全程成功。

全incident-edge rows含内部边完整集合，missing28→31、异常/extra-entry/wrong gateway/shared-return/extra-exit/duplicate owner/已visited child明确拒绝；真Reader JSR clone跨path ID与39→28额外回边仅局部row扰动，不是JVM-valid mixed class或重建NormalFlow。既有真正canonicalized不可约控制在new helper调用之前whole-body拒绝，本片不重复加门禁。

新增consumer晚Stop真实 full12332/limit12331/usage12326@31，四状态回滚；proof limit3/usage3@23，无claims；公开source17858/limit17857/usage17857，AnalysisSteps Partial；预取消usage0，不声称中途异步取消。初轮计费不足已补全新增扫描precharge/poll；snapshot clone仍按既有约定，债务另列。

新CLI61213840bytes/0555/SHA9fc77590ab671ac141dc7fb243e77b7fc103ffc87465c629ca0cf8f43e3da0b3，双JDK固定/受控各29命令都由root独立checker实际接受。所有完整原/JADX/Jarde源编译run0，同原output/计数/全部物理BCI/UTF8 map/default-all恒同。详见results/postfix-root-v1，全原失败保留。当前8/11；3.2 fresh完整检查、3.3自身CI、3.4clean主线交付仍待，不借旧产品CI或把CF11整单元计完成。

## Fresh workspace 首轮 census 失败与最小修正

首轮完整workspace在真实command92 Reader lib停下，177 passed/1 failed，旧语料tuple1093/4861/468/2882/10，真实1095/4866/468/2912/10。此前命令成功不当整轮验收；原sourceclosure/metadata/全部raw与clean0保留在results/full-workspace-failed-root-v1。Root独立核已接受javap固定2Code/10branch、控制3Code/20branch，两class均无handler/jsr；仅更新cfg(test) census tuple及两行说明，不改Reader生产逻辑。

新CLI v9同4707完整closure重建冻结，61213840B/0555/SHA与已真实对照v8完全相同；只接受字节相同，不宣称重新运行58条三方命令。此前focused closure只Reader测试文件不同，runner精确逆变换恢复原SHA；原scope旧label沿用但不能覆盖明确差异。详results/candidate-cli-build-root-v9。fresh workspace v2从头实跑中，后续lint/ownCI/clean仍待。

## 最终源码与 CLI v10（完整 workspace 仍执行中）

Fresh workspace v2 曾真实通过95批/360目标、3455/0/97并clean；最终CI范围Clippy随后发现空数组`first().is_none()`及新增test的Frame赋值风格，失败原记录在results/clippy-failed-root-v2和v3。Root只改等价`prefix.is_empty()`及带scope的Frame字面量，没有新增allow。最终region SHA5313e42faa9d7025d929f203c29e1c3bba271ec6eaf63a70323d2816a6a42604；Reader SHA09e65e4e2af90a7babe41680ca63b5eda2a50f5a1e0ef2a585bfcd235ae34400。

最终lint v4实际fmt/同CI全workspace-all-targets-all-features Clippy/global OpenSpec strict/clean四命令exit0，strict351/0，4707 inputs前后恒同，peak111152270bytes，target不存在。最终正确bin CLI v10三命令exit0，peak361685945bytes、完整4707 inputs前后恒同并clean；冻结61213840B/0555/SHA09f4eac277ee9ddfd60a627f7b132877841f4859bb2c655fc21cb8d25ac197db。较focused-v7两个文件的精确逆变换记录在build receipt，不能冒称同一全部source closure。

Root已用这版字节不同的v10 CLI重新实跑双JDK三方：固定和十case控制各29命令全部成功，各58raw；两次独立checker通过且wrapper真实exit0、stdout/stderr完整保留。固定execution SHA0f98f2cbe7524d7bec0a5ef2cbbe5311e9ff103b361cd522181a17533b37ecac，控制1722c6415c0b9ea1596646d9c35d2d018662ae4947abd7984984b7e722058637。完整原/JADXdefault-none/Jardedefault-all源码原样重编、-Xverify:all，同原stdout/stderr/exit；所有成员身份/physical BCI/UTF8 map/default-all全文和map核验。证据results/final-local-evidence-root-v1，CLI本体留private不入Git。

最终源码fresh workspace v3正在从头运行；以上只接受CLI/lint/postfix，不关闭3.2、自身CI或clean主线任务。

## 3.2 最终本地检查（已验收，9/11）

Root fresh workspace v3完整95批、metadata派生360目标，每个恰好一次，3455/0/97；format/metadata/全部test/clean exit0，5GiB/target1GiB守卫未触发，peak814346343bytes且target不存在。全部4707源码/include/fixture inputs执行前后恒同，并等于最终v10 CLI和lint v4闭包。workspace receipt SHA02311f04c679a697297fe8cd8ce09793b8394ca8e6004d792baf4bea6a609d40；lint receipt SHA c701d54876f0843bb18465d570af332241290e2cfe6d4914eee6a1b180be1d61。

Root local independent checker实际exit0，重构raw counts/目标及命令、核全部streams/source closure/resources/clean；status accepted_local_workspace_lint_only。actualwrapper与完整证据在results/final-workspace-root-v3。3.2和独立栈片3.1接受，不冒称产品Git/live绑定或自身CI已通过；3.3/3.4仍开放。
