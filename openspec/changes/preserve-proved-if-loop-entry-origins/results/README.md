# if臂进入嵌套循环的来源：5/6，待完整产品验收

只补普通If父节点的派生来源：单块单Transfer、紧邻实际Loop header、非clone、完整出边唯一且Normal。复用既有edge检查与OriginSet，Java文本/Region/物理owner/Loop.gateway_origins及公共协议不改；不增加语法机制。root审查去掉了初稿未证明的CompoundIf扩展。

最终定向CLI `56ed2f31f824187b178082d1045788e5ba09ea8f9ab4e1fe6bffeca37db8efa8`，63956112B，冻结于 `/private/tmp/jarde-if-loop-entry-focused-root-v4-20261019/candidate-cli`。实际四API为原class/JAR×default/all：均10/10来源，完整846B Java逐字同原接受基线，只有父If(primary5)新增唯一derived8、保留derived17；去掉8后整map同基线，完整JSON仅忽略usage和这一map差异后相同。最终v4 CLI与实际执行四API的v3 CLI逐字相同。绑定复用原八次完整Jarde源码编译成功及JADX两次unreachable编译失败，不称重编或运行等价；没有执行目标方法。

定向实际通过公共2/0、Java lib508/0、Reader1/0、fingerprint5/0/1既有ignored、fmt、工作流原Clippy完整argv及最终CLI构建。新公共停止测试从同一snapshot经真实MethodAnalysisRequest恢复，按成功ir_edges用量有限扫描，只接受BCI8新证明处的真实IrEdges停止；Partial/NotProduced、空正文/maps/regions、无declaration/artifact、实际usage保留均通过。真实IR原goto与goto_w正向、含iinc前缀拒绝及边/来源计费停止通过。clone/不相邻/额外或异常出口由显式窄代码门和架构审查验证，未声称为这些条件各造了动态负例；旧exception-if-join回归不是新helper的直接异常边负控。

永久12文件树由两parser与branch目标核原件及两构造控制；实际Reader新增3class/3Code/12targets，期望1320/5830/631/4183/16。指纹由明确ignored regeneration成功生成。初次public compile的policy moved、随后unit预取消计费错误期望均已修正，生产逻辑不为测试改变。所有实际失败raw保留；v3/v4最终source冻结且target finally clean，峰值分别680341365B/633909298B。

[定向、四API及审查归档](focused-and-four-api-root-v1/README.md)包含100 payload和内嵌库存，root逐字复开bytes/mode；archive SHA `42b7413819fc41c6c6a6139be218128d12a3ed87749aab263e74641d8764e0d9`。原始20命令证据仍见[真实Raung基线](../../../evidence/java-syntax-2026-10-19/cf08-looprestore2-original-root-v1/README.md)。最终fresh全workspace、本产品自身CI/双seed/继承具名门、源码Git绑定、主spec同步及strict、clean-main尚待，不用前片3fa CI代替。

CF08/71/612整体仍partial；finThrow5/5、nestedFin16/16已交付不重做。并行CF09固定五原始类审计确认旧证据是构造组合输入，下一先补TestBreakWithLabel完整原始类对照，尚无新缺陷结论或实现spec。
