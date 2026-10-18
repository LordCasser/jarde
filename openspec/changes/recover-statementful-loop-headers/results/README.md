# Seq1循环头恢复：证据入口

原始基线、旧38命令完整源码矩阵及当前八API重放见`openspec/evidence/java-syntax-2026-10-18/cf11-sequential-loops-*`，三个包225成员已root独立接受并复开；不冒称本轮重跑JDK。

- `first-refusal-root-v2`：56成员，真实Reader/SSA首拒、完整默认Frame/natural/Store5诊断，临时源码恢复，root独立接受40条含历史/诊断/build/wrapper raw与8API恒同；SHA256 `3a12f06bb52b9d141ba1579a52e51d455d634c42ce9a319c36897a957659aa55`。
- `supplement-baseline-root-v1`：318成员，119命令/238raw，4原编译/8原VM、16旧Jarde/8JADX源码格、48完整重编中32旧Jarde缺return/16JADX成功、32 rebuilt VM352case同原。root实际执行独立reviewer，核全类树/字段/方法/access/物理BCI/方法局部UTF8 map与default/all；SHA256 `f5f0cf17c62ef5b67f33d79bb52654ae8b3ef5b542a8ff0ebb498b42f97f40d7`。
- 首次focused原始1过/补充及Stop前置2失败，dup_x1依赖，11成员见`recover-field-write-test-copies/results/first-focused-failure-root-v1`。
- `focused-test-assumption-failure-root-v2`：9成员，原始/停止/错上下文3内部门通过，第四测试误断定补充只有1直接出口loop，实际为2；编译通过但测试失败，未执行CLI或后续公开测试。root修正原始BCI10/51断言。SHA256 `8206dc274fdad5bbcb35d20217114e0ad408409ae8e4ea798f80f45b55933d09`。
- `focused-origin32-failure-root-v3`：11成员，4内部门全过，公开原始与真实Stop通过；补充Structured且单次saved RHS，但完整来源缺goto32。CLI未构建，不能计作最终通过。SHA256 `25d3a741ce9c182865ee7bf6fda787ac9655e06887f1359bab961eb4bb5fd1e3`。

所有已列包已root核raw/pins/默认栈/guard/source恒同/finally clean并独立逐字复开。第四轮在修复fused exit前缀来源与加入volatile公开控制后暴露缓存失败，实际终态及第六轮验收见下方最新结果。Luna私有候选均不是执行证据，固定71/612不增计。

最新实际通过与早期失败分开：

- `focused-root-v6`：21成员/1096292B/SHA `54d038f1041b083a3e30ce7c2ed899946e71e84aabb02a26bfe75835dd53fb26`；另附绑定归档的root-review.json与可执行reviewer。486 Java lib+4公开+181 Reader，共671通过出现。旧checker少算should-panic、清理行KeyError均为checker准备错误，第六轮产品测试无失败。
- `original-final-pipeline-root-v1`：270成员/1040883B/SHA `03ddc6b1685b7e9bf22d342c6aa6a3d8d35b9b36fd8dff797e42627d5030c74d`。新CLI完整24命令基线、70命令整类矩阵；Jarde16重编/32VM192case同原，JADX12VM72case同原，另两完整outer javac8框架major55失败无runtime。独立physical checker重解析22成功类树。保存reviewer初版NameError/README错误flags范围，实际accepted运行使用root v2及位置参数。
- `supplement-final-matrix-root-v1`：476成员/1095839B/SHA `a3aacfd557501cf4d4139002ee292a2b52176df4f6901adbef25fa58c82dc41a`。183命令/366raw，48完整原样双javac重编/96双VM1056case同原；四原profile8VM88case单列。16Jarde格三个方法均Structured/全BCI/UTF8/物理身份；default/all正文、质量、fallback、map相同，regions随evidence模式变化，不要求恒同。root执行独立reviewer和额外语义收据接受。
- `boundary-inputs-root-v1`：40成员/442777B/SHA `dc23471bc853e13657d1d1cff4a3a57a1a93f68a38d5b82ec43b707286145659`。10命令/6VM验证三个完整边界class；Java两有限case实际运行，两个generated类只加载/反射。独立8Code/104BCI/0handler/16targets/2jsr；非一般恢复正例。
- `boundary-observation-root-v1`：13成员/466979B/SHA `ce0f21e6ed8422879aba9aa447bf42f08f7449cb3c1a4e566adf1b110d212fd4`。7定向+181 Reader通过，真实图/clone事实见root-review；观测版允许无候选，永久测试后已改严格事实，等待广门。额外入口在视图上无natural，不冒称直接helper额外入口分支被执行；jsr同BCI错path只为跨分析证明上下文负例。

