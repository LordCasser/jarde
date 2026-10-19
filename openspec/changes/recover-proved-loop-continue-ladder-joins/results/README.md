# 循环内局部汇合与直接 continue

当前5/6：原始完整基线与同轮IR、Luna实现经root架构审查、公开API/Reader/指纹、最终完整对照均已接受。复用既有局部join/LoopContinue/完整性证明，无新Region或全局pass；Builder减法窄依赖独立列项。

- [原始46命令基线](../../../evidence/java-syntax-2026-10-19/cf09-testcontinueinloop-original-root-v1/README.md)
- [真实IR及首gap](same-run-ir-root-v1/README.md)
- [定向与完整源码对照](focused-and-full-comparison-root-v1/README.md)：8 API、16整类编译、32VM/256观察、44/44 BCI maps。全部失败与修正保留。

待两片共享一次fresh workspace、当前产品自身CI、Git/CLI/源码绑定、主spec同步和clean-main交付。CF09/71/612仍部分覆盖。
