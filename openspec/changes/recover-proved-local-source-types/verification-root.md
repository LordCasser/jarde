# Root verification — recover-proved-local-source-types

当前 **6/8**。类型生产补丁已于关闭后的 main `5c2c06f1ec8c3ff0560f2c2d89059ee7d6d06b02` 重新应用；整 CF12 未接受。独立 pop 来源片已6/6。新增永久真实 class 测试实际 **9/0/0**；14条完整本地验证已实际通过，645/0/2；新CLI已冻结。完整源码对照已接受；自身 CI 与最终主线交付待完成。

## 第一次应用与架构边界

两轮真实 focused 测试均为 2/1：null/String 全物理来源及公开 atomic Stop 通过；char 正文正确，但缺 append 后 pop82/92/105。实际 patch、永久测试和失败 raw 已保存，随后恢复生产源码。不能将这次失败记为任务2.1/2.2/2.3通过。

类型恢复只扩展既有局部声明类型决策：准确 char producer 种子加全部写的有限兼容证明；null 首写且所有非 null 写为同一准确 Reference。copy/phi、混合写与槽复用保持原边界。root 已完整阅读 saved typed patch/test 及周边声明、调用和选择器逻辑；现有 invocation_argument 按准确 descriptor 转换 primitive，足以保留原 append(I) 重载，无需新重载机制，仍须完整类重编运行及新 javap 证明。

预算 observer 私有稿引入四个 test-only 实体、thread-local 与生产 cfg 分支，root 审查后未应用。后续用临时真实 trace 取得准确证明前缀，保存后移除；永久测试沿用真实 IR 与公开 Stop/API，不保留观察框架。

## 重新应用与真实边界

[focused-root-v3](results/focused-root-v3/execution.json) 实际3/0/0，char 锚 pop82/92/105 全部具有准确物理来源。随后临时追踪真实恢复调用，保存 [proof-trace-root-v1](results/proof-trace-root-v1/execution.json) 两项实际测试与全部原始输出。CHAR 的 write29 前 AnalysisSteps=321，下一次 charge=1；NULL 首写1 的 bulk proof 前=89、uses=6。临时 trace 已按 pretrace SHA 恢复产品；实际临时 diff 和恢复 SHA 单独保留，无永久 observer、thread-local 或 cfg 分支。

Luna 私有测试稿经 root 修正三点后应用：补齐 helper 中漏掉的全物理来源断言；预算必须绑定测出的同一 CHAR/NULL class，不能误用其他 boundary method；no-debug 槽冲突的 number 真实名为 local1。最终 [focused-root-v4](results/focused-root-v4/execution.json) 实际9/0/0。两份 debug/no-debug class 中4类 char 种子、3类 int 反例、同型 String/null 和3类宽 Object 反例共22个方法profile均 Structured、全部物理 BCI 覆盖、default/all正文及map相同。

possibleSlotReuse 两版保留准确 Fallback：同一 local2 的 write10=int、write17=Object；BCI61 读取未声明局部被拒绝。缺失物理来源准确为 `[0,1,4,6,9,16,22,28,31,32,34]`，永久测试约束这一现有失败，不声称已解决拆槽或整个边界类已可运行。两条证明内预算以 limit321/94 分别实际 Stop@29/1，维度AnalysisSteps、written0、无content/binding/text/map。公开预取消测试仅证明恢复边界原子性，不能当成 token 在证明内部被设置的证据；新增循环的 charge/poll 静态核读已完成。

## 已独立接受的 post-pop / pretyped 基线

使用冻结 pop CLI SHA251d3d4e77773a6783df6a10564ad8cf82e67f1405ccd424558f1a0ee5e6bcde，在没有类型改动时实跑 **48命令/6用例/8输入class实例/16整类报告/38方法profile**。root 独立 verifier 实际退出0；仅九处已证 pop 成为完整 Stmt derived，删除这些准确新增后，旧 segments、primary、span、方法身份及完整正文逐字相同。原程序 fresh stdout/stderr 与历史 original/JADX raw 相同，旧 Jarde 失败分类从 raw 重新核对。全部输入、class、SDK/helper、工具和原始输出重新核 SHA。资源差异仅允许非递减 AnalysisSteps 与 elapsed，其他计数保持。

完整上游控制矩阵为 pinned JDK23、Java8 source/target。第一轮 JDK8 在 ORIGINAL FallThrough.check 加载原 JADX assertion helper 时失败：helper class55，JDK8仅接受52；18条已执行命令和原始异常均保存。未改 helper、去掉 check 或采用 stub。两份自包含类型锚与独立边界可以另行双 JDK8/23 验证，不能声称全部上游控制已在 JDK8 运行。

