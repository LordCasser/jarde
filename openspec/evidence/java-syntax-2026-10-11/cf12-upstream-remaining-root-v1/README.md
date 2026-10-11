# CF12 完整清单剩余五类：真实对照

本批补齐 CF12 十个上游测试类中此前未回放的五类。使用固定 JADX 2fb1b16386941660fda07e9017285aec40fcb37f 的原源码与既有官方 SDK/runtime jars，JDK23新编译 IntegrationTest、五测试及 observer，原本源码检查和生成源码检查不关闭。direct实际 3 commands，JUnit为3成功/2失败；失败原始 stdout/stderr、XML、捕获 class 与生成源码完整保留。此失败批次只接受实际观测，不冒称 upstream 全通过。

| 上游测试类 | JADX JUnit | JADX 整类重编/运行对原 | Jarde default/all 整类重编/运行对原 |
|---|---|---|---|
| TestSwitch2 | 通过 | 通过/一致 | 编译失败，switch arms overlap/uncovered |
| TestSwitch3 | 通过 | 通过/一致 | 两 profile 均通过/一致 |
| TestSwitch4 | 失败：2234≠1234 | 编译成功，check运行失败 | 两 profile 均通过/一致 |
| TestSwitchSimple | 通过 | 通过/一致 | 两 profile 均通过/一致 |
| TestSwitchWithFallThroughCase2 | 失败：Code duplicated warning | 通过/check与68值矩阵一致 | 编译失败，outer if ArmsDoNotMeet@0 |

render 用产品6476b56c357443ef17318891b12142f509977234冻结CLI，SHA39d5699c1665b16e0c4a46934f0e773aeedf392f5bc60869a7b2762680dd10f1；15命令实际javap与default/all。完整重编运行32命令/15对照行，所有原类都经-Xverify:all运行成功；源码不改名、不补stub，生成类集合与原class一致才进入对照。助手及官方jar中没有target classes，生成编译classpath不包含原class。无check的两类用明确差分输入观察：Simple12值保留stdout，Switch2全部32初始布尔状态、9种float、12actions轮转与8步序列，672状态行；不是穷举输入证明。三种有check的类先执行原上游check，失败则保留实际异常，不伪造后续矩阵执行。

root独立verifier v2 实际exit0，复核manifest/raw hashes、JUnit失败名、原class输入、完整source/compiler/runtime记录、class集合与classpath边界，确认6个Jarde匹配profile、4个编译失败profile、4个JADX整类一致/1个运行失败；13个default/all方法map对、238个UTF-8半开区间也一致且边界有效。v1及补核原始执行/编译classpath后的v2均保留。接受JSON严格写cf12_complete=false。新增五类观测使上游十类都有实际完整class对照；旧五类要按各自已冻结产品/验收记录解释，不能把跨版本结果并成一个已通过产品。

TestSwitch4无需新增Jarde机制：当前真实输出保留数组下标的旧off值和后缀增量顺序，实跑已证明本批输入正确。Luna只读稿的“尚未运行/优先验证副作用”反映其审计时点，此后的root实跑结果以上表为准。JADX重复代码warning同样不能当作语义错误：root完整check和68输入已同原。

后续大颗粒结构缺口锁定TestSwitch2含早返回的case共享continuation，以及TestSwitchWithFallThroughCase2外层if内switch后续语句。现有fallback确认未恢复；具体内部证明器是否接受switch子形状仍待受控诊断，源码推论不冒称已验证唯一根因。不把这些结构改动混入常量名gate。conditional自身CI仍独立等待，正式产品代码没有被本批分析改动。5GiB free/target1GiB/私有输出1GiB一秒进程组守卫均无中止，本批未运行Cargo，target保持不存在。

架构候选见[两项结构缺口只读稿](jarde-cf12-two-remaining-structure-plan-luna-v1.md)：FTCase2先验证child switch next175是否由外层arm继续到197，优先复用既有continue_switch_arm的完整incoming/ownership及visited delta校验；不预设必须新增helper。Switch2的现有线性switch_forward_join排除直接case target164且不接内部return分支，需验证有限DAG共同continuation，不可放宽canonical边闭合。root更正稿中一个指令标注：外层iload在BCI4，ifle本身在BCI5；外层branch block起点是0。稿中的内部返回/责任归因仍是源码推论，实际fallback只证明方法未恢复，诊断先行。
