# Root verification

当前9/10；CompoundIf实现和整类行为已接受，最终源码重放及拒绝/Stop回归正在收尾，自身产品CI/干净交付尚未接受。以下保留实际历史失败与修订记录。

本地JADX HEAD2fb1b16386941660fda07e9017285aec40fcb37f，root实际git show该对象与工作树IfRegionMaker源码逐字节相同；原始源和Apache2 LICENSE/NOTICE作为分析资料保留于results/architecture-root-v1。生产拟议实现只参考算法，不复制Java代码。此前末位e为root笔误，v2/v3原笔记保留及更正。

同一CLI完整回放175cmd的真实Switch2在双JDK/default/all四组compile1保留，实际join trace和javap pins绑定。root对固定物理graph的16种条件组合，独立比对图执行与 `(t98 && !t111) || (t118 && t124)` 的测试顺序、出口结果，16/16相同；这是架构层固定图推导，不是新生产算法验收。每个条件树叶子一处，payload131和return136不作为测试；记录analysis.json SHA849e5f5b473a4b720c63a05b14ef384b473ddd6cf5b234290fbaf22c24abe547。

四artifact已真实OpenSpec strict通过、apply ready。Luna基于region SHA34dcf89ccee47c33e0b6e08089ae53567b30ce0f73b50e76b13aec1ddf040025/build SHA768735cd1cf5d7694d2a1d7b28030475b0bc0864e1d18d9a6350e12c999b1de1准备private候选，不改workspace/分支/工具链；root负责串行review/apply/build/完整对照。

v2私有候选经root审查拒绝，未apply/未Rust测试：展开后化简偏离design3，只支持单payload，integration Stop回滚和内部依赖/赋值/prefixfallback有缺口。见results/candidate-review-root-v1。Luna另v3修订，保持既定spec范围，当前仍2/10。

受控完整类旧候选基线实际20cmd：双JDK原65/65，JADX64/65（唯一NaN错误，不能互补float关系），Jarde default/all四compile1/未runtime；verifier rootv3实际exit0接受30项有限观察，原observer命名及验证器失败保留。root独立javap实际exit0。详results/complete-runtime-baseline-root-v1；这些不是v3新机制通过，task3.1仍未勾选。

## 2026-10-11 修订候选的首轮真实验证

region v5和Builder v4已由root串行审查、应用并格式化；private producer未运行工具链。root actual focused-v1因新增测试的三个类型错误编译exit101，target已清理；最小私有编译修正后focused-v2 lib编译成功但真实region测试15/2/0，两失败均保留。public origin候选已严格审查后应用，浮点测试允许等价反转并换臂，不固定变量名或Not的根位置；此轮public目标尚未执行。

真实两次诊断确认two-payload条件的内部dependency及pure全部闭合，四测试归约和canonical闭合成功，integration Ok(None)。根源码审阅发现existing region_at到达frame boundary时next=None，而新增integration要求Some(join)，已派发最小契约修正；同时Switch2测试必须分清block入口与真实terminal branch BCI，所有调用/Stop断言均需使用正确physical BCI。不得降低边/副作用证明门槛来掩盖失败。诊断执行后region.rs按原字节恢复SHAef9e4e230ea47455455afb5bcc920869c65e102dc0757d90d3b83aa5f9b5278d，所有actual run均clean。原始raw/sourcepins/记录见results/focused-validation-root-v1。task仍2/10，未接受新机制、Switch2、CF12或自身CI。

## 真实 frame 修正后的 region/public 验证

新增有限 void-return 的 v6 局部测试误用 boundary136，而实际 caller 在 next136≠boundary164 时才调用 helper；其 predicate 与 caller 不兼容，真实 whole Switch2 仍16/1。失败保留 focused-validation-root-v2。v7 拒绝 next 等于 boundary，沿原 frame 调 walker，局部测试改用实际 boundary164。真实 focused/public 分别17/0/0和6/0/0；复合条件两 arm、四 tests、唯一 return owner 与 scope/case负向检查通过。记录 focused-validation-root-v3；2.1/2.2据此勾选。2.3后续完整Switch2 map、2.4负向全覆盖、2.5阶段Stop、3.x整类及CI均尚未接受。新CLI SHA465282ec5c11111eb5a76fde3dfc052f058441a3c9776864c8706cf2f5d2cf87，0555，19 pins实际相同，targetclean。不能据此宣称整类运行通过。