接受记录 [pretyped-baseline-root-v1/acceptance.json](results/pretyped-baseline-root-v1/acceptance.json) SHA **bc9be073af6fd98817d1bae6e541f630efea714cc2856a3981ff2bde76c7df06**；execution SHA **93dc97cf213358e94d56bfe52846db79ffe765a9fd187c5e5bad3ed56433bc08**。接受记录的绝对路径仍指原保留目录 `/private/tmp/jarde-cf12-post-pop-baseline-root-v4`，历史路径/hash不回写。235份原字节归档均实际核同原，copy manifest SHA00170c1ee4b6d5975bb76f945294f24f9c6708ba9cf4cf340c6c356fa0aae6e3；新增解释 README 单列。原始 char mismatch、NoDefault/conditional 编译失败仍保留，不能由这份基线接受类型产品。

## 接下来的验收

类型 patch 已按保存字节应用，未覆盖 pop 修复。保留两锚全部 physical BCI，尤其 TestSwitch 原 fallback 未覆盖的 pop105；其他控制与本次独立 post-pop 基线对照。实际核一般 int、范围外 literal、混合/未知引用、copy/phi/复用与准确 append(I) 重载边界，失败记录准确层级。完成 proof 内预算 Stop、完整类重编运行、新 CLI 冻结、自身产品 CI 与全 worktrees clean 后才能关闭本片。

root 已全文读 replay v2、v3，均有静态缺陷，未执行候选。v3仍有生成class相对路径口径不一致、raw数组与item对象比较、javap switch表数字误作opcode BCI及未准确处理既有边界拒绝等问题；Luna保留旧稿修到v5，root全文审查v3与全部后续diff，准确修raw/schema/class-set/runtime及失败profile赋值问题；[review-root-v2](results/replay-draft-review-root-v2.md)和私有稿均归档。该私有稿后来经root实际执行并修正输入schema及guard globals绑定；执行失败未覆盖，实际接受见下文。

[local-validation-acceptance-root-v1](results/local-validation-acceptance-root-v1.json) 重新核28条raw及精确测试摘要；第一次 p3_patterns 为84/1/0，准确失败是旧成本61新增两个已计费局部写后变63。按该原class的 istore_0@3/astore_1@9 更新成本与说明、保留tight正文比较，生产无修订。原失败v1未覆盖，重跑v2全部14cmd实际通过：645/0/2，target峰值701852711bytes。CLI e6978d74d0935621e73db7d2640fc5a545e4ddfd4c090027ebb84930b5419403，0555；build SHA18a9750cdeb0d8bc45786f5d0766cbb256f1c8a87b1daf350965e910535a6413。

## 完整源码实际接受

[complete-source-root-v4](results/complete-source-root-v4/execution.json) 实际99条命令，由 [root verifier v2](results/verify-local-source-types-replay-root-v2.py) 独立接受：两锚 Java8/23 × default/all 共8条 runtime 与真实 original/JADX 完全同 stdout/stderr/exit；两锚全部物理BCI、10组CF12 default/all body/map对照通过，其他8个控制profile及非目标方法的正文/map/质量/拒绝保持独立post-pop基线。全部class/check/Inner与真实SDK保持。整CF12仍未接受：条件fallthrough编译失败和Labels数字投影未由本片解决。实际verifier invocation在results/complete-source-verifier-invocation-root-v5，返回0；完整raw在该片专属目录。

原边界类两JDK source运行通过，候选完整类因准确slot-conflict fallback缺return而无法编译，未进入runtime。初次JADX边界编译还含Runner包名错配；[boundary-jadx-package-correction-root-v1](results/boundary-jadx-package-correction-root-v1/execution.json) 仅给原Runner添加生成类相同package、完整JADX source原字节不变，两JDK仍唯一报107行 `System.out.println(z ? null : null)` 的 println(char[])/println(String) 重载歧义。该观察准确登记JADX缺陷；Jarde的Object局部和准确println(Object)转换在永久方法测试中保留，不能从组合类失败声称完整运行通过。

准确int重载 [int-overload-root-v1](results/int-overload-root-v1/acceptance.json) 实际9命令独立通过：char producer与46常量局部恢复后，两profile完整类编译/运行仍输出46\n，fresh javap准确append(I)、无append(C)，完整物理来源和default/all body/map相同。private verifier误将原符号工具路径/relative classpath当成resolved absolute，root按原record及SHA修正路径口径；第二次仅javap file-date中文在LC_ALL=C中变问号，root核raw diff仅该一行，其他每行/class SHA均相同。两次失败raw保留，最终root v2 verifier返回0。

