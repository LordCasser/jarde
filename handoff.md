# Jarde 接续入口

持续目标保持 active，用户要求继续在当前聊天推进。按固定 JADX **71 语法单元 / 612 测试文件**的账本完成大颗粒 MVP，再探索双方尚未覆盖的情况。组件测试通过不能直接增加整体完成数。

**2026-10-13 当前执行点：workspace-v9 session37497 已真实terminal exit0，独立accept session88638也exit0。当前没有活动Cargo工具链，target absent。** 两seed各96批/364targets，各3525成功/0失败/98ignored；合计7050成功/0失败/196ignored，195命令全0、4854来源前后一致、资源守卫/default stack/finally exact clean全部接受。receipt SHA1b0215f1ae01bf41515cc4fbda3c3e3d7667a7d4e8524c311352956a0980fa56；acceptance `/private/tmp/jarde-ordered-field-full-workspace-acceptance-root-v9.json` SHA6b119c7fedec7a84c4c9cd0aa1b66d01edec49c6fd022fe8b4de5f8ae2cbf188。见 [完整工作区验收](openspec/changes/recover-ordered-field-concat-operands/results/full-workspace-accepted-root-v9/README.md)：403证据成员+嵌入README/manifest，共405文件，2162740B/SHAfe9920f5be1d353bf566c41aba5db6f957f1b7581f5f2c7efbe36b7d29c65877，root逐成员复开。父3.2/子2.2已关闭，当前父7/8、子5/6；下一步提交推送精确产品并接受自身CI，再真实HEAD干净main。

focused-v12 session2689已exit0并独立接受：705成功/0失败/4ignored（java lib391、reader lib178、integration136），8命令全0、4854来源一致、target absent；receipt SHA887bff1cb4999f13d03e9d3e6b411a032c46ce26151b0f566ea78448a7869401。CLI `/private/tmp/jarde-ordered-field-final-check-root-v12/candidate-cli` 62277024B/0555，SHAc8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd，逐字同已接受完整类矩阵。见 [局部验收](openspec/changes/recover-ordered-field-concat-operands/results/reader-census-fix-and-final-local-root-v1/README.md)：29成员/625215B/SHA7cfb7bfc50b4aef27941091ba96f295b6bb6261a2889522debf75fb57d593ab5。root核全部4854 staged blobs、29084716B逐字同v12。

workspace-v8 session64725历史失败只来自reader末尾语料总量断言。独立核1140旧class恒同、23新增class增量139body/27handler/138target/0subroutine后，仅更新总量为1163/5157/501/3279/10及注释。原失败见 [194成员归档](openspec/changes/recover-ordered-field-concat-operands/results/full-workspace-reader-census-failure-root-v8/README.md)，没有拼接到v9。旧accepted-v8准备未执行，不能当成功证据；不得再poll已terminal的37497/88638/2689/64725。

自身CI capture `/private/tmp/jarde-ordered-field-own-ci-prep-luna-v1/capture-own-ci.py`、verifier `/private/tmp/jarde-wrap-workspace-acceptance-prep-luna-v2/verify-own-ci.py` SHAe1e9544af84c3397e170f2720743eab479e21e500a677b2f680431e22e567df9。参数按确切新product commit/run、v12 focus与v9 workspace/acceptance填写，不能复用fd13旧CI。新的私有归档准备 `/private/tmp/jarde-loop-field-own-ci-archive-prep-luna-v1/README.md`，script SHA5bca917d12e856435a976cfc3b34ec8e818d464c9a6f0600895a368f00cf4e8c，root已审跨档绑定：v12 full receipt在local归档，v9含其摘要及local引用；既有归档不重复复制。尚未执行自身CI捕获/接受/归档。产品commit message已在 `/private/tmp/jarde-loop-field-product-commit-message-root-v1.txt` 准备。

下一项EM20 baseline工具v2仅使用-g，default/all是Jarde证据模式，不能称javac debug/none。Luna正在私有v3最小添加 --javac-debug(debug,none) 与 -g:none；完整输入/handled/17场景、工具pins与guard保持。尚未执行下一项baseline或修改其生产代码。

