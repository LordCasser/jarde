# 结构化List泛型正文验收

当前5/9：1.1/1.2/2.1/2.2/2.3已验收；旧root-v2整类对照通过，新root-v3字节身份变化后3.1重新待刷新。尚未完成fresh全工作区、自身CI或提交。前片产品5f5fb2c6自身CI38197592542与最终文档f12668ed0 clean-main outside audit真实通过。前置v3诊断Cargo test/clean0，完整4784source恢复/target absent；采集器把UsageSnapshot并入结构逐字相等而失败，原失败保留，root独立结构观察acceptor实际exit0，45条逐BCI唯一完整、参数两槽6读无写、cast22/52与add79/remove91已核；仅接受观察，不是泛型产品通过。见[证据](results/structured-observation-root-v1/README.md)。此失败不代表产品修复失败，也不能抹去重记成功。

根目录私有实际平台契约采集 `/private/tmp/jarde-cf11-list-contracts-root-v1/execution.json`：6次Java8/23 javap exit0，List.add(Object)有Signature(TE;)Z、remove(Object)无泛型Signature、remove(int)有(I)TE;、iterator/next泛型签名核准。源级绑定仍需真实完整用途证明。root独占所有真实工具链，编译成功不记成实现通过。

## 六份实际基线与私有补丁审查

root 已完成合并控制、独立 String、独立 Integer 三家族的 debug/none 六份原/JADX/旧 Jarde 双 JDK 整类编译对照。独立汇总 v2 实际退出码 0：166 命令、332 raw streams、28 精确 oracle labels、24 个 Jarde 报告物理 class BLAKE3/length/location、方法与原始 map origin 身份核准。32 组独立正例编译运行与原类一致；24 组合并控制编译失败无 runtime，handler 来源缺口保留。见[独立汇总](results/six-baseline-summary-root-v2/README.md)、[合并 debug](results/controls-combined-baseline-root-v1/README.md)、[合并 none](results/controls-nodebug-baseline-root-v1/README.md)、[独立正例](results/positive-baselines-root-v1/README.md)。旧产品仍 raw List，此证据不证明泛型产品通过。此基线阶段停止控制和完整反例 SSA 归因尚未完成，当时1.2未勾选、1/9；后续补足状态见末段。

生产私有 v1 未应用；v2 经 root 审查后已应用。root 对同次真实 SSA 的再次解析确认 Local4/V34/Store25 具有 Phi 与 BCI57/67/77 多次合法 uses，而 add 参数的 Stack V52/Load77 才唯一使用于79；要求整个局部变量单 use 会误拒固定正例。v2 修正此问题、无 add 槽的空 actual-types 门控、Vec move 和计费变量；root 再修复实际编译暴露的引用/元组/slice匹配错误。

focused-root-v2/v3 实际编译失败分别保留于 `/private/tmp/jarde-cf11-structured-list-focused-root-v2/execution.json`、`focused-root-v3/execution.json`。v4 生产编译成功，公开API结果为1通过/2失败/1 ignored；precancel通过，String正例仍raw，Integer尚未执行到。v5临时诊断复现相同失败：List用途证明进入4次且没有拒绝分支，说明用途证明通过，事务后续仍拒绝。诊断恢复原源码的外置记录 `/private/tmp/jarde-cf11-structured-list-diagnostic-instrument-root-v1/execution.json` 中 `source_restored=true`；其外层exit0仅表示恢复成功，不能冒充测试通过。v2至v5四次focused记录均核完整4805来源闭包稳定、finally clean成功、target absent，当时未进行library测试或CLI freeze。

root已修复两处事务遗漏：最终编辑集合重新证明List移除；窄VoidBody进入共同AST编辑/重发射路径。保持清单相等校验与原子安装。focused v6 实际六条命令exit0，3公开API/371 library测试通过，CLI冻结SHA beb4c3e747512be55a1d657726f8c8382c462376eb39ca8011941cbae5c03879，finally clean/4805来源稳定。八份当前对照独立接受224命令/448raw/48组正例完整生成类/24组失败编译无runtime，见[验收](results/current-comparison-observation-root-v1/README.md)。此为保存观察验收，尚未关闭产品任务。

