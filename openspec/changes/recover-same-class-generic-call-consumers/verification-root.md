# root 阶段验收：同类泛型调用消费位

本片实施未完成。固定 CLI4 已完成候选完整类对照，当前更新后的源码尚未完成全门禁或远端交付。当前 main/origin/main 为 d158989b；该主线 CI 37836010542 四项成功不代表本工作区新实现已通过。本文件按实际证据补记，任务完成状况以 tasks.md 为准。

## 最终本地验收：CLI8/v41，8/9任务完成，等待远端交付

本节优先于历史。root已独立验收[完整候选](results/root-input-audit/final-candidate-acceptance-v41.json)与[最终本地门禁](results/root-input-audit/final-local-gates-v41.json)。定向101/101（1ignored）、graph2/2、fmt/diff、CI同口径strict Clippy；两个固定seed各348targets、3304passed/0failed/93ignored；显式ignored P3三项、constructor一项、bound receiver一项与strict OpenSpec均实际exit0，全部源码前后hash一致。3.3只等提交推送、最新HEAD四job真实JDK25 CI及清理/干净handoff，不提前勾选。

## 最终候选：CLI8/v41 已独立验收，完整第二seed运行中

本节优先于历史。main/origin/main仍d158989b，当前生产与证据尚未提交；tasks7/9。固定CLI8 SHA256 `8770d823c70e7f61749cac836e468c0a991093822d1926822d4769adc1cf7339`，八文件真实source archive `d2367f1cc6a63c0c99804412a7d4f84c7c97eca3807987eab36828e40f97df1d`；完整lineage见 `results/local-gates/candidate-cli-v8.json`。

root亲自执行全部独立verifier：[汇总](results/root-input-audit/final-candidate-acceptance-v41.json)。140主矩阵检查3793文件，Nested4检查34文件与48条实际归属/marker断言，完整144重编/行为、96完整API；完整outer run manifest3836文件重新核hash。44控制逐输入/完整源码/Probe与已审阅CLI6一致，核实际manifest reflection字段，九族bounded refusal、两族positive control。[控制裁定](results/root-input-audit/candidate-v8-control-root-disposition.json)不从marker推断不存在的内部trace。

GC09 field46的44成功行为/26API，SCGB两既有编译拒绝；ctor80行为/36API；raw64行为，字段/方法/类API60/52/60，旧已接受API零回退。CLI8保存真实原/JADX/候选源、独立空classpath/sourcepath编译、新classes-only -Xverify、前后工具/输入hash与stderr；失败成员未删除，临时生成class未保留。各protocol分母保持原口径，不将可靠拒绝计为API恢复。

架构保留现有Signature擦除、同次AST/Code/SSA/InitRecord、method→field边界；有限callee-first +最终incoming/overload +八字段窄overlay原子提交。独立NoBody、具体无TypeVariable的ordinary声明与constructor自身method formal原路由保留。Stop准备/收集进入已有Partial/diagnostic，预算后保留此前完整独立commit，未提交cohort移动回退。普通NotThisRule空record不付构造事实复制费。没有新crate/parser/IRpass/fixpoint/container或跨类推断。共享Signature/cache/物理事实复制是另片债务。

v41定向101/101（另1ignored）、graph2/2、fmt/diff、CI白名单同口径strict Clippy通过；第一完整seed348targets为3304passed/0failed/93ignored，exit0/source8hash unchanged。第二seed与ignored/strict尚在执行，最新HEAD真JDK25四job CI、提交推送/Cargo清理/干净handoff仍待完成，不能提前勾3.2/3.3。v39编译遗漏、v40两个计费失败与所有前轮均原样保留；v41不改旧scope/init断言。

## 最新接续点：v40完整门禁收齐两项计费回归，v41修正验收中

本节优先于历史。main/origin/main仍为d158989b，当前修改未提交；14辅助树detached、干净、为main祖先，无分支占用。v39编译接线遗漏保留；v40定向87/87、graph2/2、fmt/diff、strictCI Clippy及CLI7构建通过。

v40第一seed使用no-fail-fast实际收齐全workspace，只两失败：preserve_local_scope_refusals最后预算停止锚移到新InitRecord复制；class_initializer_candidates普通report与class-source adapter多一AnalysisSteps。两者根因是没有init判定的NotThisRule空record也新计费1。root仅guard prologues.answered()，非constructor没有拥有的初始化事实可复制；原两测试和断言不改。scope预算测试已4/4通过（1 ignored），v41固定新tar与预检运行中，等待最终CLI8和两seed/ignored/strict。

CLI6已独立验收144/46/80/64与44控制（facts v2字段修正，root裁定通过）；CLI7 field46/ctor80也root独立核验通过，完整matrix/raw仍在执行。各旧CLI只证明其真实快照，不能替v41。所有失败、真实脚本、命令/双流/退出码/hash保留。新版必须全对照→root验收→tasks/71局部账本→提交推送→最新HEAD真JDK25四job CI→Cargo清理及干净handoff。未扩大其他功能。

## 最新接续点：CLI6对照全通过，v38完整门禁发现两项回归，局部修正中

本节优先于下方历史。main/origin/main仍为`d158989b`，当前所有修改未提交、tasks1/9。辅助14树再次核对detached、干净、为main祖先，无分支占用；protected副本保留。

v38第一完整seed实际exit101，`ordinary_generic_projection`12/14：既有具体`List<String> bodyOverload`已由ordinary/deferred闭集binding独立证明，新binder事务错误纳入并回退；class Signature准备在事务stop handler外`?`重读导致`BudgetExceeded AttributeBytes limit184 consumed177 requested8`直接API Err。此失败与v37编译失败均保留，第二seed/ignored/strict尚未跑。

root已独立核固定CLI6全部144/46/80/64保存证据：144完整编译/行为、完整API96/144；字段44/46可编译并行为一致、26/44 API，SCGB既有两失败；构造80行为/36API；raw64行为、字段/方法/类API60/52/60，无旧片已接受API回退。额外单JDK/no-debug Object返回context与abstract原形参/explicit this动态实现调用均完整重编、新类-Xverify、method-binder反射与旧CLI一致；`legacy-projection-preservation-cli6-root.json`记录，不算四腿。44控制尚需root对CLI6新facts重新裁定。