workspace-v7 session4414已terminal exit1，第76批只有WA.main的println预期陈旧；其他全类字节和九accessor正文相同，同批生成全类编译/运行对照已通过，finally exact clean0。root保留历史样本，核旧literal恰1次再替换为精确新字段拼接，仍全文exact比较。见 [失败归档](openspec/changes/recover-ordered-field-concat-operands/results/full-workspace-wa-golden-failure-root-v7/README.md)（173成员）；该失败不是生产恢复回退。

最新 focused-v11 session5827已terminal exit0，八命令全0、527成功/0失败/4ignored，含391库测试与完整九accessor套件，4854源码前后一致、target absent、fmt/当前CIClippy/global strict通过。receipt SHA c840bdb9bbab37339e6cdcbd4663174bd29aa7fa10b6c35f798ae79722b8ebd5；CLI `/private/tmp/jarde-ordered-field-final-check-root-v11/candidate-cli` SHA c8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd，逐字节同v10，源码闭包唯一变化是该测试预期。原已接受的完整类矩阵仍覆盖此二进制。见 [最新局部验收](openspec/changes/recover-ordered-field-concat-operands/results/wa-golden-fix-and-final-local-root-v1/README.md)（23成员/508748B，SHA2904892a6d9518f1c215bb4ea8635744db1bce85b04953124691f5786263effe）。root随后核全部4854 staged blobs逐字同v11，计29084349B；尚未提交。focused独立acceptor `/private/tmp/jarde-wa-focused-accept-root-v1.py` SHA934df9986016bff1eebaecce699edfb1a11d69a0b59d268eedf827d821288dcc，保留原门并加两项WA具名通过门。以下v10为生产二进制一致的历史完整类证据，v11仅覆盖修正reader总量之前的来源；当前全来源须由v12实际验收。

workspace-v6真实exit1与失败时源码在 [wrap Load 回退](openspec/changes/recover-ordered-field-concat-operands/results/full-workspace-wrap-load-failure-root-v6/README.md)，126成员/1455868B、SHAeaaa3a35aff9d8cfe8c227adb416ffe533f71c2139877c6637f59383b9ffd52e。第59条C2W.main被新增全局Caught拒绝误判为slot1逃逸，第60条finally clean0。root撤回该分支，保留handler入口store ValueId身份证明、新Load逐handler别名与精确类型门、空/缺失Phi与Itself拒绝。Caught源自throw site的Stack(0)，catch local须经入口astore；Luna对抗审阅未找到aload绕过入口的可达路线。build.rs最终SHA2bc675af2124bb4d883ce18b24b415da917648a59fbf426ea3f99e0a5a5c4678，原scope与wrap断言不变。

focused-v10 session85362已terminal exit0，实际八命令全0：518成功/0失败/4ignored、4854来源前后相同，fmt/当前CI Clippy/global strict/CLI构建/finally exact clean通过、target absent。CLI `/private/tmp/jarde-ordered-field-final-check-root-v10/candidate-cli` 62277024B/0555，SHAc8c18f26a256027262b514455c33f24d0ae1cbcec658e3d6ab76b2c3a9c2ccfd；receipt SHA722c0c738a85fc03cddb2249049f4fd5087002abe0cdd4e35c4df5173acbbf74。root focused验收及4854 staged/live来源29083524B全部与冻结manifest相同。新工具原14项focused map保持，wrap具名测试独立加入两seed workspace/ownCI要求：runner SHA557d1decba94c3f0e58fee22c7335de77f3087c9337924824724f0412135c38f，acceptor `/private/tmp/jarde-wrap-workspace-acceptance-prep-luna-v2/accept_workspace.py` SHA5a0ba7f3e763706701117017fd628383c7281ee288e3af902a01bfcdaa8f1d4c；ownCI 同目录 `verify-own-ci.py` SHAe1e9544af84c3397e170f2720743eab479e21e500a677b2f680431e22e567df9。root按focused实际raw将旧acceptor的P3计数91改为92、库下限390改为391；14项named与每seed wrap等门不变，活动runner未改。