## 新CLI真实65路径整类行为

完整28cmd及双JDK八runtime腿已独立接受：原/Jarde两profile全部65/65，JADX两JDK唯一NaN误判。完整目标和Runner class集合、source/工具/raw pins、default/all完整正文/maps等28checks通过。v1 verifier误用README hash和字面symlink路径的exit1保留，v2严格修正后root真实exit0；3.1勾选。准确来源另见public permanent tests，不冒称verifier内独立javap已核。详complete-runtime-candidate-root-v1。十个CF12回放正在执行，3.2/最终产品CI尚未接受。

## 新复合条件CLI的完整CF12组合验收

同CLI SHA465282ec5c11111eb5a76fde3dfc052f058441a3c9776864c8706cf2f5d2cf87 实际十类179cmd/20profiles/80sides；v8独立核验实际exit0，Switch2和FT2四profile均整类compile/runtime0、class集合和原stdout/stderr准确相同，20组default/all正文、identity与map恒同，8组目标完整BCI集合与独立javap相等78/90（重复来源检查数115/130不当uniqueBCI数）。旧oracle与新CLI实际build分别绑定，19源码pins/raw/build/clean核验。9编译及3原runtime环境不可用仍为JDK8上游major55；唯一非环境runtime失败为JADX Switch4，Jarde保持原正确。详compound/results/cf12-complete-candidate-root-v1。组合结构与行为已接受，尚需负向/阶段Stop、完整适用回归lint及自身CI/提交clean，未宣称整体CF12/71-unit完成。Switch2状态矩阵不替代65路径独立副作用/异常/NaN覆盖。

### 新增阶段Stop与负向观察（尚未关闭任务）

Builder私有v3 root实际unit1/0：prepare52/51与emitter52→68/67，Stop均AnalysisSteps/written0@59且状态恢复；public整目标7/1失败为测试写死written0，实际Stop limit516/usage516/written633@7。精确pin v4仅test修改已应用，尚未实跑。取消发生在prepare完成后、进入emitter前，不宣称异步中途注入。Clippy按CI范围实际exit101，只有测试pool字段简写三处，守卫未触发且targetclean；修订待验收。

内部iinc/istore独立写回实际21cmd：双JDK original各8/8、JADX各4/8、Jarde default/all完整类compile1，目标整体ExplanationOnly。iinc16、istore21及36/39个唯一BCI须由独立verifier闭合；不冒称此例已到达新增Builder guard，也不记恢复通过。完整原始记录和独立后续assignment共享条件债务见compound results/independent-effect-boundary-root-v1；当前机制的纯条件约束保持。

## 最终源码 CLI v2 的独立运行验收

最终CLI SHA378c410083dccb6d7d0ac08dcd7c0bcc519b57e29dd4a5d073b314779c06fe3d、60819568bytes/0555，实际5条构建命令全exit0、124 source/include/class pins前后恒同、cargo clean后target不存在。复合条件5测试和公开8测试真实通过。完整受控类28命令、双JDK八腿再执行，原/Jarde default/all仍65/65，调用次数/顺序、异常和NaN同原；JADX每腿唯一NaN差异不当Jarde失败。root读取原始流后运行独立v3 verifier，29 checks全部一致，status为independent_runtime_observations_accepted。collector的总体incomplete_or_mismatching_observations包含JADX已知差异，不是Jarde成功判据。证据results/complete-runtime-final-cli-v2，CLI二进制仅存private tmp。十上游类与四拒绝class最终来源验收尚待，不借历史CLI结果关闭交付。