facade Luna独占生产修正：无method formal且没有任何TypeVariable的具体Signature不进入binder事务，沿既有ordinary/deferred路径；新caller仍读取实际已发布callee contract。不新增container infer、leaf种类或Signature缓存。新eligibility解析采用现有窄attribute读取并明确计费；prepared class parameters+txn放同Result/catch，失败恢复未提交cohort；候选collection guard/snapshot与VoidBody新计费stop接入正常per-method execution/diagnostic/raw record，不直泄漏API Err。非stop拒绝保留原具体原因；NoBody旧独立证书与此前完整commit不撤回。共享Signature解码/缓存属于另片债务，避免提前全局decode改变malformed Signature仅拒绝头的语义。

修完freeze后root脚本 `/private/tmp/jarde-generic-gates-v39.py` 先87定向/graph/fmt/diff/同CI Clippy、构建新固定CLI7，再同输入重放。full两seed使用`--no-fail-fast`收齐所有剩余失败（测试集合与CI一致，不能隐藏失败），避免只见第一个旧模块失败；全部实际argv/log/exit/hash保留。随后root验收→更新tasks/71局部账本→提交推送main→最新HEAD真JDK25四job CI→Cargo清理与干净handoff。当前Cargo已结束，无编译；生产/测试不全冻结，不得另跑Cargo。磁盘约39GiB可用，20GiB停建线。

## 最新接续点：v38 局部门禁通过，固定CLI6全量验收运行中

本节优先于下方历史。main/origin/main仍为`d158989b`，所有变更尚未提交，tasks1/9；辅助14树detached、干净且为main祖先，无分支占用。当前生产/测试全部冻结，代理禁止Cargo/Git。

v37实际编译E0277失败保留；只修receiver槽引用解引用后v38八文件archive SHA256 `83b1da417d94bcbcfd83d8871a053be93aa31cb2d2e6e3e05f215699db5a5f11`，root定向73/73、图预算/取消2/2、fmt/diff、CI同口径strict Clippy与CLI构建均通过，无warning。原abstract声明与新raw形参/explicit this、Object/Serializable返回上下文的完整重编/GenericDeclaration验证通过；请求末尾预算停止允许此前完整commit，但头/marker原子性与真实Partial/usage/诊断仍断言。

固定CLI6 `/private/tmp/jarde-generic-calls-candidate-v6-cli` SHA256 `c42c2f64df21fd74e22710e6887eac0040850d2e03906a202e051b65af1dc6d8`，metadata `results/local-gates/candidate-cli-v6.json`，仅对应v38。NoBody沿旧独立声明certificate作leaf，仍完整incoming，raw本类直接形参要求AST/slot/SSA匹配；失败保留旧不依赖新caller的声明，caller回原raw形。不造bodyproof/AST，不扩推断机制。上界→raw返回用既有单向assignability，不令T=Object。

root Cargo session55566运行v38两seed/ignored P3、constructor、bound receiver及strict OpenSpec，尚无结果；三个Luna分别重放144到candidate-v6、字段46到gc09-field-23-v7/构造80到gc09-constructor-80-v6、raw64到gc09-raw-64-candidate-v6。原冻结输入/runner/criteria不改，旧CLI5及全部失败保留。CLI5已root独立核144/46/80/64与44控制，但它不证明CLI6。root另重跑旧Object返回补充legacy-direct-return-context-v2，不冒充四腿矩阵。

接续：收齐CLI6实际结果→root独立verifier与控制/架构验收→补tasks/71局部账本→提交推送main→确认最新HEAD真JDK25四job CI→清理共享Cargo与干净handoff。共享Cargo仅root/jobs1/incremental0/testthreads1；磁盘约49GiB可用，20GiB停建线。不能提前勾交付或宣称generic单元追平。

## 最新接续点：CLI5 对照完成，v36 完整门禁失败，修复已确认回归

本节优先于下方历史。main/origin/main仍为 `d158989b`，当前实现未提交，tasks仍1/9。辅助14树detached、干净、为main祖先，无分支占用；受Codex保护副本保留。

v36局部58/58、graph2/2、fmt/diff与CI口径strict Clippy通过；固定CLI5 SHA `010e9f1ecd82a1f2c7c4dcd49010afc787d3980a0194e9fc6a1d12994132dac5` 仅对应v36七文件archive `870589b655d855c7eedf6951c230bd8a69158f98b4ea54f54c14da008d9dc9a2`。144输入完整重编/行为通过，完整API96/144；字段46保留SCGB既有两失败，44行为/26API；构造80行为/36API；raw64行为、字段/方法/类API60/52/60。root已亲自核对CLI5 field/ctor与raw保存证据，matrix/Nested与控制新结果仍需独立核对；不能用CLI5证明后续源码。

`final-gates-v36-index.json` 第一seed实际exit101，generic_throws_projection为11/13，后续seed/ignored/strict未执行。abstract target原有已投影NoBody声明被新事务重新要求body证明而回退；另baseline总usage−1触发请求后期停止，但singleton已完整提交，旧测试误要求其头消失。root还保存 `results/root-input-audit/legacy-direct-return-context-v1`：相同raw Object caller，旧CLI完整编译/新类验证为 `true:1:T`，CLI5为 `true:0:class java.lang.Number`，确认旧API回退，非四腿矩阵。

已授权facade Luna最小修正：上界→raw caller返回描述符采用已有单向assignability；已实际投影NoBody声明作为已验证callee-first leaf，仍完整验incoming；raw本类直接形参receiver必须AST名称/descriptor槽/SSA load一致，拒绝alias/parameterized。不造AST/body证书或新推断机制。预算测试仅修成真实request Partial预算与完整头原子性，root新增Object/Serializable上下文及abstract完整重编/GenericDeclaration测试。生产/测试正在修正，无Cargo运行；修完需新snapshot/CLI6与新全对照/门禁。

接续实测：CLI5 matrix/Nested/field/ctor/raw与44控制均已root独立核对，九族按范围拒绝、两族正控制；非marker源码和完整Probe与CLI4一致，36条marker改变保留具体拒绝。不声称有逐调用AST/SSA trace。修正后v37八文件snapshot已保存，fmt/diff通过，实际编译exit101：新receiver槽位`&u16`误与`u16`比较（E0277）。测试未执行；只修解引用后另冻v38，不覆盖v37失败日志。

下一步：root审阅修正→冻结新源码与CLI→原冻结输入144/46/80/64重放及独立控制核对→两seed/ignored/strict→提交推送main→最新HEAD真JDK25四job CI→Cargo清理与干净handoff。磁盘约49GiB可用，停建线20GiB；共享Cargo仅root/jobs1/incremental0/testthreads1。旧失败证据不覆盖。

