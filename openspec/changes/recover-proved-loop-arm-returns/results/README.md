# 当前证据与任务状态

当前5/8，原始/补充完整基线与旧矩阵、真实首拒以及最小实现的定向测试已root接受；完整API/生成源码矩阵、广泛门、两fresh workspace及自身CI尚待。固定71/612与CF-11整体计数不变。

| 归档 | 实际范围 | 成员 / bytes / SHA256 |
| --- | --- | --- |
| first-refusal-root-v1 | 11外层命令含八API+guarded build/clean；完整canonical/normal/SSA/effects、ForHeader/terminal/gateway；非usage JSON恒同、源码完全恢复、target absent，root独立接受；未执行错误草稿单列 | 43 / 1257613 / 564087ec09f53b39ded089fec47ff83c8a1c3cecc1ff46e0a9ba80fd199075b3 |
| original-baseline-matrix-root-v1 | 42命令/84raw，24原样全源码编译；Jarde8javac23缺返回/8javac8框架边界，JADX4javac23成功、8core双VM24case+4check23case同原、4check8边界；root独立review通过 | 112 / 490333 / 7ed8bfb27eb2bfe77342d30f8ed1bb4e172575df49ad7a79040794ef6c79a3e8 |
| original-baseline-root-v3 | 28命令/56raw，完整原始源、outer+child debug/none、8API全3方法/全物理BCI、4core双VM12case同原、2check23通过/2check8 major55边界；另保留早期8命令16raw的collector失败，不能并入成功 | 129 / 868355 / 7700e8fbb3f27bccfaf8be73cd6e03f33c753487059369a7ce36f0044f376a4c |

三归档284成员由root独立逐字重开，见baseline-archive-review-root.json；所有payload bytes/SHA匹配，未执行草稿与失败只按原范围解释。CLI二进制不入包，精确pin保留；JDK/依赖按原路径SHA绑定，未宣称随包提供。

首拒实际到达loop_arm_join_source并返回None。日志已证明child16完整Loop、return29、normal join37、ForHeader15/31/slot3、gateway34；对closed_body首个谓词及后续natural-only/terminal/latch契约的说明是root对同一AST与当前源码的架构推导，不能说成另取了helper内部逐谓词日志。

补充A/B已root独立接受并归档：`supplemental-baseline-root-v1`为65命令/130raw、8target/16完整class、16原VM48case各同自身oracle、16Jarde run fallback/constructor structured、8JADX提取。32方法全部物理BCI覆盖且owner/UTF8/default-all合法。原审查草稿和两次实际checker拒绝原因单列，不称产品失败。

`supplemental-baseline-matrix-root-v1`为97命令/194raw、48完整源码原样编译：Jarde32全部缺return，JADX16成功、32双VM96case各同原oracle、16错误CodeSource负控拒绝。全部2方法/5字段、actual Code及source/class稳定/最小CP逐格核，失败不算runtime。task1.3已完整关闭。

五归档均由root独立逐成员复开，最新总成员及SHA见baseline-archive-review-root.json；前三归档284为历史分项，非最新总数。共享fixture/公开测试复制和旧生产RED随后执行，尚不称实现通过。


## 当前实现与定向验收

生产修改仅位于`loop_arm_join_source`：所有者必须精确为natural与同Frame、同normal join的已证明return叶之并；消费原有ForHeader/latch并逐字核gateway，保留完整canonical边闭合及停止传播。`implicit_tail_latch_origin`保持旧产品原样。实际完整树确认原始tail If/gateway34、A/B tail Straight/gateway96，因此删除未被证据需要的CompoundIf扩展。

`focused-root-v8`已root独立接受：479 Java库（含7新内部门）、3新公开、4旧循环臂控制、181 Reader，共667通过出现/0失败。公开门核10实际class输入的完整方法、BLAKE3物理身份、全BCI/UTF8、SourceMap-only/all正文/map及真实late-budget无部分输出。CLI63070624B/SHA256 `09dda40069dc362bc59f141a7d82c8cdcaee8b64072d304ec1ba848bdc14da25`仅为候选构建。22完整class/38Code/2handler/74targets/0jsr与Reader实测增量一致，census1280/5677/626/3921/12。

有效异常边界归档`exception-boundary-root-v1`保存独立源、双编译器major52、双VM四输入及错误CodeSource控制。它证明合法class输入及真实canonical Exception入射消费者拒绝；测试中的Straight区域是显式proof-context，不冒称该形状完整Java恢复。其他共享叶/额外入口出口是Reader分支字节变形配原SSA/Region上下文，foreign-jsr是实际clone ID作为异域上下文，均不冒称有效完整JVM正例。

`earlier-trials-root-v1`保留七轮准备/实现测试试跑的真实失败与源码限制；公开旧生产RED三测试质量均失败，单列`public-red-root-v1`。库essential默认无map的测试假设已改为显式SourceMap，不将准备断言错误称为产品问题。补充旧map checker的边界曾用整类文字，已在`supplemental-map-scope-root-v1`独立重核32方法全部局部report.text坐标和BCI。

广门v1因新测试5处field_reassign_with_default失败；标准指纹再生成功，旧2356项逐项不变，仅新增本项31输入，总2387。广门v2已通过CI原样Clippy，但Homebrew cargo不支持`+1.88.0`，尚未开始MSRV项目编译；改用已安装rustup入口后继续，其失败不倒填通过。两轮均exact clean。后续结果须按独立收据追加。

当前11归档879成员已root重新逐字审计，见`all-archives-root-review-v1.json`；原五基线729成员的`baseline-archive-review-root.json`保持冻结，供最终API绑定。此总审计不包括之后新增归档。

本地组合广门现已root接受：v2 CI同款Clippy；v3 rustup绝对入口MSRV1.88及479库。v3第五P5未启all-features导致0tests，runner拒绝，故v4重新以all-features执行全部八P5，实际104/0/5并构建CLI成功，与focused-v8逐字同；当前3134来源/raw/预算守卫及每轮exact clean已独立核，见gates-final-root-v4。两fresh seed及自身CI仍待，不提前关闭3.2。