新完整类矩阵 session61762已exit0，九步骤实际全0：parent-root-v6的40命令/16renders/12运行全部同原；field-root-v5的24命令/4Jarde+4JADX同原；TWR-root-v4的23命令双JDK同原；anonymous-root-v4的29命令保留整组constructor compile-failed/no-runtime，三份生成source与上一候选逐字相同，不宣称整个物理类通过。C2W新增session43779已exit0，27命令/8份完整类双JDK编译/16次main及Runner运行全部同原；JADX default/none均fresh，全部static方法及双pick overload保留，生成源码未修补。root独立验收实际raw/source/argv/pins，receipt SHA f81e64899de9dce803f8f34c6efd58ba9cc161c6fe55ded0eab1e10204a12df4。

局部验收完整归档在 [wrap Load 修复](openspec/changes/recover-ordered-field-concat-operands/results/wrap-load-regression-fix-and-final-local-root-v1/README.md)：665成员/4123217B、SHA9e5d4e93339d197754489c10a177a5ac72c7e088668eb0796b215a91d52ebddb，逐成员复开核验。workspace-v7失败后不再poll旧session；v8也已失败；后续应poll新workspace实际session，workspace实际成功后独立accept_workspace核195命令/两seed target与具名闭包，再提交推送确切产品、捕获并接受自身CI，最后真实HEAD干净main。失败需保留raw并修复，禁止拼接或调低断言。

以下 focused-v9及更早记录均是历史，不能替代当前修复的验收。

历史focused-v9（含后续workspace发现的回退）session76270实际exit0、8命令全0、513成功/4ignored、4854来源不变、finallyclean0/targetabsent，rootaccept-v5通过；冻结CLI `/private/tmp/jarde-ordered-field-final-check-root-v9/candidate-cli` SHA947d6dd29a2f8e0442fc848c8ab24da2a292ea54d0ca54e574a4be26c78059d2，receipt SHA35f5a89750967a3ae99e0c1332f7f76ce24acf414002b1514417882b72bbc100。新Load复用bounded catch别名证明，补空phi/directCaught，逐写只认named精确类型或java.lang.Object；不认简名Object/Unknown兼容。原SHARED/ESCAPED断言保留。

新完整对照：parent-root-v5实际40命令/16renders/12完整类同原；field-root-v4实际24命令/4Jarde+4JADX全类同原；TWR-root-v3实际23命令全0，原/JADX/父项/当前双JDK同原，one/two文本恒同、只use/close改善。旧TWR checker路径钉旧CLI的失败原样保留，root-v4新checker接受相同实际raw。匿名-root-v3实际29命令，3份当前source与旧候选逐字相同，整体已有constructor债务仍compile-failed/no-runtime；root/Base投影历史证据仅按相同源码复用，不宣称整个物理类通过。见 [最终修复局部归档](openspec/changes/recover-ordered-field-concat-operands/results/reference-load-regression-fix-and-local-acceptance-root-v1/README.md)，620成员/5344251B、SHAaae359d8543d6488f4f3ca0e14437170332382dcf632443d93f54e37cce9553b，root逐成员复开核验。

workspace新runner提前报告真实产品失败，旧batch67已离线replay；七个focused普通命令包括通过测试后的 package cargo clean，第八为finallyexactclean。root-v5 runner SHA b67ceb155c458eb32a7df27ee3a031e87343d756e4c9c0b4328e4cd808845359、acceptor SHA43b416ab710ddec2830fda717a2ab5e5bb91b5266aacd8ebdfe42683383fa6d1；ownCI root-v4 verifier SHA44f3e5a069718cb3104627a38a72666bf47e6612c491b65e159e67214aec1b6c。三工具检查所有七门+最后clean，14项focused具名映射一致，root补CLI构建必须在测试/package-clean之后；Luna静态审阅保留。workspace终态通过后先独立accept_workspace核195命令/双seed精确target/named闭包，再提交推送实际产品、查自身CI，不可复用fd13/ca45 CI。