## 最新接续点：v36 局部门禁通过，CLI5 全量验收运行中

本节优先于下方历史。主线仍为`d158989b`，当前所有变更尚未提交，任务1/9；14个辅助树detached、干净且为main祖先，只有main分支。

v35实际编译通过，泛型方法17/19，两项失败保留。null-return回归根因是AST保留显式this被incoming错误拒绝，现允许implicit/explicit this并与同次SSA load/initialized-this严格匹配。另一项是合法nongeneric class中的method-local relay已恢复；按GC04将旧混合负例拆成仍拒绝的赋值alias和完整重编/独立双method GenericDeclaration正例，未退回class Signature过滤。Exception上界/不同参数位置正例也通过。

v36七文件实际archive SHA256 `870589b655d855c7eedf6951c230bd8a69158f98b4ea54f54c14da008d9dc9a2`。root定向58/58、图预算与取消2/2、fmt/diff、CI同口径strict Clippy全部通过，无warning。固定CLI5 `/private/tmp/jarde-generic-calls-candidate-v5-cli` SHA256 `010e9f1ecd82a1f2c7c4dcd49010afc787d3980a0194e9fc6a1d12994132dac5`，source identity仅采用v36，metadata=`results/local-gates/candidate-cli-v5.json`，preflight每实际命令/日志/退出码及前后hash见`preflight-gates-v36-index.json`。

完整验收已开始，尚无结果：root共享Cargo session49567运行两seed/ignored P3、constructor、bound receiver及strict OpenSpec；三个Luna分别回放CLI5 matrix144到`candidate-v5`、field46到`gc09-field-23-v6`+ctor80到`gc09-constructor-80-v5`、raw64到`gc09-raw-64-candidate-v5`。沿原runner/criterion/冻结输入/Probe不改，旧CLI4与所有失败证据保留。生产/测试全冻结，代理禁止Cargo/Git。结果完成后仍需root亲自执行新版独立verifier、验收控制、提交推送与最新HEAD真JDK25 CI，最后清理Cargo并写干净交接。

## 最新接续点：v35 修正冻结，定向门禁运行中

本节优先于下方历史。main/origin/main仍为`d158989b`，仅main分支；重新检查14个辅助树全部detached、干净、HEAD是main祖先。主工作区未提交，任务1/9；不能作为干净主线交付。

修正已冻结：原有raw拒绝的具体诊断在rollback及stop cleanup保留；非stop Signature/contract拒绝只影响当前component；raw caller唯一直接返回只采用已staged的同run叶子证书、sole method formal/单普通Class上界及实际descriptor/保留展示wrapper/SSA/完整AST清单，未放宽通用未观测U推断；图及事务重复扫描补归属到实际阶段的计费/取消检查。没有类型名/固定arity白名单、没有新pass/registry。root补两个旧正例完整重编/行为/GenericDeclaration验证，以及Exception上界与不同参数位置正控制。

v34 fmt/diff通过，实际编译exit101（显式扫描改变借用类型后五处多余`.as_ref()`）；测试未执行，失败日志和真实tar/hash保留。只修五处接线后重新freeze v35：七文件archive SHA256 `5cbbcbd6cb65317f52426c84c1609d4de104eb49a177d3127213e137af0a8ce9`。root Cargo session61118正在跑19项泛型方法与原38项回归，再跑图预算/取消、fmt/diff、同CI strict Clippy及CLI build；结果未完成。脚本`/private/tmp/jarde-generic-gates-v34.py v35 preflight`，每实际命令/双流合并log/退出码/运行前后source hash在`results/local-gates`。

CLI4全部144/46/80/64的独立核对仍有效，但只对应v33；CLI5尚未构建或冻结。下一步以v35实际门禁结果修正或冻CLI5→相同冻结输入全对照→两seed/ignored/strict→提交推送main→最新HEAD真JDK25 CI→Cargo清理/干净交接。最新可用约47GiB，target20GiB，20GiB停建线，root独占Cargo/jobs1/incremental0/testthreads1。

## 最新接续点：CLI4 对照完成，v33 完整门禁失败，继续修正

本节优先于下方历史。main/origin/main 仍为 `d158989b`，只有 main 分支；14 个辅助 worktree 全部 detached，无分支占用，受 Codex 保护的副本保留。生产、测试、spec 和证据尚未提交，任务仍 1/9，不能作为干净交付。该 HEAD 的实际 CI37836010542成功，不证明当前未提交实现。

已读 handoff 并接续独立验收。固定 CLI3 的 140 主矩阵与 Nested 4 全部完整重编并通过行为 Probe；完整 API 一致为 92/140，加 Nested 后 96/144。root 已独立核对矩阵、Nested、raw64及44项控制事实；七族待复核控制现按实际 receiver/调用/间接目标/varargs/继承范围裁定为可靠拒绝，拒绝不计API恢复。详见 `candidate-v3-control-root-disposition.json`。

CLI3 的 GC09 发现 ListWrong 两 JDK field List<T>→raw List 回退；v31 完整第一 seed 又发现已有 method-formal constructor 被新事务擦除。两个生产修正已完成：方法自有 formal 的构造器保留原 deferred binding 路由；直接formal字段写含参数化Class segment则不授予新body proof，交回旧候选门。不新增机制、不造cast、不用字段Signature反向证明方法。旧失败结果保留。

v32原构造4/4、ListWrong旧测试通过，另两项旧断言冲突日志保留。更新事务拒绝标记，并将GC02明确支持的ParameterShift宽槽字段写从负例拆成完整重编/GenericDeclaration/marker正例，保留其余8项负控制。v33四组定向38/38、fmt/diff、CI同口径strict Clippy全部通过，无warning，实际源码六文件tar/hash冻结。root初次复制CLI误写bin名称target/debug/jarde，已记录harness错误；Cargo构建本身成功，正确bin为jarde-cli。

新固定CLI `/private/tmp/jarde-generic-calls-candidate-v4-cli` SHA256 `f6fe20c6dfb4be1673ba58e4536b4f8b1da38efe1ee991285b77dbd429717525`，源码archive SHA256 `36768f4551ec2836b9e81a661df3571906bc7acec304a659bb4964b4899e0082`，见`results/local-gates/candidate-cli-v4.json`与`source-snapshot-v33.*`。这些全新目录已完成144/46/80/64对照且root独立核对通过；v33第一完整seed实际exit101，旧generic_method_projection七项失败，后续seed/ignored/strict未执行。另有新图拓扑重复扫描计费缺口正在修正；当前源码已进入CLI4之后的下一轮，不以CLI4替代最新源码验收。