以上矩阵同一CLI63146896B/SHA `0bb89f827c1e2b6b0e215d1940299be046a7adc57866389d87e6595ba0295677`。volatile完整矩阵见child，本父补充若被child引用不重复计数。当前15包1792成员全体独立逐字复开，见all-archives-root-review-v1.json；后续广门/seed/CI不属于该快照。

`final-gates-root-v1`：44成员/380889B/SHA `f52deedb3068c8c7766ba8434d60ac8252e3932d2d22e1ef4f2eb64739eb095c`，18命令+clean，785通过出现/0失败/6忽略，含八P5 104/0/5，旧预算不改。追加字段两个明确负例后，仅重跑受影响fmt/指纹/CI Clippy/MSRV/Java/Reader/CLI，`final-gates-root-v2`23成员/374644B/SHA `379b5b5b726719e750a44c08f35d5a00fb723180f4a310174f3bfbf77ce4f0af`实际678/0/1；491 Java lib、181 Reader以及指纹再生/检查，CLI字节仍完全同128方法配置和120次源码矩阵。最终独立fixture增量20class/68Code/1308BCI/4handler/119targets/2jsr，Reader1300/5745/630/4040/14通过，标准指纹2387旧项不变加40到2427。

两fresh workspace seed及产品自身CI仍待实际执行；父5/7、child4/5。后续验收不得把v1/v2通过出现相加当作不同测试数量。总三方矩阵60完整源码格/120次重编（118成功，2 JADX outer javac8框架major55边界）、236 rebuilt VM/2280case同原；Jarde1536case、JADX744case，各原JVM192case另计。字段负例只引用child，不重复计入主语义矩阵。


完整workspace首次失败已确认：`workspace-legacy-expectation-failure-root-v1` 81成员/966025B/SHA `64a55f234447f876d4a29f82cf1baa76f3f02f9bc790446b47dd831ca2b9b199`。34完整批后，第35批command36发现旧LeadingTick fallback预期与新Structured恢复冲突，38cmd/76raw，finally exact clean/target absent，不能算完整seed通过。`cancelled-candidate-ci-root-v1` 8成员/124007B/SHA `1b2ab30dd0503b2eb10064897eb0d06e61ff0ae3781d1ad0ad6a46ca4366ddfb` 记录1bc产品CI39095771161真实cancelled（supply-chain/MSRV成功，stable/fuzz取消），不当作CI通过。

`legacy-leading-tick-final-matrix-root-v1`：139成员/694761B/SHA `5c18dba3c83d5eaaf8939ac17bccf22d81eafd8bc6e64ce255b37d7c769d23f2`。完整原冻结ForConditionRefusals源/字节不改，双javac输入、四Jarde和两JADX完整源码格、12原样重编全部成功，24双VM/288case同原oracle（Jarde192/JADX96）；另四原VM48case。5方法×4配置全Structured/全物理BCI/UTF8及default-all恒同，原冻结javac23全Code一致。原有限case -1/0/1/4，每次leading→assigned→leading，返回/counter/limitCalls逐字相同；wrong-CodeSource控制实际拒绝。root独立review v2误把内部内容标识当SHA256而失败，v3修为完整method owner一致，外部文件SHA256照常核验，实际接受后才更新旧测试。