收集器两处独立问题与一次root执行错误都已保留：原boundary tools keys实际javac8/javac23；runpy返回mapping不是执行函数globals，OUT未生效，root中止后将146条raw按byte-exact relocation manifest归档；另一次cargo clean与guard target扫描并行导致FileNotFoundError，未接受，顺序纠正后全矩阵重跑。没有修改冻结产品来通过这些脚本检查。cargo clean实际3042files/669.3MiB，target已移除。

## 自身 CI 的旧断言与完整 char 对照

产品 `8b899778118ca51ecd7b02d59c95465028dd9847` 已提交推送，但自身 CI `38084415019` 的 stable 第一 seed 在 `tests/p3_meeting.rs::a_position_performs_its_own_widening` 失败，第二 seed 被跳过，MSRV/fuzz/supply 成功；不能接受任务3.2。失败原始日志在 [ci-repair-evidence-root-v1](results/ci-repair-evidence-root-v1/jarde-typed-ci-failure-capture-root-v1/execution.json)。旧断言要求 `int local1 = arg0;`，新全写 char 证明准确输出 `char local1 = arg0;`，由方法 `(C)I` 的 int 返回位置隐式扩宽。生产代码无需修订；断言现在同时约束 char 参数、char 局部、int 返回和无多余转换，准确 `pass(int)` 转换、真实 i2l/i2b 及 pop2 控制保持。

追加独立 [focused repair](results/ci-repair-evidence-root-v1/jarde-typed-ci-repair-root-v2/execution.json) 实际 fmt 与6/0/0通过。完整 Meet 类全部12成员保留，[char 全范围接受](results/ci-repair-evidence-root-v1/jarde-meet-char-root-v2/acceptance-root-v1.json) 实际19命令/38raw，两JDK8/23 × 原源码/JADX/Jarde default/all 共8腿，每腿逐值检查0..65535，输出均 `chars=65536,sum=2147450880`，正文/全部方法身份/既有来源map在default/all相同。root独立verifier退出0。首次CLI参数误写debug而非evidence、首次verifier过度比较debug报告计数的失败均保留，未修改任何生成源码来通过。

追加 repair inputs 与原17/10/50冻结构建闭包分开核，不回写历史meta/build或重建未改的生产CLI。新CI adapter沿用完整冻结核验，每seed额外核六项p3_meeting准确测试名、原class/源码/测试的Git与live SHA。任务仍6/8，修复提交自身CI待实测接受。本地一次全workspace编译触发1GiB target守卫，真实中止/清理保留；小批全量验证正在执行，不能把中止算测试通过。

## 后续两个旧断言修复

只读审计与冻结CLI实际整类报告又定位两项相同的旧期待：RequiredConversions 的 declared/assigned 全写满足 char 证明，旧 int 声明断言改为 char；field/int 调用/拼接所需转换保持。NullThenBuilder 原历史 Object 局部现在由 null 后准确 StringBuilder 构造写证明，移出旧 unknown-null 拒绝列表，新增完整类只改局部声明类型的正例，并核三成员、essential/all 正文一致及 all 全来源身份。历史class与baseline原字节保留，其余七个拒绝边界保持。生产源码及原17/10/50冻结闭包未改。

[focused repair v7](results/ci-repair-evidence-root-v2/jarde-typed-ci-repair-root-v7/execution.json) 实际fmt与三组测试通过：Meeting6、ReferenceSlots8、RequiredConversions8，共22/0/0，无守卫中止。新增测试先错误比较essential空map与all完整map，后两次误访问private字段，实际失败v3/v4/v5均保留；v6通过后补齐三成员数量约束，最终v7再次通过。

RequiredConversions 完整13成员与field、NullThenBuilder 完整三成员的源/JADX/Jarde default/all双JDK实际运行均通过：前者逐值检查65536char及五个相关方法，后者 true/false 两值结果7/6。Null的原Java不可得，原执行腿使用原canonical class，不声称恢复原源码；独立verifier复核尚未完成。全工作区小批验证已保留前49个成功命令，从此前Null旧断言失败的第49号命令开始补完，其结果不能预先视为通过。新提交自身CI及最终clean仍待完成，任务6/8不变。

root审查CI adapter v4：局部repair三个Cargo Running headers来自stderr，准确名字/计数来自stdout；CI combined日志继续严格核目标header与双seed。九项追加repair pins独立核Git/live与实际repair，不混入历史冻结闭包。v3私稿与v4及审查改动保留，不把静态适配当CI验收。