同一最终CLI已实际完成十个上游类179命令/20profiles/80sides，独立v9 verifier exit0、Switch2_restored和FT2_positive均true，20组default/all完整正文/member/maps恒同，Switch2/FT2准确78/90 unique physical BCI闭合。9编译、3原运行仍为上游JDK8辅助major55环境不可用；JADX Switch4唯一非环境错误2234仍记录，Jarde同原1234。结果cf12_complete保持false。真实原始证据与v9输出保存results/cf12-complete-final-cli-v2，不能将历史CLI v1作为最终验收替代。

## 最终拒绝边界与短路合并对照

四个冻结合法class实际同CLI执行12命令（4 javap+8 default/all class-source），完整8成员包括constructor、目标所有unique physical BCIs、class/method owner BLAKE3/length、非空来源及UTF8 slice/range均独立核验，default/all正文和maps恒同。cycle/irreducible/no-repeat/clone各真实unit拒绝与状态恢复此前Java350/0/0已通过；本次render只接受拒绝后的整类来源观察，没有调用目标方法，也不要求某个质量标签来伪装恢复。v2 verifier实际exit1因要求collector没有提供的冗余逐fixture.expected_owner；v3仅删除该镜像字段检查，冻结顶层oracle和实际API owner验证全保留，root重建差异并实际exit0。真实失败raw、12命令记录与两个verifier都在results/negative-full-class-final-cli-v2。

NullThenBuilder冻结class和旧baseline不变；永久test仅在原精确local-type替换之外增加唯一nested-if→short-circuit-AND期望，保留全类/3成员/maps和七拒绝controls。最终CLI实际19命令、双JDK原/default/all/JADX八腿全部true=7/false=6一致；原Java源码不可得，original使用canonical class作oracle。独立v2 verifier核全类唯一改动、精确argv/工具/build124pins/raw/生成class集合及八运行腿接受，private v1的JADX package路径错误先经root审查纠正。永久Rust回归尚由fresh workspace v3验收。

## Fresh完整workspace v4真实验收

同一最终生产源码从头运行95批，Cargo metadata完整359目标各一次，3444/0/97；完整原始source/fixture pins恒同，5GiB free/target1GiB守卫未触发，cargo clean后target不存在。独立workspace validator实际exit0，无failures，逐项重建全部批次/targets/counts/raw/pins。Java351/0/0（含真实nested Switch certificate None）及JVM243/0/0全库，全部既有conditional/loop/switch/value/control-flow回归通过。Stop详细reason/dimension/location/usage仍以此前真实专项记录为准，未改预算或取消契约。四合法负例全类来源、内部iinc/istore拒绝、clone/case/frame路径边界、external/duplicate/non-Normal edge rows及所有trial恢复检查完整闭合。

候选歧义的边界有独立限制：完整incoming ownership、empty fallthrough和cycle约束使两个直接case target同时成为成功candidate不可达；拒绝重复/外部入边和真实cycle/nested/unknown terminals按既有proof测试覆盖。不制造不可达list mock，不宣称命中最终multi-candidate list match。完整行为通过不等于所有未覆盖形状新增恢复：当前纯条件及直线tail约束保持。

P5 actual recorder的前五shape各+23 AnalysisSteps、deep0、两arms+115，其余八维及完整报告/文本断言不变；corpus指纹只P2两文件摘要/一个长度共三字段，2068文件分母和维度/分类不变。完整历史失败在p5-billing-reconciliation-root-v1。最终Clippy、自身产品CI、提交推送和主线干净交付仍待，不能将本地通过当CI。证据在compound/results/final-validation-root-v1/full-workspace-final-v4及独立validation目录。

Final fmt及CI同范围workspace/all-targets/all-features Clippy实际exit0，29项既有allowlist不增减，-D warnings保持；完整WS v4 source inventory与当前pins相同，守卫未触发，clean exit0且target不存在。证据compound/results/final-validation-root-v1/final-clippy-v4。自身产品CI及最后主线干净交付仍待；本地通过不替代CI。