内层元素和null实参的实际整类探针仍raw；root已按原设计应用有限SSA null证明和至少一个匹配循环祖先门，不新增机制。复制永久冻结fixture，新增反例/inner/null/实测lateStop API回归。focused v7实际6通过/1失败/1ignored：null、negative、lateStop与旧正例通过，inner-option仍raw；library/CLI未执行，4822来源稳定、finally clean成功。内层Declare init尚不在赋值库存，正在补有限库存，见[失败](results/focused-inner-init-failure-root-v7/README.md)；旧v6不代表新增修改已通过。原始边界/预算观察见[记录](results/add-boundary-observations-root-v1/README.md)。

独立 handler 债务已从当前 region.rs 核实：loop_body 清空外层 own_try，header-tested gate 不接纳已有 named-catch 证明；复用精确 ordinal/range/handler 校验的方向见独立审计，不混入 List 泛型实现。

## 当前 v9 初步产品验证

有限初始化库存和Null实参证明经root应用，v9实际全部7 API/371 library通过、CLI build/clean0，4822完整来源稳定/target absent，峰值747983549 bytes。见[证据](results/focused-list-boundaries-root-v9/README.md)。v8 inner已经通过，唯一失败旧lateStop期望25195→实际25299已保留[原始失败](results/focused-late-stop-count-failure-root-v8/README.md)。当前最终六家族12份完整生成类对照正在运行；fresh全workspace、自身CI和当前HEAD交付未完成，任务完成数暂不增加。

当前独立接受2.2/2.3两个组件任务：v9 371 library具名结果包含Local wrapper的wrong method/BCI/opcode/target/arg index/duplicate/physical checkcast拒绝及input AST不变，公开API验证声明与正文同时发布、保留对象remove及物理cast；三profile原始正文/map一致、UTF8有效、precancel和最终List证明停止不发布部分overlay。当前3/9，2.1的全量反例事实归因与最终整类验收继续完成；fresh workspace/CI未关闭。


## 最终完整类对照已接受

root 最终 checker v4 实际 exit0；此前 v2 的 Runner 副本路径绑定失败、v3 的 pins schema 失败原样保存。12 份六家族 debug/none 双JDK整类对照共352命令/704原始流/41独立oracle labels；80组完整生成类编译运行逐字同原，Jarde与JADX各40，24组控制编译失败且无runtime，handler每模式4个map gaps保留。原始签名/正文、物理class/method/BCI与UTF8 source map、编译 target/Runner副本/生成class身份均核准。inner对象identity与两型add(null)对照通过；fixed8路径含null/CME/heap污染。

新CLI独立lateStop实际exit4/Partial/AnalysisSteps，limit32531/used25299/requested8112，最终List再证明停止，不安装泛型overlay，raw正文/map同完整请求。初次记录器路径失败未记录CLI退出码，原样保留。完整归档与独立summary见[最终对照](results/final-comparison-observation-root-v1/README.md)。task3.1勾选，当前4/9；反例完整SSA归因、fresh双seed workspace、自身CI与主线交付仍未完成。双seed workspace root-v1已真实启动，期间冻结共享源码，不把运行中记录当通过。


fresh双seed workspace root-v1实际终态failed：第一seed第64测试批的p5_corpus_fingerprint有实际4通过/1失败/1ignored，30个本片永久fixture未登记；Cargo停止后同批第四目标未运行，记录器随后报summary/artifact cardinality mismatch。实际失败和原因见[归档](results/full-workspace-fixture-registration-failure-root-v1/README.md)。finally clean exit0、target absent、完整4822 source前后相同，未触发guard。3.2不勾选；最小登记须核固定来源、只增加本片30项，随后fresh完整验证。

## 同次反例归因、登记与永久census刷新

root 独立acceptor v2实际exit0接受18条同次Code/SSA/effects/operations和真实class Code逐BCI一致，原v1顺序比较失败保留；以冻结javap补足wildcard Signature，而不由SSA猜variance。alias/store、参数真写、handler/Caught、Object/index remove、物理cast和null唯一路径分开归因；诊断恢复的4822来源和focusedv9闭包相等，finallyclean0。结合原双profile双JDK三方基线与停止记录，1.2已接受，见[归因和永久回归](results/list-same-run-ssa-and-census-root-v1/README.md)。

