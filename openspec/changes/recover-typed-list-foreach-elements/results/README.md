# 原始 TestNestedLoops 与 typed foreach 实施入口

root已实际独立接受未修产品ee013/aabf CLI，当前Luna候选已应用并通过root focused-v5。OpenSpec任务1.1、2.1、2.2、2.3、3.1完成，当前5/7；最终广门及20API重绑已通过，两fresh native与自身产品CI待验收；不增加CF11或71/612整体完成数。

| 证据 | 实际范围 | 归档 |
| --- | --- | --- |
| original-baseline-root-v1 | 20命令40raw；完整原始outer+child debug/none，4原JVM/32case，8Jarde API，2JADX完整输出 | 77成员 / SHA256 823669b76a4ad3f63e42ae0d063a1c697ef88e970be860e8278ac063b5931a11 |
| generated-baseline-root-v1 | 75命令150raw；20原样编译/18成功，36 rebuilt JVM/288case同原，18wrong CodeSource拒绝 | 190成员 / SHA256 49c44b3fcf63375e0beec9f9f9e8c243b78dbbece2c84dae66256027990de446 |
| baseline-tools-root-v1 | 原始/矩阵collector与review各版本、root修正、源码pins、JADX/事务/外层绑定/规划的静态审查 | 40成员 / SHA256 6846f5860127f492a3da24ebfa6c777a55f7a3acb2a22205d81325bbbb576fa7 |

三包307成员已按相邻capture-review及embedded manifest逐字复开，见[归档审计](baseline-archives-root-review-v1.json)。仅观察和规划，不冒称新产品CI或新实现。

上游完整876B source SHA256 `4897fd18a15fce6e0abb40dc8afc6c7a3fa9f16e617f3c7e5e88072f6a37309e` 未裁剪。完整原源javac8遇IntegrationTest major55/expected52；javac23 --release8编译debug/none为major52。每profile完整2class，child ctor/test两Code、无字段，目标45BCI/119B，outer ctor/test两Code无字段。原始32case包含两CME、三NPE和正常/空列表后的实际列表状态，四profile/VM结果同，不硬编码oracle。parentless loader fresh每case、独立expected CodeSource、driver-only classpath；本Driver实际调用target构造器和实例test。

原始receipt SHA256 `4f48e3b14738cb4d3746763290788f525bba4bb1a1a3d47aebcf0111e6f8d482`；root审并实跑baseline audit root-v3接受。8API精确2方法Structured/无fallback、完整请求执行，两个List<String>形参但两个Object foreach/body String cast；所有物理BCI/UTF8/source owner都按每个raw RecoveryReport.text核，而非projected method/whole text。四组default/all在三个文本平面分别比较。BLAKE3只核same input身份/owner一致，未声称重算；输入物理SHA256另绑。

矩阵receipt SHA256 `46d5ff4f12d339d6fc2ddf9df963a256e25421b9f7c2b06eeec0cf5e08288012`。Jarde16完整选定child原样编译/32运行256case同原；JADX2完整outer+child编译/4运行32case同原，另2JADX javac8失败仍是原outer框架major55边界，无runtime。选定child完整源码编译不冒称整个outer源码编译；JADX保留outer完整源。20成功class/40Code均双parser独立重解析，完整class/method/field/access库存同原，物理指令向量/边界接受。未手修源、删方法或把原target混入runtime classpath。

最终matrix checker root-v4 SHA256 `9b434bb2a9badf72d11be77c0b642490b45b1c6f718495aa8d33a5595e6afbb2` 已实际执行接受。root-v3的真实driver tuple/JSON list比较拒绝保留；v4只规范JSON值表示，不改字节/顺序/Code。其他未执行草稿的schema/manifest/copy/负控契约错误及首次归档缺父目录错误在tools历史说明，不记作Java运行失败。3298来源/CLI/JDK/依赖/原源与全部生成source/class/driver前后及finally闭合不变，default stack/5GiB/target1GiB守卫接受，target不存在。

