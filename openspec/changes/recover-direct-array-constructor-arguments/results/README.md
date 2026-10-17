# 直接数组构造参数：证据索引

架构、最小组合实现及完整源码/JADX/Jarde 对照已由root接受；当前进行双fresh native workspace及产品自身CI交付，tasks3.3尚未完成。固定71/612总数不变。

| 目录 | 实际范围 |
| --- | --- |
| tf3-original-inputs-root-v1 | 未改写 JADX TestFinally3 源码、原始 inner/outer 两 profile、真实 harness jar/CP pin、4命令8raw；javac23 source/target8 |
| tf3-original-baseline-root-v2 | 冻结7287 CLI 的12cells与JADX1.5.6四cells，16命令32raw；100物理方法配置来源审查，helper各缺BCI4是旧基线事实 |
| tf3-baseline-matrix-root-v1 | 38命令76raw，8次原oracle/64case；Jarde20编译各helper missing-return、JADX6编译各inputStream definite-assignment错误；无rebuilt runtime；root与Luna独立复核 |
| tf3-ssa-probe-root-v1 | 原始两profile真实SSA和构造首拒，1诊断测试及exact clean；临时probe已恢复 |
| direct-array-inputs-root-v3 | 自有完整诊断类的双javac输入，含有意String拒绝及无关AssertionError拒绝，不能删方法作为正类 |
| direct-array-ssa-probe-root-v1 | 双javac各12方法的真实array facts/uses/final值/source集合/元素sites及首拒，1诊断测试及exact clean |
| architecture-review-root-v1.json | root重新打开实际raw并确认组合边界，未将静态推论或probe视为产品 |
| complete-array-originals-root-v1 | 独立完整正类及外部隔离反射driver，8命令16raw，双javac/双JVM四次运行56cases全同固定14行oracle |
| complete-array-baseline-root-v1 | 26命令52raw，旧Jarde八次完整编译红且无runtime；JADX四次重编/八次运行112cases符合原oracle |

各capture.tar.gz包含manifest，capture-review记录逐成员复开数。归档内execution保留原运行绝对路径，manifest可核归档本身。旧失败不拼成成功。源/JDK/JADX pin 是所述运行的来源，不证明未来重建。driver无法观察原始空close方法的调用次数或顺序；owner harness为major55，owner只javac23重编，不能声称javac8接受。

准备脚本的前置失败仍留在/private/tmp：baseline抓取v1错误classes路径、独立collector早期Path/dict与manifest缺bytes假设、数组compile-v1/v2工具字典路径；这些发生在工具命令前，不冒称已执行编译。首次归档因父results目录不存在失败，建目录后成功的归档逐项复开；不覆盖工具执行的原始失败。

独立完整正类的私有准备v1未执行，root发现driver Object/int比较错误，以及caller内赋值造成额外array consumer；v2在编译前修正：数值显式解包、观测引用移至PairSink构造器中记录，从而外层数组仍唯一由constructor消费。v1未冒称编译或通过。原始诊断类保持原样，不与新正类混淆。后续统一证明也应接受已有完整证据的String副作用元素，不把旧诊断注释的人工特判作为产品边界。


## 最终实现与实际验收

最终CLI 62815008B / SHA e01c6014bd1d1d220d72400beeb711791221aa2aa56e60eed0ba52f8714777c0，与真实完整矩阵所用focused-v3 CLI字节恒同。未新增solver/pass/公开carrier；移除被通用数组证明覆盖的String窄特判，保持原argument_dependencies语义，借用已证明sibling来源而不吸收所有权，并按receiver SSA身份选择同owner构造调用。

| 目录 | root接受范围 |
| --- | --- |
| tf3-original-final-root-v1 | 16命令32raw，100方法配置全物理身份/BCI/UTF8；default/all正文及映射同，原helper缺BCI4已闭合 |
| tf3-final-matrix-root-v1 | 98命令196raw，Jarde20次原样整类编译/40次隔离运行320cases；88非目标方法配置正文/map不变；JADX6编译均有两处inputStream未初始化错误，no-runtime |
| complete-array-final-root-v1 | 42命令84raw，88方法全BCI，Jarde8重编/16运行224cases，JADX4重编/8运行112cases，原oracle另56cases；负长度、长度/元素异常、对象身份、多个实参和构造顺序均核对 |
| complete-array-independent-review-root-v1 | Luna只读独立复开上述raw、输入与88方法映射；all-evidence额外工作计费如实保留，不要求usage相等 |
| same-owner-originals-root-v1 / same-owner-final-root-v1 | 真实双javac与固定身份/三次构造oracle；最终Jarde8重编/16运行16case、JADX4重编/8运行8case，Jarde四配置均class Complete |
| reverse-originals-root-v2 / reverse-final-root-v1 | 合法SWAP原字节码双JVM8次验证；最终69命令138raw，Jarde自然8编译/16运行正确，反序8次编译失败/no-runtime；JADX反序4编译成功但8运行全部trace21而非原12。Jarde自然类级仍Partial/CLI4，见独立债务 |
| order-probe-root-v1 | 16命令32raw，另一后置维度重排经双JVM8次验证，trace123/132且实参值同；自然构造接受、变体实际首拒array BCI5，证明无需新增排序机制；不宣称拒绝正文可重编 |
| control-provenance-baseline-root-v2 | 8命令16raw，旧7287/新e01 × 双javac × default/all，确认既有StringBuilder head53 map缺口与all-evidence presented不一致；单列债务，未混本实现 |
| final-gates-root-v4 | 12命令24raw，4公共正例/停止、181reader、1指纹再生、5指纹检查，共191通过出现/0失败/1忽略；fmt、CI同款Clippy、MSRV1.88全workspace/all-targets check及CLI构建通过，最终exact clean；只允许指纹变化。Java库461/0来自生产代码相同的前序命令，不把其整体失败收据当作成功 |

