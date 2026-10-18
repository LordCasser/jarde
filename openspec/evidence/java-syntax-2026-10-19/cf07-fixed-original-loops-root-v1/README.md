# CF07 四个固定完整 Java 用例验收

本轮用产品 ef4fbef5d9443a2d7e835712f81a22bbfed5cd50 的最终 CLI（SHA256 `96e534b0000fe91c1d34a282919479a97eb4706063501d188cdff010c4640366`）对完整上游文件编译出的嵌套目标分别采集原始、JADX、Jarde，再完整重编生成源码并执行。root 已独立重开来源、原始流、完整物理方法/字段/Code/BCI、UTF-8 map、编译器/VM 网格和 parentless CodeSource 负控制。

| 固定目标 | 有限场景 | 原始 VM/观察 | Jarde 成功整类编译/VM/观察 | JADX 成功编译/VM/观察 |
| --- | ---: | ---: | ---: | ---: |
| TestComplexWhileLoop | 7 | 4/28 | 16/32/224 | 2/4/28 |
| TestLoopCondition5 Java test0 | 10 | 4/40 | 16/32/320 | 2/4/40 |
| TestLoopCondition2 | 2 | 4/8 | 16/32/64 | 2/4/8 |
| TestLoopDetection2 | 5 | 4/20 | 16/32/160 | 2/4/20 |

共380条实际采集命令。Jarde 64次整类编译、128VM、768条场景观察全同原始；JADX 8次成功编译、16VM、96条观察全同原始，另8次JDK8框架major55编译失败保留且不运行；原始16VM/96条观察另计。default/all完整文本、逐方法raw/map一致，物理来源闭合。root复核现有短路头、计数循环、数组读取、条件赋值与终端return所有权已足够，无新生产机制或OpenSpec实现任务。

完整外层Java文件未经改写并参与编译；没有运行外层JUnit断言，Condition5的Smali test1也未运行。Complex只用有限输入，不声称wrap后有限退出。Jarde使用分离声明和i=i+1等规范化拼写，不冒称所有JADX精确文本断言通过。旧2026-09-27 LoopCases证据仍是组合fixture，不改称完整原始类。本轮不增加CF07/CF11/71单元/612文件的整项完成数。

`capture.tar.gz` 包含1248份证据及一个嵌入库存（共1249成员）；root逐成员与现场原始字节重开，SHA256 `8f7352d804ac30dbc0dca4473512fb329989ac0c9159556b2f673d83bc983111`。来源清单见`capture.tar.gz.inventory.json`，root复开见`capture-review.json`。包含全部原始/生成源码与class、20+75命令收据及raw、driver/helper、独立审计、历史静态退回版本和许可；CLI/JDK/cache/target不入包。tar内的`private-tmp/.../check_generated.py`为函数适配，重放时须从同包保留的最终v9 `review_final_api.py`导入helpers，并设置对应INTERNAL；适配diff只改变driver与场景数。
