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
