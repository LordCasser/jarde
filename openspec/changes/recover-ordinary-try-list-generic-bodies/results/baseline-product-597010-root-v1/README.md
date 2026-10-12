# 普通 try + 泛型 List 的实际完整类基线

产品 `597010a8cb9992eead8276b2f2a67186ada393d9`；这是本项实施前的观察，不是泛型恢复完成，也不代替该产品自身CI验收。

debug/none各27命令、54raw；每份分别用JDK8/23编译原始完整17方法类，再对default/all Jarde整类输出编译运行。8条Jarde整类运行的44行输出逐字同原class，包含旧17行oracle与新增27行分支/null/异常/嵌套/remove控制。每个JDK以自身编译的class作为JADX default/rename-none输入；8条JADX整类编译真实失败，保留源码和stderr，全部no-runtime。

`handled`、`tryStringBranch`、`tryIntegerBranch`、`tryStringRemovePrefix`、`tryStringNested`已有完整正文但泛型仍拒绝。`tryStringUnknownCall`与rawAliasEscape/重赋值/wildcard/indexOverload等是拒绝控制，不能为了签名恢复放宽用途闭合。

root实际重放独立auditor-v2，状态audited/errors空，接受文件SHA `6cf0567b6d31cf3b63d95c0d8ad7b03c7d85a4db29fec5bb72a60db87d211048`。物理17方法与Code长度来自独立class解析，真实render stdout与保存report JSON匹配，default/all声明/正文/来源一致；所有已给出的BCI均为真实指令边界，物理owner正确。存在未映射的纯goto：handled@44、String/IntegerBranch@40/60/79、removePrefix@55、nested@90、unknownCall@40。不宣称全物理指令映射覆盖，该债务独立处理。

[capture.tar.gz](capture.tar.gz)为2004769 bytes，SHA `3738e0b0725f67f4aea9c4bfab12a74d58b6a111734e9dddb7b4413430c8448a`，276成员。root独立逐成员复开、核大小/SHA与embedded manifest，108份基线raw和额外focused-build/final-clean raw共134条绑定均完整。完整源码/Runner、输入与输出class、JADX/Jarde sources/reports/maps、原始receipt与审计/采集/parser/guard工具都在包内；4860产品source closure保存完整metadata，冻结CLI/JDK/JADX安装二进制仅保留pin引用。

debug receipt SHA `f9b25f4f93c98c795618590ec75041dd827bd058f344e1c1e8a16dfc92b271e3`；none `b94ca14e7940a4eb4a964172337393e496ed061195e26af2a8eceb79c53d258b`。final exact cargo clean实际0、target前后不存在，receipt SHA `f58d128bdb6ab593bff76e33d1df6380aa2e63b658a4bd98306c32524252cf31`。resource guard无触发，source/tool before-after一致。采集准备曾因source-only manifest缺bytes而在执行工具链前失败；[collector-preflight-review.json](collector-preflight-review.json)保留原失败与只补两项实际bytes/SHA的修正，未降低行为门。

[acceptance.json](acceptance.json)、[root-archive-review.json](root-archive-review.json)及[capture-manifest.json](capture-manifest.json)记录真实绑定。第一次root复开误以全部jar为安装二进制而停止；第二次独立解析四个受控input jar，仅含一个原class且与同模式同JDK编译class逐字相同，安装二进制按实际origin继续排除。归档、运行输出与审计门均未修改。只有任务1.1可据此关闭；其余实现/最终产品验收及整CF11/71/612计数保持开放。