focused-v7实际391库测试及8 integration targets全成功，但CLI build后target1097288324超过1GiB，守卫判失败/finallyclean0；不能当成功receipt。focused-v8增加测试结束后的 `cargo clean -p jarde -p jarde-java`，7正常命令+finallyexactclean共8，所有门与4854source closure通过，CLI15824b47；新parent runtime-root-v4实际40命令/16renders/12未改写完整类compile/runtime均同原。随后root审查进一步收窄简名Object与Unknown类型门，并在真实handler SSA单测增加FQN/简名边界断言，因此v8证据是历史，v9必须新CLI与新矩阵。

workspace私有root-v5 runner已修在artifact JSON前报告真实失败；上一真实batch67离线replay接受。focused七命令适配保留全部gate、明确局部clean argv，并增加两个新lib具名检查。路径 `/private/tmp/jarde-ordered-field-full-workspace-prep-root-v5/{workspace_runner.py,accept_workspace.py}`，ownCI verifier `/private/tmp/jarde-ordered-field-own-ci-prep-root-v4/verify-own-ci.py`；Luna独立只读审阅中。下一步v9成功后：rootaccept-v5、fresh parent-runtime-v5/field-runtime-v4/TWR-root-v3，然后fresh双seed workspace-run-root-v6；不能拼接历史失败或未跑lane。

**2026-10-13 当前接续点：main 的循环与字段拼接候选尚未提交交付；父项6/8、字段子项4/6。** 当前产品锚点仍 fd13a034，未声称自身 CI 或完整工作区已通过。

最终局部门禁 `/private/tmp/jarde-ordered-field-final-check-root-v6` 的 session64098 已 exit0，root独立接受：fmt/当前CI Clippy/lib390、TWR4、P5 bulk5/1 ignored、fingerprint5/1 ignored、field-copy3/1 ignored、Unicode4、p3 patterns92、CLI build/global strict/finally clean均成功。4854来源前后逐字相同，冻结CLI SHA `f991644eb87cef29df1509b270ba48d7c5d26ba72f46a5fae3271fc4528e3622`，receipt SHA `71d00d2c5f890a6272ae57d3d5063d4783179e0776696e21d925a11b41e52c63`。本轮只更新P5三处准确预算基线与新增29项fixture索引，未改已接受的产品正文。

P5实际新旧CLI对照90命令/43物理方法，text/outcome/content/source_map全等，15计费维度只增加44 AnalysisSteps：Guarded.syncThrowsCatching 13、secondInitFailsCatching 13、suppressedCatching 18；IR阶段15计费维度不变。ignored recorder与随后正常严格测试相互核对many=12104、direct/shared=19955，保留374请求的正文/分类比较。CLI未序列化canonical CFG，不把失败合取项推断冒充直接扫描图规模证据。见 [589成员归档](openspec/changes/recover-ordered-field-concat-operands/results/p5-billing-and-fingerprint-closure-root-v1/README.md)。fingerprint中断的running原receipt未改写，root实际清理及29项增量核对后，由final-v6正常gate验收2193项。

**修复前完整工作区记录（已失败，非活动工具链）：fresh双seed `/private/tmp/jarde-ordered-field-full-workspace-run-root-v5`，session83104。** 使用 `/private/tmp/jarde-ordered-field-full-workspace-prep-root-v4/workspace_runner.py`，metadata实际96批/364targets/总195命令，两固定seed；尚在第一seed。禁止并行工具链或清理活动target。原v2前置解析漏掉--nocapture多行P5结果，root-v3又漏should-panic名称，均在创建workspace目录/Cargo之前exit1。root-v4只修日志解析，实际严格提取focused的390与116具名结果（后者113成功/3 ignored），完整inventory/失败/资源/cleanup条件不变。未拼接先前失败workspace-v2/v3/v4；它们分别揭示匿名旧断言、TWR旧golden、P5旧基线，证据归档保留。