`legacy-final-gates-root-v1`：15成员/344789B/SHA `a950c8aeaad7604b9af98d810ce20fdc56ccb6a858463fbdeb8e70d9293b02de`。仅旧test后半改为Structured while/no for/效果顺序和全物理来源，AssignedCallBound断言原样保留；fixture README注明历史。fmt、12 p3_for_latch、5标准指纹检查通过（1标准ignored），CLI与所有已接受矩阵0bb字节恒同。新build source_before==after，包含改后的test/README；exact clean/target absent。17通过出现，不与其他广门相加当不同测试数。须以此最新build绑定后续产品和两fresh native源码，不能用旧build的旧test闭包替代。

当前四组完整三方矩阵合计66源码格/132次原样重编，其中130成功、两次旧JADX outer javac8 major55边界无runtime；260 rebuilt VM/2568case同原（Jarde1728/JADX840）。各原JVM240case另计。两fresh native和新产品自身CI仍待接受，tasks维持parent5/7 child4/5。


第二次完整seed实际失败而非旧快照应更新：`workspace-copy-refusal-failure-root-v1` 160成员/1240048B/SHA `132f081c9dc4e1d223a64d4257ab5aab600e0556eb95f436c51550fa2b02eea3`。73完整批后command75，第74批的旧 `recover_loop_test_copy_store` 参数目标与MultiCopy拒绝控制被新路径改为带quoted比较的部分Endless while，两个public断言真实失败，77cmd/154raw，source恒同/guard/exact clean。原完整拒绝snapshots不改，新增task2.3收紧复制准入，2.2重新打开，parent4/8。此失败不是完整seed通过。

`cancelled-copy-refusal-candidate-ci-root-v1` 9成员/345483B/SHA `69f084fd9d751d00457faa583e0b1c37d5aadb63b3e38b7135a0a72a181ffad5`：5a自身CI39097292812实际cancelled，supply-chain/fuzz/MSRV成功、stable取消；3实际API/ZIP命令6raw，双API job结论一致，官方45个ZIP成员可读。不作为CI通过。Gate修复后需新build/新candidate/两个fresh seed及自身CI；此前0bb的完整正向矩阵仍是历史有效Java证据，须将最终CLI的全部源码/来源重新绑定，不能拿未验收候选交付。


## 头部复制准入修复：已接受本地门，完整 workspace/自身 CI 待验收

`copy-admission-focused-root-v1`：29/0/1，两个真实 Reader 新测试、旧公开拒绝与 Seq1/LeadingTick 门。新 CLI 63164416B / SHA256 `aabf62fe373876d176e6001bf3ed1fd84f80d2169ff5e787c4095c5d253c63d9`；20成员逐字复开。实际改动与 proof-context 边界见 `copy-admission-scope.md`，未更改旧负控快照。

`copy-admission-broad-gates-root-v1`：18命令/36raw，fmt、CI Clippy、MSRV1.88、Java lib493、Reader181、公开复制/LeadingTick/Seq1、八P5 104/0/5、CLI。合计798通过/0失败/6既有忽略。3187来源逐字不变、默认栈、资源守卫、finally exact clean，target不存在；CLI与focused逐字相同。独立reviewer核每条真实harness与两新增Reader各一次。41成员/402470B/SHA256 `7a57344570e37ebe972abbfee8f30c039bf53ff5c87342396a0b717a5e84b3b3`。

`copy-admission-api-replay-root-v1`：56实际CLI命令/112原始流，原始8、补充16、volatile16、LeadingTick4，共44正向单元。逐份完整生成Java及所有递归非usage JSON与旧0bb报告相等，含完整方法身份/质量/来源/诊断。旧双JDK矩阵只因**同一生成Java字节**而复用，未重跑或冒称新JDK运行。另12 API录入两个编译器×三旧复制控制×default/all，整Java与方法质量/fallback/map/诊断模式一致；原参数目标、guardIfFirst和MultiCopy仍fallback，未将负控冒称全BCI。root独立脚本核全部raw/input/report/source/CLI及前后source/external闭包。341成员/2486800B/SHA256 `f1d6555a8daad531f984a8410df244c93af6ae86c356c3f546a206656599caf2`。

父任务6/8；新产品尚待两轮fresh native v3与自身CI。两个历史failed seed及cancelled CI保持失败，不能用focused/broad/API重放替代完整工作区验收。
