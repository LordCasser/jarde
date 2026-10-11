# 提升iterator声明的内层foreach验收

当前9/9：1.1–1.3基线/诊断和2.1最小实现已接受。私有v1的strict-prefix列表误判被静态拦截；v2按当前准确空声明行区分两条路径，root补充owner anchor的Iterator类型核验。实际focused-v2默认栈通过新增API 2、既有foreach 5/6/4和Java lib360；8cmd/16raw、4729输入闭包前后/live一致、peak654226085B、finally clean。root独立accept实际exit0且Luna只读复核一致。2.2永久反例已应用并实跑接受，完整class3.1已root实际接受；最终4732输入的fresh full workspace/fmt/同CI Clippy/global strict/final CLI字节绑定已root实际接受；自身CI及交付尚待。

准确范围：inner42在iterator_declaration_before_init；outer13在body_first_not_initialized_declaration。现有Object iteratorElement头保留cast/body binding，泛型和外层Assign独立。不将同列表找不到空声明推断成其余gate已通过，不跨frame删除祖先声明。

已派Luna私有准备1.2诊断patch：须记录同Program的准确LocalVariable/RegionPath/placement/at_region、init/test/next/Phi/reads/effects；临时continue必须在最终投影前强制拒绝，不能返回虚假删除索引。root可在当前产品自身CI等待期间按守卫/原字节恢复约束运行临时诊断，永久候选待当前交付后应用，诊断恢复源码并clean，不冒称实现。资源5GiB free/target1GiB一秒守卫、默认Rust stack、root独占工具链/Git/shared。整个CF11/71/612计数不变。

调度修订：前轮CI stable实际41分钟；当前产品完整local验证和push已接受，下一片临时诊断/完整class只读观察可以并行推进，最终原字节恢复/clean后仍用原product/live闭包验收自身CI。此修订不授权把未验收候选混入当前产品。

Task1.2中当前CLI wholeclass基线部分已actual完成并独立接受：29cmd/58raw，原2/JADX4/Jarde4完整class runtime腿全部同八case oracle，constructor3与test45 physical BCI/source map完整，4727闭包同finalbuild/live、target不存在。准确execution SHA4f3322219d09505fe9b81f5753d1c4d0996145581b0e81538d40f39b0d73d22a，archive见results/current-baseline-root-v1；声明owner/后续proof真实诊断尚待，因此1.2整体仍未勾，仍1/9。

1.2完整接受：current CLI wholeclass29cmd/58raw已接受；owner-only临时诊断v1和扩展Phi事实v3各actual test1/0/0+clean两命令，root acceptor-v3均actualexit0。v1 peak347717482B、v3 peak320749435B，4727源码按字节恢复且target不存在。inner42三LocalVariable均slot5/index0、Elevated owner[1] visible/emitted/at_region一致；真实首拒iterator_value_and_phi_use_ownership。V37三条None@97对应Phi22三V37输入、uses空/replacement12，header12=[V37,Itself]、uses空/replacement37。更后的next/cast/effect门未执行。详results/same-ir-diagnostic-root-v1和-v3。

诊断prep-v2词法作用域错误由root静态审阅拦截，未应用/编译；v3仅移logger到同一作用域。acceptor-v1正常SystemExit被捕获由root静态拦截，未跑；acceptor-v2实跑两次因Rust Debug结构闭合regex错误拒绝，完整failure raw保留，v3精确补闭合并核重复文本为同一六use tuple后接受。不改真实证据或生产门。

root接受Phi ownership review-v2：仅检查实际含精确source ValueId的Phi及occurrence multiset；v1对无关同block Phi也要求inert属过度拒绝，已明确纠正。four artifacts已相应修订，同一个existing iterable proof消费replacement事实而非新增liveness机制。written-gap只对新祖先路径收紧为空声明，旧同列表时序债务单独记录；inline/separate cast证书深度不混称。

