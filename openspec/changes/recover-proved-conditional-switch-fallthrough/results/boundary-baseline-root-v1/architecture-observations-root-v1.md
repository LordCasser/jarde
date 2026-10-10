# 实际观察与架构判断

完整类双 JDK 原输出一致；JADX 1.5.6 双腿各五行差异：partialBreak(selector=0,flag=0) FB→F；innerLoopBreak 两旗 LLON→LON、LON→LOON；caughtExceptionThenFallthrough 两旗 7N→7、CN→C。innerSwitchBreak 和 terminalCase 的实际 Runner 行保持。Jarde 当前六成员中 constructor、terminalCase 为 structured，其余四方法 fallback；四份完整类均缺 return，未运行。以上只接受完整观察，不能由方法文本声称整类运行通过。

本地 JADX 的 addCases 用 dominance frontier 交集判断下一 case，插入 break 在 try/catch wrap 后按 case block 集和末尾 successor 判断，common-break pass 还按 Region 分支/退出标记改写。参考其目标作用域 RegionRefAttr 和 case 顺序检查很有价值；但不能照搬“case 外 successor 即 break”：下一 case 是合法穿透目标，只有准确 switch join 才是该路径退出目标。运行版 1.5.6 与本地 checkout 的 revision 同一性未证明，不能将观察直接归因于本地某一行，也未跑 JADX 单 pass trace。

Jarde 现有 group fall_through 表示整体末尾是否自然进入下一标签，Frame case/join boundary 只限制块所有权；二者不能表达一个条件分支到 join、另一个分支到下一 case。此片必要补充是带物理 source_bci、当前 switch branch_bci 与 canonical join 身份的零块 SwitchBreak 叶，Builder 仅在最近 switch 且 loop/switch depth 未跨越时消费现有无 label Break。证明必须先核全 canonical incident edges 和 incoming 闭合，不能只看隐藏异常边的 NormalFlowView 或生成后修 AST。

本片仍仅有限无环、单相邻 case 出口。循环、异常组合、嵌套 switch 非 comparison 分支是独立恢复债务，不能在当前片为改善组合类而放宽证书。真实图、实际拒绝位置和新增叶的完整来源仍须在应用后验证。现有 CanonicalCfg/CanonicalEdge 构造字段对 Java 层不可写；人工非法重复边/clone 等合成反例目前不能直接调用完整 Walker 入口，不能拿一个 scalar predicate 测试代替。下一步优先真实 class 验证可表达路径；尚未决定增加 graph 测试 seam，未修改公开 API。