接续：修复旧泛型声明/诊断回退及图计费 → 冻结新源码/CLI并重验 → 全部门禁 → 提交推送main → 最新HEAD真JDK25 CI → Cargo清理和干净主线交接。当前可用空间约48GiB，停建线20GiB；共享Cargo仅root/jobs1/incremental0/testthreads1。不能引用旧CLI3成功证明最新v33实现。

### v33 全门禁失败后的真实接续状态

固定 CLI4 的144完整重编/行为、GC09字段46（44成功+SCGB2既有拒绝）、构造80行为、raw64行为已经完成并由root独立核对；完整API仍96/144，raw字段/方法/类为60/52/60，无旧片已验收API回退。CLI4控制44项的源码、输入jar与Probe转录和CLI3逐字节相同，root重新核对后按同一范围裁定；不能将可靠拒绝当API恢复。

v33第一完整seed实际exit101，`generic_method_projection`为11/18。七项失败包含已支持的method-local null/conditional generic头被事务外incoming误拒，及原erasure/type-use/source-shape具体诊断被投影前raw恢复覆盖；不能修改测试凑通过。其余seed/ignored/strict因首项失败未执行。core Luna已进入修正：保留未投影raw记录的具体拒绝；补单method-formal/单普通Class上界的实际raw caller直接返回证明；补有限事务各重复阶段计费和内层取消检查。尚未freeze或运行下一轮Cargo。

GC10独立审阅确认`generic_call_components`拓扑更新每callee扫描全members并线性查依赖，dense DAG最坏Θ(N³)，该重复processing没有相应charge/内层poll；另facade has_edge多次扫描需明确计费。class_source图调度已复用既有adjacency缩短遍历并补预算/取消测试；facade的has_edge、forward staging、final validation和final incoming重复扫描也在补归属到实际阶段的charge/poll，不新增registry。PhysicalMethodId复制/旧attribute_shells字节记账是另一层共享债务，独立记录，不引入通用计费实体。

当前源码正在下一轮修正，不等于CLI4。Cargo session36407已结束exit101；root各CLI4独立verifier已完成。后续需新源码快照、新固定CLI及完整门禁，仍未提交/推送，任务1/9。

#### 旧声明路径与字段证明边界

GC09 完整回归发现两个 publication 接缝。方法自有 formal 的构造器已有独立 deferred binding 与 constructor declaration 证明，普通调用事务不能用类 formal 的 constructor contract 覆盖该路由；在进入事务前读取唯一 Signature，借用属性 shell 并计费解析，保留旧路由的原 gates。类 formal 构造器继续按本片 InitRecord/调用消费证明处理。

直接 formal 字段写只证明 TypeVariable、其数组或不含参数化实参的物理可赋值源。含参数化 Class segment 的 formal 写入不因可赋值给 raw descriptor 就获得新泛型头许可；交回原候选 gate。否则 ListWrong 的 put(List<String>) 会替换原 raw writer，使已支持 List<T> 字段退化。此限制复用当前 body proof 拒绝，不读取未发布字段 Signature 反证方法、不制造 unchecked cast，不改变已证明 call-result 字段写或完整未读槽的声明许可。

## v31命名修复与candidate-v3验收

candidate-v2完整回放实际114/144完整编译和行为、68/144完整API；30个失败都发生在debug腿，主矩阵14族各两条加Nested两条。root独立核3737结果文件与140输入，确认主矩阵112/140编译、成功者行为通过、66/140完整API；这是失败结果来源核对通过，不是语法验收通过。

根因在ordinary_parameterized_declaration：其余证明已经读projection，最后formal_names仍读已被take的record，fallback生成argN而AST保留x/fail。最小修正读同一projection body proof后才回退record/candidate，不重命名正文、不加机制。root补CallRelay/Nested/Catch真实debug/no-debug整类重编测试；v30首轮10/11，仅root测试把private maybe误写public而断言失败，完整日志保留；修正测试访问级别后v31集成11/11、constructor7/7、strict Clippy通过。v29首次workspace seed失败在旧CallHold/ExceptionHold拒绝断言；改为独立四腿整类重编/构造类binder/行为正例，保持另外7族拒绝和字段独立边界。

candidate-v3 `/private/tmp/jarde-generic-calls-candidate-v3-cli` SHA256 `5b58a8816479c63fbcae8c0d9dcb0c7c7be59e80203551a0c72977585353a1b7` 从v31真实源码快照构建，archive SHA256 `417fb4f9444769b245f74f13f7d155ab60cdf9f80a76dfa297741637b8a8e6d9`，生产与两个测试共五文件构建后hash一致。正在新目录重放全部144/字段46/构造80/raw64，root跑v31完整门禁。尚无最终结果，任务1/9，不提前提交。

旧v29 CLI raw64 root独立核1916文件、710实际子进程及132内部header检查、59工具/8版本probe：行为64/64，字段/方法/类API60/52/60，无新增回退。field v2遗漏ReflectDriver preflight，保留后验说明并重放field v3补齐；这些GC09成功仅针对旧CLI，仍须CLI3确认。root verifier首次错误baseline路径也保留记录，不混为目标失败。

## v29固定候选与完整验收启动

root重读handoff并核实际HEAD/origin `d158989b`及CI37836010542 success；当前新改动未提交。v28唯一测试Limits初始化lint已按struct initializer修正，v29 CI同口径strict Clippy通过，实际argv/env/exit/log与真实源码tar/hash均保存，没有增加allow或白名单。

candidate-v2 CLI `/private/tmp/jarde-generic-calls-candidate-v2-cli` SHA256 `43e41e391250e53982e7d2f948ae016b486b5d62da2b6e76707576455ff77f68` 从v29源码构建，archive SHA256 `81f82e8eb3229239029178ad4dabffb0bebfd8ba76331216c5bd250d9a81ed06`，四文件hash构建后仍一致，无warning。140+Nested4、旧字段46/构造80/raw64候选回放已派发，root正在跑完整v29门禁。结果待完成，不将CLI构建或Clippy通过计作全片验收；任务仍1/9。

## 当前接续审阅与 v25 局部门禁