控制基线v1在工具链前因Runner/manifest不同被拒；v2准确清单后在javac启动前被5GiB守卫拒（4430757888B），没有Rust/Java编译。恢复资源后v3实际43cmd/86raw，collector/checker exit0：Positive四isolated生成class可运行，combined/Refusals八compile失败/no runtime；这是观察，root独立接受仍待，不先勾1.3。

1.3已root独立actual接受：acceptor-v3 exit0，43cmd/86raw与全部input/tool/live4727源闭包、guard/default-all/20method maps核准。原两JDK15行oracle SHA d785d6e818f4c66780362f43cb524dd412ffd57d8d5b63f9e1e15c3136132240；Positive四同源isolated完整generated class同原，旁侧Refusal仍原源码，不能称生成Refusal运行。JADX四完整class在written-gap-iterator-throws-before-write返回1/原0，trace同values.iterator()；其异常求值重排不作为本仓标准。Refusal四profile各缺24物理BCI，八compile失败/no runtime原样保留。3/9仅基线与架构范围接受，永久生产候选/新的Phi gate与更后门尚未运行。

3.1实际接受：2026-10-11 19:58:18 UTC两个独立acceptor均exit0。fixed29cmd/58raw，四Jarde完整类各1foreach+1while/default-all同源同map，原/JADX/Jarde十runtime腿同八case oracle、全部test45/ctor3物理来源完整。controls43cmd/86raw，Positive四isolated完整generated class同十五行oracle且内层恢复；Refusal/combined八compile失败/no runtime、24BCI缺口与JADX异常重排差异保留。准确capture SHA92f634...958a9/f29269...97c5，focused父receipt与4729闭包及真实build子命令绑定见results/current-candidate-root-v1。此验收只支持冻结Runner案例，不作一般语义等价或fullworkspace声明。

2.2实际接受：regressions-root-v3 20:01:39–20:03:23 UTC实际8cmd/16raw、lib365/0/API2/0/reader census1/0/p5常规5/0/1及再生1/0。4732闭包前后/live一致，peak654224895B/final clean/defaultstack/guard未停，candidate-v3与runtime-v2 CLI准确同8bdb868...。owner/current-vs-other-list/anchor/Incomplete/type/visibility/emitted/split身份、真实SSA重复input/selfedge/无关Phi/reader拒绝，以及编译所得mixed/live与alias额外Load11均通过。mixed actual sourceV14→Local3 Phi6@18 [V14,V22] live；alias actual sourceV10 useLoad11、slot4 Phi7@25 arms[V15,V18]，不得冒称exact-source跨slot Phi。两helper AnalysisSteps usage1/limit1@35；公开API near-complete AnalysisSteps usage8230/limit8230@6、written2273，未发布text/map/content/news且pre-cancel零计数。具体实际维度/位置不替换为预设proof phase。

fixture单canonical javac8 class606B，javac23同source603B不同字节，两腿真实javap及独立decoder核3Code/0handler/5targets/0jsr；reader人口由1103/4911/469/2987/10精确变为1104/4914/469/2992/10，指纹再生只改manifest。原private-v3 alias假设在应用前静态拦截；采用v4，全部producer保留。首版验收器单行named regex漏计p5 nocapture打印，实际exit1原raw保留，v2分段核terminal状态实际exit0。full workspace/Clippy/strict/自身CI/main交付仍待。

两次 fresh 全工作区失败均保留，不能拼接接受：[v1](results/workspace-failure-root-v1/README.md) 在 Unicode 的 SB.cond 旧 Iterator/while golden 停下；[v2](results/workspace-failure-root-v2/README.md) 已通过该批，再在 countEmpty 的旧 while 断言停下，JSONDecodeError 是 runner 解析 panic 文本的次生错误。两次 finally clean 均完成。仅修准确过时预期：[SB 完整类](results/unicode-golden-root-v1/README.md) 与 [countEmpty 完整类](results/gateway-expectation-root-v1/README.md) 各实际15cmd/30raw，原/旧CLI/新CLI在双JDK六条运行腿相同；新gateway断言精确 foreach/Object头、String cast、计数/return及原BCI anchor/唯一owner，独立负例不变。Luna只读复核实际raw见 [notes](results/handoff-review-root-v1/private/notes.md)。

