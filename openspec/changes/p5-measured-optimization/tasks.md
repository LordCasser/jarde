## 1. Measurement baseline

- [ ] 1.1 固定版本/编译器/打包/身份/字节码/恢复/退化/对抗语料 fingerprint，并验证 A01–A18 覆盖清单
- [ ] 1.2 建立冷/热、局部/全范围、direct/cache/parallel 的重复 benchmark，记录耗时、读取字节、物化范围、内存代理和取消状态
- [ ] 1.3 定义结果 fingerprint、稳定排序、coverage/diagnostic 对照报告，验证 A15/A16

## 2. Data reuse candidates

- [ ] 2.1 基于测量选择是否需要多查询合并或细粒度并行，并验证 single-flight、取消和稳定发布满足 A14/A15
- [ ] 2.2 基于测量选择 cache/index 范围与 key 维度；在未证明收益前保持 disabled，并验证 A01 candidate 不直接成为事实
- [ ] 2.3 实现 cache 损坏、依赖补齐、profile/output/registry 变化的失效和直接路径回退，覆盖 facts-cache spec

## 3. Release gates

- [ ] 3.1 对每项启用候选执行 optimized/direct evidence、coverage、representation、diagnostic 和资源差分，验证 A13/A14/A18
- [ ] 3.2 在 ZIP bomb、condy 图、不可约 CFG、缺失依赖、取消压力和被选目标阶段的共享不变量语料上运行回归，任何语义差异阻断默认启用
- [ ] 3.3 发布实测范围、重复策略、未决阈值和启用开关；不添加无证据性能数字或未评估依赖
- [ ] 3.4 执行 cargo fmt、clippy、test、benchmark smoke 与 OpenSpec strict validation，记录本阶段仍未改变结果契约