v26已清所有临时trace并保存真实源码tar/hash。新增真实javac事务负例实际1/1通过：正常nested identity relay及独立leaf均T；测试仅截断expected keys，完整invokes供graph/incoming保留；缺一条或全空均命中validator并使identity/relay整体raw，leaf保T。故障仅cfg(test)、线程隔离、guard恢复，无release新proof机制。身份索引改借已有AST中完整物理身份，避免深拷贝，扫描/索引构造前收费。

fmt check与diff check通过。CI同口径strict Clippy v26报告侧2项、v27 root库5项新lint失败，实际argv/env、退出码、源tar/hash与日志均保存；正在删仅递归转传的无用参数及修Copy/表达式风格，不扩白名单或改证明门。下一轮源码仍待freeze，不把Clippy失败计门禁通过，也尚未构建candidate-v2 CLI。

最新实际结果：同一 `source-snapshot-v25.tar.gz/json` 字节，集成10/10、report16/16、class_source全部52/52通过，无warning；当前change strict再次通过。SCGA v24精确拒绝在local-read全use唯一门，移除该误限后保留当次aload栈值唯一调用消费、exact local slot/store与实际raw声明/inventory，v25旧field/setter T恢复。以下v21～24为失败过程，不覆盖本段。

v25仍含临时诊断，consumer继续补测试专用expected-key缺项的真实事务负例、身份复制前收费并清trace；生产尚未freeze。新固定CLI完整144/GC09/全工作区门禁与新HEAD CI均待执行，任务仍1/9。root重查已提交d158989b的CI37836010542 success；当前target3.6GiB、约69GiB可用。

root 已重读 handoff 并核对实际工作区：HEAD/origin/main 仍为 d158989b，仅 main 分支，14 个辅助树 detached；本片生产、测试、spec 与证据未提交。可用空间约64GiB，共享 target 约3.5GiB。

BoundOverload 的未投影声明已定位到旧普通参数化声明 gate：裸 TypeVariable 可沿原 void 只读证明恢复，`Comparable<T>` 不能仅凭不写参数获证。设计/spec 因此补逐槽真实未读证据，复用 `VoidBody`，不新增候选类型或容器推断。报告 producer 与消费者正在接线；root 新增实际 javac 的混合宽槽测试，区分未读 Comparable、已读 Comparable 和写参控制，尚未执行。

SCGA 的 raw incoming 修复须沿实际 emitted raw local 声明、同次 SSA direct load/definition、精确 descriptor 实参和完整 invoke census 验证，不能跳过 component 外 caller。root 已审阅新 helper 并要求 expected inventory 的精确容量/复制预收费。partial census 的 report 单测不能替代关联事务回退及独立 leaf 验证，后者尚待实施。

旧 candidate-v1 的事后控制裁定与未来 candidate 判据分开保存：允许合法无展示 cast 的 null 调用，也允许完整 API 恢复时计为正例；不覆盖历史输出或统计。当前代码仍在编辑，新快照、完整四腿重放及最终门禁均未执行。

v21 已保存四文件真实字节 `source-snapshot-v21.tar.gz/json`，并运行10项集成入口；编译阶段 exit101，13项错误均来自新报告 helper/candidate 分支误用 reader Budget 方法与本crate StopReason 错误类型。测试没有执行。修正须复用既有 `crate::stop::poll/charge`，不增加错误转换机制；失败日志 `integration-v21.log` 保留。此轮不能证明 Bound/SCGA 或新增未读槽控制通过。

v22报告编译通过，root库仍有三处接线错误（matches模式误写vec构造、两处槽引用未解引用），未执行测试；源tar/hash和日志保存。v23修正后完整10项实际9 passed/1 failed，无warning：Bound/Plain/SameName所有关联重载断言通过，真实未读/已读/写参+long宽槽控制通过，其余旧通过项保持。唯一SCGA在raw incoming helper返回false；v24仅增加窄临时拒绝行号，正在跑该项定位，尚未放宽门。所有局部成功不替代候选四腿/GC09/最终门禁。

root独立复算旧candidate-v1的12条事后裁定并核固定判据快照、输入/输出manifest hash：SameErasureBinder与MultiUseResult各4条为可靠拒绝控制、API仍不同；IncompleteSite4条为完整API恢复正例。无错误，见 `results/root-input-audit/offline-control-disposition-verification-v1.json`。没有运行CLI、javac或JVM，没有改历史统计。

## 输入冻结与 v5 基线

35 个源码族、140 个唯一 `(name, leg, debug)` 输入：32 个新族的 128 个真实 Corretto8u432/OpenJDK23.0.1、debug/no-debug 输入，加三个历史族的 12 个原 jar。root 独立检查 1,072 个冻结文件哈希、144 个 jar class 条目、全部新输入编译命令与原始源/历史 jar 身份，无错误。见 [输入核对](results/root-input-audit/verification.json) 和可重放的同目录 verifier。

修正后 accepted-cli-v4 使用同一冻结输入。root 首次独立检查 3,657 个结果文件、CLI/JADX launcher 与 57 个 libs、当时匹配的执行脚本hash、Probe、逐例源/jar/编译和运行日志。重算泛型反射摘要和行为行，并核对空 classpath/sourcepath、运行仅独立新 classes、`-Xverify:all`，未借原 jar、未删除失败成员、没有残留生成 class。后续candidate标签修改暴露runner_source指向mutable路径而非已保存快照，第二次核对如实报saved-runner-mismatch；其余输入/输出检查仍通过。见 [该次核对](results/root-input-audit/baseline-verification-v4-runner-changed.json)。没有伪造原版本。

现已将同140输入完整重放到 [accepted-cli-v5](results/baseline/accepted-cli-v5/summary.md)，每项编译/Probe/API指标与下表一致。run开始保存的实际脚本快照SHA256为 `6b102b7c16e0bc318d42d3fbd21a32d51bd3bbeeeb72c8f66722808c09f28960`，root重新核对该固定版本及3,657个结果文件全部通过，见 [当前基线核对](results/root-input-audit/baseline-verification.json)。NestedCallArgument 独立四腿现已完成并经 root 核对，任务 1.1 完成；其他八项尚未验收。

| 输出 | 完整类编译 | 已重编者的行为 Probe | 完整泛型 API 一致 | API 不同 |
| --- | ---: | ---: | ---: | ---: |
| 原源码 | 140/140 | 140/140 | 基准 | — |
| JADX | 130/140 | 130/130 | 126 | 4 |
| Jarde 已交付主线 | 72/140 | 72/72 | 20 | 52 |