当前仅登记新增30夹具（原2134保持，总2164），reader实际增量+7classes/+23Code/+1handler/+93targets/+0jsr由root旧接受decoder和Luna saved javap独立计数；不是盲改失败期望。永久四census单测首次因helper输入预算0全部失败，补现有IR测试有界预算后实际4pass，reader/fingerprint也通过，失败原件保留。focused v10产品目标全成功但P5 nocapture输出交错导致记录器误拒，尚未build CLI；私有v6标准捕获后freshv11八命令全0、API7/1、lib375、reader1、P55/1、4822来源稳定及clean0已root独立接受，2.1关闭，见[刷新验收](results/focused-census-and-registration-root-v11/README.md)。

新CLI root-v3 SHA45b5fd1ef637d8879254743d51ea5ce66ec5400558b023e65a7448a9d596de09，与旧root-v2不同；旧12矩阵仍是真实历史通过，当前3.1需新CLI重跑，不借旧身份。fresh full-workspace-root-v2 正在按新source/CLI双seed执行，尚未接受，不把前轮部分结果拼成通过。

## 2026-10-12 双 seed workspace：运行器汇总失败

Fresh root-v2（session90103）两 seed 各96批实际全部命令0，fmt/metadata/clean同为0，共195条；最后clean成功、target缺省目录不存在、4822source前后同字节。原始收据SHA `a230c1a4de86c3d92cc55799bd36be586eacbcffae81f1b5ee8b3df6e01d52c8` 状态failed，错误是把刻意ignored的late-stop诊断当required ok，尚无聚合totals字段。完整原失败归档见[记录](results/full-workspace-bookkeeping-failure-root-v2/README.md)。只读全target独立验收正在准备；3.2仍待最终门禁及独立接受，不以命令均0自行关闭。当前CLI v3的新12组整类对照session26674运行中，3.1也未关闭。

Root 后续独立只读接收器实际退出0，接受本次390raw/195命令与728 suites/7200具名结果，两seed各364目标一次、3502/0/98，合计7004/0/196；summary SHA `638d99f2f718d9bc4c71b6f547ce187b104be8dd9e0d2562f4ac169bbfe3c130`。原runner failed与唯一ignored误分类错误保持，见[同次完整raw接受](results/full-workspace-raw-acceptance-root-v1/README.md)。当前完整workspace测试事实已核准，3.2仍待final工具链门禁。

## 2026-10-12 新CLI矩阵：磁盘守卫阻止末组

Root session26674已终结1：前11组完整collector0，controls-none第15命令启动前5GiB guard拒绝（5359357952 bytes）。[完整原失败归档](results/final-v11-matrix-space-guard-failure-root-v1/README.md)保留12组观察及原failed batch。本机随后可用约3.4GiB、target早已clean，无在跑Jarde工具链。3.1/3.2保持待办，空间恢复后新目录重跑完整matrix/lateStop/finalgates；不将guard失败解释成产品失败、不拼接旧观察冒充新成功。目标仍按71/612推进。

## Clippy 修正与新来源（2026-10-12）

2026-10-12 最新源码：最终 Clippy 实跑发现 report.rs 两处 iter_cloned_collect 及 class_source.rs 一个 unneeded_wildcard_pattern，root 仅做等价写法修正；失败原样归档 final-clippy-failure-root-v1 与 clippy-preflight-root-v2。完整 Clippy 预检已通过。focused-root-v12 八命令 0、API7/1/lib375/reader1/P55/1、4822 闭包稳定、finallyclean0；新 CLI root-v4 SHA d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f（61920032B/0555）。旧 v11/CLI root-v3 仅为旧来源事实，不能关闭当前门禁。workspace-root-v3 正常 passed 规则双 seed 全目标验证正在 session18940 运行，勿重启活跃 handle；工具 prep-v4已冻结。整类矩阵 v12v6 已按实际新 receipt/CLI 填 pins 并完成只读 focused 准入预检，尚未运行。下一循环 try/catch 私有基线准备仍在审阅，不是实现或当前缺陷验收。当前tasks仍5/9、未commit。

## 当前来源 fresh 全工作区已通过