JADX实际两个String头与当前Object头构成呈现差距；不是旧iterator CFG拒绝，更不是当前语义错误。最终design保留合成header名和原body绑定，将精确受证cast吸收入typed头，不新增作用域solver。原名s1/s2头折叠、冗余iterator声明、equals/remove Object包装仍独立，不能声称此片将通过整条原上游文本黄金。

当前定向验收见handoff首节：496 Java库+17公开/相邻+3 iterator共516通过/0失败/1既有忽略，默认stack、guard与exact clean通过，CLI d1304ad…；最终原始类/控制家族矩阵已独立接受，广门/native/自身CI尚未验收。控制旧版完整基线与真实JADX问题见[19case基线](controls-baseline-root-v1/README.md)和[启动补跑](controls-jadx-java23-supplement-root-v1/README.md)。

## 最终候选完整对照（工程广门进行中）

CLI 为63386112B / SHA256 `d1304ad2c3acca940a744eb3023bea0f77f62d279afc675aff84a6c076d447c4`。完整原始 TestNestedLoops 最终收据 `433b6a723137adb4aed904b2c7935079821def5b1b91f1593630c224bc73f439`，20命令、4原JVM/32case、8API/2JADX；两个头为String，原s1/s2（无debug local4/local6）声明/绑定保留。相对未修projected方法，仅头类型和首RHS cast改变；raw方法text/map/quality/fallbacks逐字不变。完整生成矩阵收据 `1d171a5ad5a3ef83fd72bde289ae67977ae9fe0cf36a622c9b4e576dbf8d1149`，75命令、20完整编译18成功、36JVM/288case同原、18wrong-root拒绝；两JADX javac8 outer框架边界失败保留，无伪造runtime。

控制最终收据137命令、4原profile/8JVM/152case、12API、4次JADX均在JDK23启动成功。Jarde 24完整编译、48JVM/912case逐字同原、24生成及4原wrong-root均拒绝；strings/integers/declared三个头类型化，原绑定与其它语句保留。raw/mismatch/ctor的projected text不变，全部6方法的raw text/map/quality/fallbacks不变。控制正例首次进入既有AST emitter，方法body相对旧fallback emission每行多12空格；root checker v1因此真实拒绝，v2精确核这一个固定缩进变化并保留所有其它字节，未任意strip或改源。JADX完整类在合法 `List<String>` + raw Iterator + Integer cast上错误输出Iterator<String>，8完整生成编译均实际String→Integer失败；未删方法、手修源码或声称JADX运行成功。

| 最终归档 | 成员 | SHA256 |
| --- | ---: | --- |
| original-final-root-v1 | 77 | 610ee3cda882edb831b0611b541fdfc457d65990b23625b715ad5a7da27311db |
| generated-final-root-v1 | 190 | 47f3335f5b51da53c24c166748ed8c65895b83b15f0ca8cea66608e0632bec1d |
| controls-final-root-v1 | 381 | 62fc3e03ea91308833b7b72ea0f56c14314375e24319fb3b2781bcfd44c31c08 |

三个final归档648成员与两个控制旧版归档397成员已由独立Luna只读逐成员复开，见final-archives-review-v1.json及controls-baseline-archives-review-v1.json。完整原始/矩阵/控制采集由root串行运行，源闭包3301项、CLI、JDK、原始与生成产物/工具pins前后及finally不变，默认stack/5GiB/target1GiB守卫通过；独立结构/源码/运行审阅已接受，归档完整性审计不替代产品验收。

## 广门v1和最后对抗审查

广门v1已实际终态通过并root独立接受：18命令含exact clean、802通过/0失败/6忽略，fmt/CI Clippy/MSRV1.88、Java496、Reader181、公开17+3、八P5含指纹regenerator。Reader实际1301class/5751Code/630handler/4050targets/14jsr；标准指纹2427→2429，仅新增控制Java/class，全部旧项不变；README不在指纹范围。CLI与focused-v5逐字同d130；目标清空。

