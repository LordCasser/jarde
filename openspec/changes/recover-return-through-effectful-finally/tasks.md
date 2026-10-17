# Tasks

## 当前状态

当前产品30afa/冻结05e CLI已正常归档交付main 4be31d03b。父级实际矩阵与Q-i/Q-ii见instrumentation.md；原patrol保留历史，不再用旧fallback矩阵描述当前FY/FX/FO。当前新增父级材料为设计、物理来源及返回覆盖/主动抛异常红控制；没有生产代码改动或新的rebuilt运行。

## 1. 取证与基线

- [x] 1.1 复跑patrol完整FY/FX/FO并单列当前矩阵，不改历史README：最终24命令保留两编译器来源6完整类/32方法，全Java/Structured无fallback，6次原样重编0与12次rebuilt-only main逐字等原oracle；见leaf final-runtime-v1及独审。
- [x] 1.2 补齐instrumentation.md：历史Q-i对应loopless实际首拒16!=25与融合尾48/49，Q-ii分别对应noCall真实iinc/handler phi与CallReturn保存调用结果/cleanup异常链，区分transfer不可达handler与可抛调用；逐项引用已归档事实，登记新finally返回覆盖/主动抛异常红项。

## 2. 实现

- [x] 2.1 单return穿副作用finally叶片已交付：noCall产品02de294f7、CallReturn产品464543594及后续transfer产品30afa各有私有证明、真实负控制/停止、公库/CLI/完整源码与自身CI；复用既有Finally/SavedReturn，判据不放宽。父范围为FY.noCall/loopless与FX.finReturn，finally自身非局部完成保留拒绝。
- [x] 2.2 FO求值序判别测试入测试集（1/100）：recover_static_field_prefix_return三项永久测试与已归档完整源码/driver证明先保存再finally；历史target为3 passed，最终父矩阵再核FO两编译器/双JVM固定main。后续若测试入口改变，必须核实际selector覆盖这三项。
- [ ] 2.3 完成对照验收独审：emptyFin及28个非transfer方法文本相同，continue/break按授权去fallback并以原oracle/完整来源约束，未来固定30afa为完整基线；EX.finReturn相同fallback和FinallyCompletion三处missing return按具体形保留红项，不冒称健康finally-return对照。

## 3. 验证

- [ ] 3.1 汇总并独审FY/FX全锚与FO判别：两编译器全部物理方法/BCI/身份、完整原样重编、原始与rebuilt-only固定oracle、独立driver观察边界；JADX21命令须保留两次FX真实编译失败与0 FX rebuilt运行，不要求修工具输出制造可执行三方一致。
- [ ] 3.2 父最终源码/CLI/CI闭合：核29本地门（124pass appearances/120不同passing FQN）与30afa自身双seed CI每seed3680/0/98、MSRV、fmt/CI同参数Clippy、原P5/reader与两版strict，区分产品head和父文档head；父计划更新后再次strict并独审，符合范围后才同步/正常归档。