144 个 Jarde 物理类 class-source 输出均退出 0，140 个主目标的声明头非空；这些数据没有计为源码正确性。编译失败的 flavor 没有 Probe，也没有反射一致结论。每个成功编译输出的 Probe 使用真正 GenericDeclaration 与 formal index、数组/参数化类型递归和 bounds；method identity 包含擦除返回类型，constructor identity 包含擦除参数，defpackage 差异按所属类型名归一。

JADX 的十个真实编译失败是 UnknownIncoming 四腿、BoundOverload 四腿、MethodHandleUse 两腿。Jarde 的 68 个失败分布在 17 族，每族四腿，实际诊断及其他 API 差异见 [逐族基线](../../evidence/same-class-generic-call-consumers-2026-10-09/results/baseline/accepted-cli-v4/family-summary.md)。这提供后续候选的逐族比较入口，不代表整个 generic 单元已经追平。

前几轮 harness 的公共类文件名/路径碰撞已保留，不能计为 JADX/Jarde 的语法缺陷。修正脚本先通过 CallRelay、BridgeUnknown、BoundOverload 及 intersection binder/合法 null 控制的少量完整重编，再执行正式 v4。CompatibleIntersectionBinder 使用外部 Probe 的真实 NumberRunnable=17 marker 验证身份，同时保留 null 控制；没有修改目标输入 jar。

## 局部实现验证，不计任务 1.2 完成

报告侧定向测试 v5 为 7 passed/0 failed，无编译 warning，日志在 [report-tests-v5.log](results/local-gates/report-tests-v5.log)。范围是 exact invoke key/错误与多义 origin 拒绝、source cast 子节点与物理 origin 保留、foreign-method 同 BCI 不误改、返回/If/Throw/普通 catch 的窄消费事实、资源/finally/未知节点拒绝、预算及取消传播。facade 的 SSA 槽位与全部使用 join、关联事务、封闭 overload 及完整类验收仍待完成，不能用这些局部测试勾任务 1.2。

前三次接口编译失败均保留于 local-gates：CallTarget 无 Ord、slice/HashMap 调用类型不一致、Option<Box<Expr>> 的 iterator 函数签名不一致。修正后再跑 v5，不覆盖历史日志。

整包 `cargo check -p jarde` 的 v1/v2 分别因接口接线/借用错误失败，v3 编译通过但有三项 unused warning；随后已删去未使用的字段 Signature 证明函数，并移除未使用参数，待稳定后再验。GC07 定向单测 v1 因测试模块 API/import 错误未编译，修正后的 [v2](results/local-gates/overload-tests-v2.log) 9/9 通过且无 warning，包括不同 binder `U extends T` 不冒充负适用关系、已知/未知 JDK 上转型、物理目标、宽参数位置和预算取消。

集成 v1 因错误 scope 枚举未编译，v2 五项因 class bytes 与 PlainJar policy 不匹配而失败；这是 harness 接线失败，不能计为实际反编译拒绝。已对齐既有 SnapshotAll/SingleClass 协议，下一轮仍待生产稳定后执行，日志保留于 local-gates。

root 已发现并要求修正两处真实生产接缝：invoke operand 通常是 aload 的栈写值，必须用同次 Code/SSA 的精确 local-read 回到入口 formal 或初始化后的 this，不能直接把栈值当 Entry；调用参数的 descriptor 展示 cast 会挡住 nested/null 泛型参数位，必须区分并保留真实 checkcast，只在精确调用点和已证源类型下调整展示 cast。GC07 body 替换也必须先比对未改 AST 的原 emission，再提交获证 projected emission，不能要求新旧含 cast 正文相等。上述修正仍在实施，不将局部测试算作完整候选验收。

最新报告侧 [v8](results/local-gates/report-tests-v8.log) 编译成功，11/11定向测试通过且无warning，补充覆盖Number/Object[]展示wrapper、nested/null参数位、真实checkcast不移除、child presentation/origin和预算取消。v6/v7新增测试编译失败（slice匹配及重叠mutable borrow）均保留，修正后才执行v8。

声明参数类型/名字误比修正后，[direct trace v2](results/local-gates/callrelay-trace-v2.log) 编译无warning，CallRelay、ArrayRelay、NumberBoundRelay、WideRelay、DeepRelay五族的预期泛型头断言通过，首次停在EmptySink；该测试仍退出101，ReverseDeclarationRelay和MethodShadow尚未执行，不能报告整个direct测试通过。另跑 [integration v5其它四项](results/local-gates/integration-v5-other-cases.log)，unknown incoming保持通过，BoundOverload、NestedCallArgument、CatchCallMarker仍失败（1/4通过），无编译warning。上述仅为局部声明/文本断言，没有候选完整重编或JVM Probe结论。EmptySink当前AST需求已启用，但member census仍只沿旧own-ref需求开启，root要求贯通单成员完整证明的事实采集，后续实际trace验证。

最新 [integration v6](results/local-gates/integration-v6.log) 在需求census修正后编译无warning，仍为1/5通过。root检查外层提交门只判断旧deferred fields/methods，EmptySink虽有新输入与AST/SSA需求仍未进入关联提交；要求纳入新输入。NestedCallArgument的first/second/relay均正文证书Some且staged projected=true，但之后组回退，须定位提交或AST重发射拒绝，不能再报告正文证明失败。CatchCallMarker停在maybe正文证书None，BoundOverload停在relay overloadproof=None；临时trace已记录实际markers。下一轮尚未执行，全部实施任务仍未验收。

修正harness后的集成v3/v4均为1/5通过、4/5失败，不是候选验收成功；通过项为unknown incoming关联回退且独立成员保留。最新 [CallRelay trace v1](results/local-gates/callrelay-trace-v1.log) 确认callee-first顺序正确，identity真实正文证书Some（formal arg1、Parameter slot1），但project_method_signature返回Settled且projected=false，组在进入relay之前回退。root源审阅发现ordinary声明校验把method_descriptor返回的类型字符串与proof参数名比较；正在修正并补方法binder正文证书路径。该trace没有记录底层拒绝marker，不能声称已从日志确认具体error code。临时trace最终必须移除。


## 后续集成快照与停止审阅

`integration-v7.log`、`integration-v8.log` 均为2/5通过，无编译warning。direct测试中的八族头断言和unknown incoming回退通过；NestedCallArgument、CatchCallMarker、BoundOverload仍失败。它们是局部声明/文本测试，未做候选完整类重编。