最新 [regressions-v5](results/regressions-root-v5/README.md) 实际10cmd/20raw，lib365/API2/census1/p5常规5-0-1及再生1/Unicode4/gateway15，全通过；4732输入闭包前后/live相同，defaultstack、5/1GiB守卫、peak744067802B、finally clean。两个helper Stop及公开late Stop同此前真实值，candidate-v5同runtime-v2 SHA。第三次fresh全workspace正在执行；尾部13批 [诊断](results/tail-diagnostic-root-v1/README.md) 仅为定位验证，不替代完整通过。

第三次fresh [full workspace](results/full-workspace-root-v1/README.md) 实际完整99cmd/198raw、362目标/96批、3480-0-97/3577逐named已独立接受，peak815113245B、4732闭包一致并finally clean；两次旧失败批75/85分别11-0-2和112-0-0通过。随后final private输出路径记录器失败保留于 [recorder failure](results/final-recorder-failure-root-v1/README.md)，只修stream绝对路径adapter后新v4的fmt check通过；Clippy在cfg(test)的精确Phi membership写法报manual_contains，实际失败及成功finally clean见 [Clippy failure](results/clippy-failure-root-v1/README.md)。生产无改动，测试写法待改contains；最终字节输入将重新验收，不把旧full拼接升级为最终提交通过。3.2仍未勾。

3.2最终实际接受：[regressions-v6](results/regressions-root-v6/README.md) 与 [fresh full-workspace-v2](results/full-workspace-root-v2/README.md) 及 [final-v2](results/final-validation-root-v2/README.md) 绑定同一最终4732输入闭包。完整362targets/96批、3480-0-97、3577named/19required，最终5cmd/10raw fmt/CI-Clippy/global strict354-0/CLI/final clean实际全通过，独立acceptor exit0。CLI61314560B/0555/8bdb868...准确同双JDKruntime-v2；contains只是cfg(test)写法，无生产变化。3.2已勾，7/9；3.3产品自身CI与3.4最终主线交付尚待，不借前片CI或未走完的driver成功。

产品已实际提交推送：993e9f8c0f413bb7c31538e6f7c2c92a742e7af4，自身CI38176302604正在运行；MSRV/supply chain/fuzz成功，stable完整双seed尚待。7/9不变。并行下一固定outer13同IR诊断已实际接受，全部4732源码恢复/clean，详下一片recover-proved-iterator-assignment-foreach/results/outer-assignment-diagnostic-root-v1；该诊断没有永久改写，不影响本产品/live字节绑定，亦不关闭本产品CI或下一片实现。

3.3已root实际接受：[自身CI](results/own-product-ci-root-v1/README.md)绑定993e9f8/38176302604，四jobs/52steps、两seed各362targets/3480-0-97及3577逐named同本地，19required全部通过。capture及verifier-v3均实际exit0；原v2漏记examples pin错误与显式七example productGit/live补充核验保留，不伪造历史source pin。当前8/9，关闭文档后按真实HEAD做3.4 clean audit，target不存在。

3.4实际接受：关闭文档e29b587f7推送后root真实clean auditor exit0，15物理checkout/14aux detached clean main祖先、main/origin/remote及仅main、所有无target/fuzz-target、free10456530944B及4732输入/五实际receipt/CLI准确绑定。详[clean audit](results/clean-main-root-v1/README.md)，receiptSHA25690a748176b8cf1b749804b685872b8e6212598fec14216537a6dfbfede791983。窄片9/9，CF-11/71/612完成数不变。随后关闭记录提交按真实HEAD外置再次audit后才应用下一候选。