2026-10-12 session18940 实际 exit0，workspace-root-v3 正常 passed：195 命令全部 0/无 guard stop，两组 seed 各 96 批、364 targets、3502 passed/0 failed/98 ignored。390 raw streams、728 目标摘要/7200 具名结果按 metadata exact-once；4822 来源前后相同，finally clean 0、target absent。root 实际执行独立 accept_workspace v4 和 ownCI verifier v8 的第二解析路径，都通过。execution SHA 3c2b8d52b4f1927091ea273b2503e371f066efcf64e62c944c5cdfa4cd08a7ba，acceptance SHA 56d4bf06a166e2074da8951ad37f66fc2df165c045983b17ad6c21657373a461。见 [完整归档](results/full-workspace-accepted-root-v3/README.md)。

最终门禁 session49761 正在执行 normal v4 链，当前尚无结果；新 CLI 矩阵/late-stop、自身 CI 和 clean main 未完成，因此任务仍 5/9。旧 workspace bookkeeping failure 没有改写，也没有拼接作为通过来源。

## 最终本地门禁：3.2 已闭合

normal v4 链 session49761 exit0；fmt/当前 CI Clippy/global strict357-0/CLI rebuild/exact clean 五命令全部0，无guard stop。当前 CLI 全61920032 bytes与 focused root-v4恒同，4822来源前后一致、target absent。root 实际独立 accept-final.py exit0，final receipt SHA 3cf03577eea64a7d19f1164cbc2da3d322686c7eb5d5baca3dd50f1e22286b04，acceptance SHA f91f3ab30fea420d4d39ba1662d1cb9ef229b64aaef4af9359ef3ae29d43ad0c。见 [完整本地门禁](results/final-validation-root-v2/README.md)。3.2关闭，当前6/9；整类矩阵已按新CLI实际启动，未提前关闭3.1。自身CI/main交付尚未完成。

## 当前 CLI v4 对照：3.1 已闭合

session6599 实际exit0，12组全部采集成功；root独立checker exit0接受352命令/704raw/80成功编译运行对/24控制失败无runtime/41 oracle labels，acceptance SHA a9511e5048e2c06ef9f6bc4d4d0baae0f6cd5c59893b01646e9ca36bececbb34。新CLI停止两调用实际4/0，collector读取错误name层级导致failed收据保持；root独立raw验收SHA99c22e68f09db83eadb59140834c043e69584382f11e199e1b0e89a2449dde20接受同物理方法/完整raw正文map/准确25299+8112越32531/partial无generic/full有generic。见 [整类及停止完整归档](results/final-comparison-cli-v4-root-v1/README.md)。结合已闭合1.2同次归因与当前永久回归，root关闭3.1；当前7/9，自身CI及main交付尚未完成。


## 自身产品 CI：3.3 已接受

产品 `ca45c0ed2193fa8709eeba1196ef325f1804f63e` 已提交推送，实际 run `38225064059` completed/success，4 jobs/52 steps 全成功。root fresh capture-v2 保存三条 gh argv/rc/raw；旧 capture-v1 属 b155 产品未复用。root verifier v10 实际 exit0，两 seed 各364targets/3502-0-98，逐具名结果/顺序/状态同 fresh 本地；完整4822动态来源同 focused/workspace/final/live/product Git，CLI准确d3d15d...c65e6f。

v8/v9 两次实际验收脚本失败原样保留：前者 tuple/list 容器比较，后者 ignored 原因文字比对。root 实解析全部差异仅为状态原文的原因，v10 精确规范化 ignored 并另存194条原文观测；没有改产品、测试状态或通过计数。归档 [own-product-ci-ca45-root-v1](results/own-product-ci-ca45-root-v1/README.md)。当前8/9；最后文档推送后 main/remote/15checkouts/4822输入外置审计尚待，不能以自身CI替代。


## 主线交付检查点：3.4 关闭

已推送文档 `70cd4b48410b118011b8be3d4411fd421f3e3dd2` 的root实际外置audit exit0，接受4822来源同focus/产品/docs/live；local/remote仅main、15checkouts/14辅助detached干净祖先、全无target/fuzz target，5GiB满足。见[检查点](results/clean-main-checkpoint-root-v1/README.md)。当前9/9仅关闭本片；关闭文档推送后真实finalHEAD另执行外置audit，不把检查点或产品CI冒充该finalHEAD的结果。CF11/71/612完成数不增加。