v8的精确证据：Nested三个成员的staged正文与泛型头均成立，旧正文核对未重建既有Signature marker而拒绝重发射；Catch中的异常对象构造receiver实际来自new@4经dup@7到ctor@8，不能直接比较new的SSA值；BoundOverload仍在overload site证明拒绝，已增加窄诊断等待下一轮。实现agent已按这些事实修改，但本记录不把未测试快照计为通过。

root另审出预算原子性缺口：只在单个component开始恢复原始记录，不能覆盖索引、依赖构建或尚未访问组的停止。新增集成测试对analysis/output两个预算维度扫过正常CallRelay的多个截断点，要求已恢复的两正文不留下仅一个泛型头。owned输入移动恢复、失败metadata整组准备后安装的修正已进入v9快照；root随后补充两个互不依赖调用组的停止场景，该补充尚待下一轮执行。收费和取消审阅仍需继续。

最新 `integration-v9.log` 实际4/6通过，无编译warning：direct八族、unknown incoming、NestedCallArgument以及单链预算截断通过。Catch的maybe正文和泛型头已证明，构造器调用仍在overload proof被拒；Bound明确在body-consumer查询返回None，尚未进入重载适用算法。query缺少独立Call表达式语句消费事实，补充只允许精确Call来源的channel并要求独立已证调用BCI；构造器receiver仍需独立InitRecord，不能借旧directconstructor候选的全局EH拒绝。修正中，不能将4/6算作候选整类或任务完成。

候选汇总入口已准备，但尚未运行；预期144个输入行、148份物理class-source输出。全反射差异独立记录，GC-06按原spec核对构造参数、callee API及全部行为，字段独立判定；尚不能证明拒绝边界的控制标为needs-root-review，不能直接计为验收通过。

## v10～v12 与旧片新基线

报告侧 `report-tests-v9.log` 为12/12通过，无warning，新增只接收精确Call表达式语句的消费channel、普通catch scope及错target拒绝。v10因facade参数名接线错误未编译，并出现一个unused参数warning；修正后v11编译无warning，实际3/6。BoundOverload的Number转换与PlainUpperBoundOverload的无冗余转换均已通过，overload集成测试停在最后的SameNameOverload头断言。新增两个独立调用组的预算测试揭示旧deferred publication先改一个leaf，再停止而跳过新事务，留下T/Object半组。

`integration-v12.log`编译无warning，4/6：direct、unknown incoming、Nested以及单链/双独立链预算截断通过。外层deferred循环结束后、字段发布前恢复saved原始记录，覆盖跳过事务的stop。Catch的callee与constructor正文证书均已成立，但构造器声明仍因旧route要求direct candidate而走入禁止<init>的ordinary方法speller；需复用constructor声明门并以真实body/init证书独立路由，不能放宽旧EH候选。SameName还受旧arity-only binding门阻断；新增字符串常量写仅有真实CP String/唯一SSA写消费/实际String字段的来源，不等于泛型字段类型证明。正在补closed overload待证状态与局部staging，最终全incoming重验保持。

GC09 raw-64已验收CLI的新基线重放完成，root另核对479个文件哈希、工具/CLI和执行runner快照、实际javac/runtime命令及逐例API与行为。原/baseline完整编译与验证运行64/64，baseline字段/方法/类API分别60/52/60，行为64/64；JADX完整编译60/64，失败为InstanceRawLocal四腿。见 [root核对](results/root-input-audit/raw-64-baseline-verification.json)。首轮root检查器遗漏实际-cp参数而报错，修正后才接受；目标runner未改。这不是候选回归，GC09任务仍未完成。

## 尚需兑现的架构与交付

关联方法必须保存投影前头与正文，按 callee-first 使用实际已独立获证的 staged contract，最后对全部 incoming 调用重验，再整组提交或回退；不以未证明 Signature 充当 callee 类型。方法 binder 的同名遮蔽不能复用只允许 class binder 的旧字段比较函数；预算/取消错误不能被 `unwrap_or(false)` 吞成普通拒绝。构造器通过真实初始化记录和完整 AST/SSA 单独证明，不能移除旧 EH guard 或伪造 Parameter 候选。

封闭 overload 由独立 Luna helper 与 facade 接缝并行实施。root 保存了真实 JDK8/23 Number/String/Comparable/Object 声明头，见 [平台头证据](results/root-overload-platform-headers/manifest.json)。既有引用 widening 表只给部分正关系，其 false 不能冒充所有负适用关系；Java8 源语义不能借 JDK23 的新增接口。只能投影已证明无运行时检查的上转型，不猜 Object→T。

直接调用结果参数位 NestedCallArgument 独立四腿已核对：115 个文件、jar 内 class、实际 runner/CLI/JADX 57 libs/JDK 工具哈希均一致；原/JADX 各 4/4 完整编译与 Probe 通过，每腿 12 项声明归属和 marker 检查，baseline 四腿均有真实 Object→T 编译诊断。核对空 classpath/sourcepath 与仅新 classes 的运行命令，不修改旧 140 输入或混入其统计。见 [root 补充核对](results/root-input-audit/nested-baseline-verification.json)。23字段回归中有9族的既有协议只有固定源码而无保存jar，本片沿原协议在临时目录用相同固定源重建，明确记录源/编译参数及jar class hash，不冒称历史jar。其他14字段族、80构造、64raw及140本片输入保持原jar复用，回归计划见 [regression-replay-plan.md](results/regression-replay-plan.md)。

候选四腿、23 字段族/80 构造输入/64 raw receiver 回归、完整 fmt/clippy/两固定 seed/ignored 门禁、strict OpenSpec、提交推送及最新 HEAD 真 JDK25 CI 均未完成。本片尚未提交，也没有新 branch/worktree。共享 Cargo 留待集成验证复用，验收后清理；2026-10-09最新核对约72GiB可用、target约2.8GiB（后续构建会增长）。

## v13 局部快照验收

`results/local-gates/integration-v13.log` 已执行结束，6 passed / 0 failed，无 warning。SameNameOverload 的封闭重载延后投影与 CatchCallMarker 的独立 InitRecord 构造声明路由通过局部断言；direct 八族、unknown incoming、Nested、单链及双独立链预算截断均通过。此结果不是冻结四腿完整类重编、行为或反射验收，任务仍 1/9。实现 agent 随后报告一行旧 deferred 条件收窄，尚需在清理诊断后的快照重跑。当前未构建最终候选 CLI、未重放候选矩阵、未提交生产改动。