当前父项40命令/80raw、16renders与12完整类compile/runtime同原，四份retry-v2的Error逃逸/消息/计数已修复；字段24命令/48raw中的4份Jarde与4份fresh JADX的11行输出同原，Object转换修改后续字段/null/char[]/volatile/null receiver/source-map已独立接受，见 [集成归档](openspec/changes/recover-ordered-field-concat-operands/results/integrated-comparison-root-v1/README.md)。匿名子类super前capture赋值的独立编译差距保留，原与当前投影完整root+Base双JDK行为同原，见 [匿名归档](openspec/changes/recover-ordered-field-concat-operands/results/anonymous-regression-and-final-local-root-v1/README.md)。TWR完整类原/JADX/父项/当前双JDK23命令同原，只改善use/close两字段concat，one/two资源正文恒同、历史golden不变，见 [TWR归档](openspec/changes/recover-ordered-field-concat-operands/results/twr-regression-and-final-local-root-v1/README.md)。final-v6 CLI逐字恒同这些运行门使用的产品二进制。

交付门继续保留：最终修复的新focused/完整类矩阵及fresh workspace两seed实际通过后，才能关闭父3.2/子2.2；再提交推送精确产品、验收自身CI、真实HEAD干净main审计。不增加整CF18或71/612计数。具体当前session和私有工具版本以本文件顶部为准，下面旧证据仅为历史。

并行只读下一项分析已完成：`/private/tmp/jarde-em20-reuse-design-audit-luna-v1/report.md`更正旧结论，reference_retype_split已经有SSA引用生命周期拆分，debug例卡在records非空及同名Whole门。优先把LVT范围/名称核到已有所有权事实，不新增solver；当前全局NameTable会给第二value加suffix，不能承诺同名文本。正式实现前先用当前冻结CLI复跑 `/private/tmp/jarde-em20-raw-alias-escape-baseline-prep-luna-v2/run-current.sh`完整类baseline（已适配v10 receipt，尚未执行），核handled()独立拒绝是否仍在；不修补生成源码、不预定none输出通过。本项尚未提案/实现。root新决策 `/private/tmp/jarde-em20-architect-decision-root-v1.md`：将LVT read范围匹配到现有SSA Split.by_bci身份，初始化store由SSA归属；names-plus-Split复用typed_split形状。DebugLocal.descriptor()只在唯一generic匹配时保证存在，不能为普通LVT强加新reader机制。必须用最新v12 receipt参数复跑完整类，并重新观察outer-try foreach的handled()是否仍拒绝；不预定旧失败仍在，也不以方法局部改善计整单元完成。

/tmp已授权结束任务ScriptOS缓存清理及增量缓存实际累计释放2.35GiB，验收材料保留，当前可用约23GiB（构建前）。只有main分支，14辅助worktree均detached；本轮不新建worktree。root独占共享工具链与Git，Luna只读/私有准备。

## 最近完成的 MVP

OpenSpec [recover-structured-list-generic-bodies](openspec/changes/recover-structured-list-generic-bodies/tasks.md) 当前 **9/9**。结构化 void 正文的物理 census、List 参数真实用途证明和泛型声明/调用正文事务已经实现；复用现有 AST/精确 argument edits，不新增 solver、pass 或公共 carrier。永久测试覆盖 String/Integer、inner option、null add、控制反例、三 profile、来源和停止。详见 [verification-root.md](openspec/changes/recover-structured-list-generic-bodies/verification-root.md)。

**本轮本地验收起点是 e9e3a0ec27e5bf7698bdf4817ff08d38bcb63374，交付提交以当前 Git 与下述自身 CI 记录为准。** 产品 `ca45c0ed2193fa8709eeba1196ef325f1804f63e` 已提交推送；自身 CI `38225064059` 已独立接受，3.3 闭合。已推送检查点 `70cd4b48410b118011b8be3d4411fd421f3e3dd2` 的干净主线审计已通过，3.4关闭。关闭文档推送后真实finalHEAD还会单独外置审计；检查点不是未来提交的证明。旧产品锚点 5f5fb2c6、旧 CI 38197592542 和 f12668ed0 的最终审计仅证明前一项，不能用于关闭当前项。

## 最新有效源码与本地证据