该轮仍不是最终产品交付：Luna最后只读对抗审查指出，binding/cast的derived-only foreign来源需同primary拒绝，final计划比较应计入可变owner identity字节，新typed遍历不应在计费前调用program_node_count。root核实后派私有最小修正；旧charge_class_source_ast本身的既有node-count预扫属于独立架构债务，记录在此，不趁本片全局重构。修正后重跑必要门，并明确绑定新CLI后的Java对照，不冒称旧矩阵直接覆盖修改后的binary。

最后来源/计费修正已应用并fmt完成，工程广门v2正在执行。v3补丁root去掉重复header_origin字段/clone，直接复用foreign_origin_bcis非空拒绝；有真实typed edit时才核final identity工作；first-binding/cast来源在inventory clone前先受计费验证，明确foreign不复制；三个derived-only内部拒绝负控已补入。只提高本片新AST成功用测试预算到100000，旧fresh、生产与所有P5阈值不改。原已验Java矩阵绑定的是d130候选；task3.1暂退回未勾，待hardened CLI的20同argv API逐字对应后重新绑定这些完整源码运行证据。不会声称修正后已重跑JDK。

## 最终 CLI 与完整矩阵的绑定

最终hardened CLI为63405424B / SHA256 `16bd83857cfbcc6023013a6fc67b79030472f5beb0a448baba0b9b6599c7d27f`。broad-v2实际18命令含clean终态exit0，802通过/0失败/6忽略，3190来源完全未变；fmt、CI Clippy、MSRV1.88、Java496、Reader181、公开17+3及八P5全部通过。最终独立review接受，见broad-final-root-v2（41成员，bc17a1550bca58882bdd145812ca7a6fd446c59f951142d21e9dec706ac716fa）。无新fixture变化或旧阈值调整。

root实际以新CLI重放20个旧argv请求（原始8+控制12），40raw，3301来源/全部冻结报告、class、源码、CLI及矩阵产物前后不变；独立review-v3逐格接受。递归只去usage键后其余JSON全部相等，包括limits.elapsed_millis、执行状态、物理身份、诊断、evidence、whole/projected/raw文本及全部map，不任意strip时间/来源。故精确相同的完整Java与输入绑定原始final生成矩阵36JVM/288case和controls final Jarde48JVM/912case；本次只重跑API，没有重跑JDK或声称新JADX runtime。原JADX 8个类型不匹配整类编译失败与2个outer框架javac8边界原样保留。

replay-v2在发出任何CLI前因full class facts含额外字段而误当纯pin拒绝；v3仅投影path/bytes/SHA后核pin，原物理事实仍由冻结收据SHA绑定。checker-v2在任何JSON比较前误把source_closure dict当pin list；v3核dict.values并保留完整live closure相等。两个工具失败均保留，没有转记为Java语义失败。hardened-api-replay-root-v1含实际收据、全20报告/源码/40raw、root-review与修正工具/失败记录。task3.1恢复勾选；两fresh native/实际产品自身CI仍未完成，不声称7/7。

## 独立bulk阻断修复后的最终候选

b78候选第一native实际在既有bulk生命周期门失败，未计成功seed。根因及最小guard位置修复另见 `align-bulk-class-task-execution-interval`，没有混入typed算法；旧失败保持。修复后CLI为63405424B/SHA256 `85ca6e7d36c0edeb72f7f6a3c33c405f203028fb21d128522f0b0a3a92e1cf78`，额外bulk16门+4精确重叠复跑及原全部广门共822/0/6通过，20同argv API重新接受全部非usage JSON/完整Java/raw maps逐字相等。对应独立change的broad-final-root-v4/final-api-replay-root-v2闭合源码3301/构建3190与全部产物，task3.1再次勾选；本片仍5/7，待两fresh native及新产品自身CI。