## v14 诊断清理与预算审计

`report-tests-v14.log` 12/12、`overload-tests-v14.log` 10/10、`integration-v14.log` 6/6通过且无warning。生产临时 fixture 硬编码 trace/eprintln 已移除；`cargo fmt --all -- --check` 与 `git diff --check` 通过。初次fmt发现一行新清理后空白行包含空格，已清理并重新格式化；未冒充首次格式化成功。源码身份保存在 `results/local-gates/source-snapshot-v14-formatted.json`。

审计确认 `GenericCallProjectionInput.source_record` 与事务stage的完整 `ClassSourceMethod::clone` 包含 `RecoveryReport`、文本、注解等副本；现有数量级preflight不覆盖这些完整副本的计费。此问题属于本片GC10，必须修复，任务不提前勾选。正在分析窄projection备份或既有计费路径，禁止为此新增通用serde计费框架。候选v1将用于完整矩阵定位，不是最终接受CLI。

候选探索CLI已冻结为 `/private/tmp/jarde-generic-calls-candidate-v1-cli`，SHA256 `b613c0482c8aa56bdf479321abbcc6cadd12c500dc9eec6e2efd8a62daa00900`，从已完成v14局部门禁源码构建，无warning。固定CLI与源码快照索引在 `results/local-gates/candidate-cli-v1.json`。fixture agent已获授权重放现有144输入完整候选矩阵，不修改判据；结果尚未产生。GC10窄projection staging在该CLI之外实施中，后续必须构建新固定CLI并重验，不能把candidate-v1当最终实现。

## candidate-v1 完整探索与root独立核对

固定v14 CLI完成140主矩阵与Nested4：144输入、148实际物理输出，全候选完整编译和行为通过。root `candidate-v1-matrix-verification.json`逐3793文件/140case/工具/实际命令/Probe/反射转录核对通过：原140/140，JADX130/140，候选140/140；候选完整反射84/140。Nested root另核34文件、4唯一JDK/debug组合、真实JDK工具与48个owner/parameter/return/marker断言，与冻结原Probe完全一致。此核对不是GC01～GC10全部通过。

反射审阅确认：BoundOverload number行为4/4，但pick(Comparable<T>)丢泛型；VoidDirect是原与baseline全API一致的既有positive，candidate两头退为Object，属于明确回归。GC09 field23在SCGA reflection首断言停，constructor80整体编译行为72→80但RawNewHold已证ctor/field与PeerNewHold二参头退化（Peer的整体API本来不同，不能用聚合计数遮住成员退化）；raw64同scope指标未降，尚待root重核。原runner失败与所有diff保留。GC04/GC08不同binder及多use拒绝控制须按可靠来源/原子拒绝证据review，不能只看完整APIhash或任意缩feature要求。

root测试新增constructor-only/SCGA/VoidDirect的已证头保留及Bound Comparable<T>断言，现9项，新断言未跑。overlay实施源在agent修改中，与候选CLI相互独立；下一轮需冻结后新编号门禁与新CLI，任务仍1/9，当前未提交。

## 接续核对：raw64候选与v15冻结

root以固定candidate-v1 CLI重新独立核对raw64：842条实际命令、59项工具hash、64行实际runtime转录，字段/方法/类API为60/52/60，行为64/64；对accepted 3f75基线无新增API回退。核对脚本及JSON位于results/root-input-audit/verify-raw-replay.py、raw-64-candidate-verification-v1.json。runner未保存成功命令stderr或运行前JDK二进制hash，JSON明确保留该证据限制；其文件索引是本次root核对生成，不伪称原runner manifest。该结论只覆盖固定旧CLI，不替当前overlay源码验收。

v15保存了实际源字节tar与hash（results/local-gates/source-snapshot-v15.tar.gz/json），随后9项integration在编译阶段exit101：两处projection_state缺Some包装、两个unused变量。未执行测试，不计通过；日志integration-v15.log保留，下一轮只修接线后另存新日志。

## v16 局部验收与控制裁定

v16编译接线已修复，9项integration实际6 passed/3 failed：constructor-only原API保留通过；BoundOverload的Comparable<T>头、VoidDirect sink/relay、SCGA字段/setter仍失败。迁移遗留staged contract及两个旧record投影方法dead-code警告未清理，不能计完整门禁。源字节快照与失败日志分别为source-snapshot-v16.tar.gz/json、integration-v16.log。

root按既定acceptance裁定SameErasureBinder/MultiUseResult为可靠拒绝控制，保留完整API未恢复事实；IncompleteSite由四腿真实javap确认是合法conditional单调用正例，不能计为缺失inventory控制。具体裁定在results/candidate/root-control-disposition-v1.md，不覆盖既有统计，也不放宽VoidDirect/Bound/SCGA的正例要求。GC08还需独立partial census负例，任务仍1/9。

## v19 定向验收

v17临时trace宏作用域导致编译失败，v18trace包名条件错误未记录实际拒绝，日志和真实源快照均保留。v19完整9项integration仍6/9，三个精确gate为BoundOverload未获证声明投影、VoidDirect最终cast/removal计划不一致、SCGA nongeneric main不在component入边门。不能因graph已接就宣称GC07完成。

v19 report16/16通过（新增完整census、缺项/全漏/重复/未知opcode/不完整Code、budget/cancel及零AST命中），class_source::全部52/52通过（包含GC07与overlay exact-second-charge原子回滚，以及detach后读取旧generic void状态），无warning。两项均对应source-snapshot-v19真实字节，facade census接线尚未完成，不能计整组缺失inventory控制已接受。root还独立核IncompleteSite四腿jar与javap转录hash、每条唯一BCI6调用，记录root-input-audit/incomplete-site-census-verification-v1.json。目标change strict通过，日志openspec-current-change-v1.log；全工作区最终门禁尚未运行。

## v20 完整9项局部测试

root运行integration-v20实际7 passed/2 failed，无warning。VoidDirect的旧body证书成功分支确实漏登记空/已验证cast与presentation计划，补登记后保留最终全入边和重载重验，sink(T)/relay(T)恢复断言通过。constructor-only保持通过。Bound全组Comparable+relay仍停未获证投影gate，SCGA仍为nongeneric main的raw incoming未验证；未计GC07/GC09完成。源字节source-snapshot-v20.tar.gz/json保存，含临时trace，最终交付前须清理并另冻CLI重放。