## P5 实际计费基线与独立整类复核

产品修复2c5ee0d6已提交推送。前一修复f5e36ba43的自身CI38086421940实际在旧NullThenBuilder拒绝断言失败，raw SHA bbcf546fa98252989c522b6ccfe0427ede15e599532e4367661e41be2b368d4f，第二seed跳过；失败原样归档。2c5ee0d6修了两个声明期待，但全workspace小批随后在P5旧AnalysisSteps pins发现另一冲突，不能接受该片3.2。

P5既有ignored recorder实跑六个固定archive形状，逐项复核只有AnalysisSteps增加16/22/14/43/14/3；两个187成员逐方法路径均从19458变19570，增量112与六行之和一致。原因是decide_types新增全物理write扫描及有限producer/null证明，必须准确计费，所有fixture/read/IR/source/outcome/delivery维度不变。本次只更新六行与两arm的AnalysisSteps常量及说明，不增budget，不改生产。root独立复核原Git before、actual recorder raw、各九维计数、所有sum与live after；小批v4第62号命令常规P5五项通过/一ignored，完整文本/outcome/content/execution三路径比较及两个成本断言都通过。[P5接受范围](results/ci-repair-evidence-root-v3/jarde-typed-p5-billing-repin-root-v1/acceptance-root-v1.json)仅限此片成本，94批全量尚在执行。

两个完整类运行现在已独立接受：[RequiredConversions](results/ci-repair-evidence-root-v3/jarde-required-char-root-v1/acceptance-root-v1.json)与[NullThenBuilder](results/ci-repair-evidence-root-v3/jarde-null-builder-runtime-root-v1/acceptance-root-v1.json)各19命令/38raw/8运行腿，root实核完整sources、精确argv、13/3成员及field、源map spans与所有physical owners、live工具SHA和独立BLAKE3。Required逐值调用五个相关方法，其余成员完整保留并编译；Null原Java不可得，original仍仅准确class oracle。Luna verifier先误读identity.name为对象而非raw数组，root修正后实际退出0，原失败保留。冻结CLI与17/10/50闭包不变。

CI adapter v4的source/local预验收真实退出0，核77分类条目/28构建raw及三组22测试/9追加pins与Git/live相同；它不接受CI。root生成预验收脚本的newline syntax失败原样保留后v5修正。任务仍6/8，待包含P5修复的精确自身双seed CI及clean主线。

## 完整本地工作区与交付前检查

[workspace-complete-root-v1](results/workspace-complete-root-v1/README.md) 保留94条实际命令、356个metadata目标及全部188条原始stream，含前几轮已成功命令的原始carry路径。root独立核编译artifact身份、unfiltered执行header/摘要、86条Git与live source pins：单固定seed全工作区 **3408/0/97**，target峰值799452287bytes，接受范围仅本地，不替代自身双seed CI。最终cargo clean实际1665files/664.2MiB；后续只读负例observer另清348files/172.2MiB，生产未改。

产品43dffc9b806dc2b69102e50f52ee78a50850c6d0已实际提交推送；提交时main/origin相同、15 worktrees全clean/14辅助detached祖先、本地与真实远端仅main、全部无target，free66015203328bytes。新增归档尚待交付，不能把这个历史checkpoint当成新增文档的最终clean。此前2c提交的CI38087514715被root主动取消；完整API状态cancelled保留，不计为失败。当前43产品自身CI38087978610第一seed已成功，第二seed仍运行，暂不接受任务3.2或3.3。

## 精确产品 CI 最终接受

[typed-ci-product-root-v7](results/typed-ci-product-root-v7/README.md) 保存精确43dffc9b产品run38087978610：4jobs/52steps全部成功，两seed各356摘要、3408/0/97；root实际v7独立verifier退出0，准确typed9名及全部86Git/live pins通过。v6只因combined Cargo header空白边界误吞后续摘要而失败，原失败与原capture保留，v7复用严格单binary解析，不改变产品或测试。任务3.2在本次strict实际通过后关闭，3.3待新增全部归档提交推送及全worktree clean实核。

## 干净主线交付

全部授权证据随9d6d51570af0f71e55ffb6fd502aee062e035a2e提交推送后，root实际audit v4接受main/origin相同、15 worktrees全clean、14辅助detached main祖先、本地及真实远端仅main、无target/fuzz target、CLI原SHA/0555及5GiB余量。v3因检查发生于尚未提交的staging而失败，原记录保留。验收记录在results/clean-delivery-checkpoint-root-v3。任务8/8；关闭文档提交推送后再实际private audit确认最终HEAD，不借文档CI。