最终 Clippy 曾发现 report.rs 两处 iter_cloned_collect 和 class_source.rs 一个 unneeded_wildcard_pattern，root 已作等价写法修正。当前 report.rs SHA ecf8c4cb10199555ae756cc3754a8a7dfe0cbc4255e7acd6280f94aea51903fb；class_source.rs SHA 16e7a07f5f133f948a5d88ea467e397f9685a18e56cd8a4734792d6efbfaa1bb。失败与后续 Clippy 通过分别保留于当前 change 的 final-clippy-failure-root-v1 / clippy-preflight-root-v2。

最新 focused-root-v12 已实际通过八命令：API 7 passed/1 ignored、jarde-java 375 passed、reader census 1 passed、P5 fingerprint 5 passed/1 ignored、fmt/build/finally clean 0；4822 输入闭包前后相同。receipt SHA 74350dfc154c2fd77aa8a9636dd8d29b301eb119e167be56a0f18af011996a73。冻结 CLI `/private/tmp/jarde-cf11-structured-list-candidate-cli-root-v4`，61920032 bytes、0555、SHA d3d15d3dd875bc79410298e3ed1e56010fc39c1849e97dc893cadb49d8c65e6f。归档见 [focused-census-and-registration-root-v12](openspec/changes/recover-structured-list-generic-bodies/results/focused-census-and-registration-root-v12/README.md)。旧 CLI/root-v11 证据对应旧来源，不覆盖当前源码。

## 已完成门禁与下一步

全工作区 **session18940 已 exit0**。fresh workspace-root-v3 正常 passed，195 命令全部 0，无 guard stop；两组 seed 各 96 批/364 targets/3502 passed/0 failed/98 ignored，4822 来源一致、finally clean 0、target absent。execution SHA 3c2b8d52b4f1927091ea273b2503e371f066efcf64e62c944c5cdfa4cd08a7ba；独立 acceptance SHA 56d4bf06a166e2074da8951ad37f66fc2df165c045983b17ad6c21657373a461。root 两个解析路径实际通过，归档见 [full-workspace-accepted-root-v3](openspec/changes/recover-structured-list-generic-bodies/results/full-workspace-accepted-root-v3/README.md)。

最终本地门禁 **session49761 已 exit0**：fmt/当前 CI Clippy/global strict357-0/CLI rebuild/exact clean 五命令0，新CLI字节恒同、4822来源一致、target absent，root独立 accept-final.py exit0。final receipt SHA 3cf03577eea64a7d19f1164cbc2da3d322686c7eb5d5baca3dd50f1e22286b04，acceptance SHA f91f3ab30fea420d4d39ba1662d1cb9ef229b64aaef4af9359ef3ae29d43ad0c；见 [最终门禁](openspec/changes/recover-structured-list-generic-bodies/results/final-validation-root-v2/README.md)。3.2 已关闭。

新 CLI v4 的整类对照 **session6599 已 exit0**，root 独立 checker 接受12份/352命令/704raw/80成功运行对/24控制编译失败无runtime/41 oracle labels。新 CLI 精确停止两调用实际4/0；collector因方法name读取层级错误failed，root独立接受其完整raw且保留原失败状态。见 [整类及停止归档](openspec/changes/recover-structured-list-generic-bodies/results/final-comparison-cli-v4-root-v1/README.md)。3.1/3.2均已关闭。自身 CI 已实际接受4jobs/52steps、双seed各364targets/3502-0-98与全部具名结果，4822来源及CLI字节同产品；见 [自身CI](openspec/changes/recover-structured-list-generic-bodies/results/own-product-ci-ca45-root-v1/README.md)。已推送文档检查点的only main/14aux detached/全干净无target/主线祖先/4822来源实审通过；见 [交付检查点](openspec/changes/recover-structured-list-generic-bodies/results/clean-main-checkpoint-root-v1/README.md)。关闭文档推送后，再对真实finalHEAD作外置审计。旧矩阵空间失败没有拼接，71/612和整CF11完成数不增。

## 工具和资源约束

