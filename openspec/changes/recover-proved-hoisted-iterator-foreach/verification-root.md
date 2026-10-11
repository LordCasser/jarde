# 提升iterator声明的内层foreach验收（规划）

当前3/9。proposal/design/specs/tasks均通过OpenSpec strict，apply已实际读取ready/1-9。root接受固定旧基线、sameIR-v3首拒及private-v3声明owner静态审阅，仅对应1.1；新CLI完整对照、后续owner/Phi/effect诊断、控制基线和实现均未执行。

准确范围：inner42在iterator_declaration_before_init；outer13在body_first_not_initialized_declaration。现有Object iteratorElement头保留cast/body binding，泛型和外层Assign独立。不将同列表找不到空声明推断成其余gate已通过，不跨frame删除祖先声明。

已派Luna私有准备1.2诊断patch：须记录同Program的准确LocalVariable/RegionPath/placement/at_region、init/test/next/Phi/reads/effects；临时continue必须在最终投影前强制拒绝，不能返回虚假删除索引。root可在当前产品自身CI等待期间按守卫/原字节恢复约束运行临时诊断，永久候选待当前交付后应用，诊断恢复源码并clean，不冒称实现。资源5GiB free/target1GiB一秒守卫、默认Rust stack、root独占工具链/Git/shared。整个CF11/71/612计数不变。

调度修订：前轮CI stable实际41分钟；当前产品完整local验证和push已接受，下一片临时诊断/完整class只读观察可以并行推进，最终原字节恢复/clean后仍用原product/live闭包验收自身CI。此修订不授权把未验收候选混入当前产品。

Task1.2中当前CLI wholeclass基线部分已actual完成并独立接受：29cmd/58raw，原2/JADX4/Jarde4完整class runtime腿全部同八case oracle，constructor3与test45 physical BCI/source map完整，4727闭包同finalbuild/live、target不存在。准确execution SHA4f3322219d09505fe9b81f5753d1c4d0996145581b0e81538d40f39b0d73d22a，archive见results/current-baseline-root-v1；声明owner/后续proof真实诊断尚待，因此1.2整体仍未勾，仍1/9。

1.2完整接受：current CLI wholeclass29cmd/58raw已接受；owner-only临时诊断v1和扩展Phi事实v3各actual test1/0/0+clean两命令，root acceptor-v3均actualexit0。v1 peak347717482B、v3 peak320749435B，4727源码按字节恢复且target不存在。inner42三LocalVariable均slot5/index0、Elevated owner[1] visible/emitted/at_region一致；真实首拒iterator_value_and_phi_use_ownership。V37三条None@97对应Phi22三V37输入、uses空/replacement12，header12=[V37,Itself]、uses空/replacement37。更后的next/cast/effect门未执行。详results/same-ir-diagnostic-root-v1和-v3。

诊断prep-v2词法作用域错误由root静态审阅拦截，未应用/编译；v3仅移logger到同一作用域。acceptor-v1正常SystemExit被捕获由root静态拦截，未跑；acceptor-v2实跑两次因Rust Debug结构闭合regex错误拒绝，完整failure raw保留，v3精确补闭合并核重复文本为同一六use tuple后接受。不改真实证据或生产门。

root接受Phi ownership review-v2：仅检查实际含精确source ValueId的Phi及occurrence multiset；v1对无关同block Phi也要求inert属过度拒绝，已明确纠正。four artifacts已相应修订，同一个existing iterable proof消费replacement事实而非新增liveness机制。written-gap只对新祖先路径收紧为空声明，旧同列表时序债务单独记录；inline/separate cast证书深度不混称。

控制基线v1在工具链前因Runner/manifest不同被拒；v2准确清单后在javac启动前被5GiB守卫拒（4430757888B），没有Rust/Java编译。恢复资源后v3实际43cmd/86raw，collector/checker exit0：Positive四isolated生成class可运行，combined/Refusals八compile失败/no runtime；这是观察，root独立接受仍待，不先勾1.3。

1.3已root独立actual接受：acceptor-v3 exit0，43cmd/86raw与全部input/tool/live4727源闭包、guard/default-all/20method maps核准。原两JDK15行oracle SHA d785d6e818f4c66780362f43cb524dd412ffd57d8d5b63f9e1e15c3136132240；Positive四同源isolated完整generated class同原，旁侧Refusal仍原源码，不能称生成Refusal运行。JADX四完整class在written-gap-iterator-throws-before-write返回1/原0，trace同values.iterator()；其异常求值重排不作为本仓标准。Refusal四profile各缺24物理BCI，八compile失败/no runtime原样保留。3/9仅基线与架构范围接受，永久生产候选/新的Phi gate与更后门尚未运行。