原2275指纹项逐项未变，仅新增30预期输入；26新class的reader真实增量78Code/4handler/6branch，先实测失败后更新人口tuple至1227/5446/602/3502/10。详见final-gates-root-v4/fingerprint-review。

## 失败与私有准备记录

focused-root-v1-failed保留三个漏更新调用点编译错误；focused-root-v2-failed保留库457/2的reader缺口；focused-root-v3-failed保留461库通过后公共测试误用region-level fallbacks。final-gates-root-v1-failed保留无Java AssertionError名称的真实fallback正文；v2/v3-failed保留head53映射缺口与presented不能保证最终正文含该构造的事实。最终测试保留完整Fallback质量和真实BCI49/52解释，数组来源只核断言分支前head<49；完整正例来源仍全BCI核验，没有宣称修复通用fallback来源。

reverse-originals-root-v1-failed保留首次将CLI4误作采集失败的收据；control-provenance-baseline-root-v1-failed保留默认未请求news导致StopIteration的第一次raw，该collector未及时保存命令row，不能当作完整命令收据，v2重新完整采集。Luna实现、reader和receiver私有补丁、JADX源码算法对照、对抗性审查初步意见与撤回结论均单独归档；静态草稿不当作运行通过。

独立架构债务见[separate-architecture-debt.md](separate-architecture-debt.md)。本片不增加71/612完成数；未关闭持续goal。

## 重复遍历修正与交付中检查点

首产品550b7ff922e32fa4ce5d4326d8c7d22bd8fa30a3已推送。其fresh seed1第65批实际P5失败：flat-mixed IrItems2497→2501、完整direct/shared35654→35687，其余计量维度及正文不变；收据保留在workspace-seed1-root-v1-failed，63批通过不能拼成完整回归成功，seed2未启动。该产品CI39060504198由root主动取消，四job真实日志及精确旧产品来源见cancelled-own-ci-550b7-root-v1；不冒称CI独立复现本地失败或成功。

Luna私有补丁经root审查，仅init.rs的12行变动：保留选中调用时已读取的operands，在既有producer循环统计物理参数次数，删除两次重复遍历；原P5表值/预算/语义断言全部未改。最终候选CLI62815152B / SHA605fdd1b2b0cc077bb9440a02b7f995ca33dded37b7527cc306f65a77bdf4c0c。final-gates-root-v5实际11命令22raw、470通过出现/0失败/1忽略（461库、P5 bulk5、四公共门），fmt/当前Clippy/MSRV1.88及构建通过，3037来源前后恒同，exact clean。后续完整native和新产品自身CI仍待完成，tasks3.3保持未勾。

output-replay-root-v1实际40命令80raw，覆盖原始TestFinally3、完整通用数组、同owner、反序、后置维度重排、fallback来源对照及完整char/String家族。所有完整源码字节及非usage报告恒同已接受矩阵；root另核实际全部差异仅usage内ir_items/elapsed_millis，不能把脚本递归忽略usage当作任意计量变化许可。本次未重新执行JDK矩阵；此前真实原样整类重编/隔离运行证据按相同源码字节复用。

补充char-string-originals-root-v1为8命令16raw、4原运行28cases；char-string-matrix-root-v1为42命令84raw、40物理方法配置全BCI/UTF8/身份，Jarde8原样整类重编16运行112cases，JADX4编译8运行56cases。覆盖String([C)、String([CII)、存储身份、普通CharSink、两个元素生产者异常及次序；实际矩阵使用e01 CLI，经上述精确输出重放绑定最终605 CLI。

p5-final-root-v1-collector-failed的首命令四target实际58通过/0失败/4忽略，但通用单target收集器要求恰好一个测试binary，因获得四个而停止并clean；不能把它当八target全门成功。改为八条单target命令重新执行，未更改产品或测试。

p5-final-root-v2-feature-missing前四target仍58/0/4，第五definition_read_reuse因未启用test-support实际运行0测试，收集器如实拒绝并clean；v3全部使用--all-features重新执行，未降低“必须实际运行测试”的门。output-replay-independent-review-root-v1独立逐值重算1255差异，只有整数ir_items696处与elapsed_millis559处；新旧160raw、精确prior请求/退出状态、源码恒同均接受。v5-independent-review-root-v1核11命令22raw与470/0/1逐行结果；该审查较早时的“replay未运行”描述仅属于其当时静态审查，实际执行及后续独立复核以后两个replay目录为准。