root 独占共享 Git/Cargo/rustfmt/JDK/JADX/CLI；Luna 仅只读或私有准备，不改共享源码，不运行工具链，不创建分支/worktree。外部命令最低 5 GiB 可用空间、target 上限 1 GiB、1 秒进程组守卫、default Rust stack、最终 exact cargo clean。现有构建仍运行时不得清 target 或并行运行其他工具链。/tmp 清理只移除可再生成缓存或做保留字节的历史去重，当前依赖与独有样本保留。

## 后续大颗粒方向

循环体 try/catch 当前 **6/8**，OpenSpec 实际8个checkbox；此前1/9为计数误记，要求没有减少，尚无产品实现验收。当前CLI固定DF/DV完整基线已独立接受56命令/112raw：debug/none × Java8/23原18条oracle同值，JADX default经独立Runner包适配后与none共8个runtime均复现loopCatch错误异常逃逸；4条Jarde整类编译失败/no-runtime准确保留。见 [基线](openspec/changes/recover-loop-body-try-catch/results/baseline-cli-v4-root-v1/README.md)。

固定四方法的同次IR真实四测试均1/0/0，最后clean0、4822源码按字节恢复、targetabsent；root独立接受16个精确physical方法观察，见 [instrumentation.md](openspec/changes/recover-loop-body-try-catch/instrumentation.md)。for/while与loopCatch的终止throw分支不在普通自然环且普通路径不达join，现有catch证明拒绝，handler→join形成第二SCC入口。foreach现有区域证明已通过，另卡在try外aload0@2/astore2@3引用producer的呈现门。优先分别补现有所有权与SSA证明，不能删CFG边强制归约或放宽fragmented特化条件。

无限retry/break两变体已实际独立接受88命令/176raw：原与JADX完整类运行同五个计数/事件oracle；Jarde test()I首拒block0的ownership overlap、整类compile1/no-runtime。v3固定事件文字消除v2的exc nested StringBuilder干扰，但test拒绝相同。八lane同次IR真实9命令/18raw已root独立接受；普通图无环，catch回header重入同一个根Try，block0/handler owners[0,0]。4822源码恢复、clean0、targetabsent。见 [重试归档](openspec/changes/recover-loop-body-try-catch/results/retry-baseline-and-ir-root-v1/README.md)。

[design.md](openspec/changes/recover-loop-body-try-catch/design.md) 已完成并严格validate通过/applyready。方案为同一次finite terminal-body事实供join/scope/coverage/induction、foreach实际引用Load原位呈现、retry复用Endless/Try/Break/Continue并区分protected exit与catch header边界。不能把Try.normal_exit_bci=None误认成guard join=None；候选须用真实Transfer/唯一successor互证。任务1.2已关闭，finite v3/retry v3/reference v2私有候选已由root审阅合并至当前main，串行focused-v5实际385测试与CLI构建通过并clean。root另修复DF保护行起点位于fused block中段的comparison BCI核验，以及retry split将Break/Continue移到外层tail的问题。永久测试覆盖八retry lane、finite物理行变异/停止、引用Load真实dup/间隔effect反例。新CLI整类的四份DV.loopNoTry/DF.loopFinally方法文本均与基线逐字相同；两个fragmented完整类文本已与旧CLI逐字节相同，八retry报告的break/continue精确物理BCI已核；retry新增证明内部Stop和完整双seed/Clippy/自身CI仍待完成。DF.loopFinally handler51 uncovered为独立质量差距，完整DF四组实际编译行为已同原；仍保留原方法与诊断，不删除方法或修补生成源码。

Structured List真实finalHEAD `9dc64e4df464b157f430bae63c74cbb2c72d195e` 已完成外置clean-main实际审计，`/private/tmp/jarde-structured-list-clean-main-audit-final-root-v1/execution.json` accepted_clean_main；该审计只覆盖该HEAD。后续仅循环设计/证据文档变化不替代新产品CI；新产品修改后必须重新完成门禁。

历史验收与架构债务分别保留在各 change 的 results 和 [completion-review.md](openspec/completion-review.md)，不能混入当前产品改动扩大范围。
