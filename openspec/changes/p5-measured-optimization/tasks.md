## 1. Measurement baseline

- [x] 1.1 固定版本/编译器/打包/身份/字节码/恢复/退化/对抗语料 fingerprint，并验证 A01–A18 覆盖清单（2026-09-20 完成，提交 `6d08124`。**落点判断**：fingerprint 属测试语料清单 → 写进仓库（`tests/fixtures/corpus-fingerprint.json` + `tests/p5_corpus_fingerprint.rs` + fixtures README 一节）；**A01–A18 的判定属验收权威陈述 → 未写 `openspec/**`，完整清单已由父级落进本 change 的 `verification.md`**。**fingerprint**：93 文件（fixtures 77 + fuzz/corpus 16）、**八维**（任务原文的七维 + `编译器` 单列）、每文件 `path/bytes/blake3`，与 `git ls-files` 范围内集合**逐条相等**；**按范围收录而非名单**（新 fixture 落地即覆盖），README 与生成脚本按扩展名排除（**记录不是输入**）。**先探明避免重造**：既有 fixtures README 的 SHA-256 表**从未被任何测试校验**（仓库无 sha2 实现，**表是散文**），golden 的 45 条 blake3 回放**已构成一部分**（本片**只索引不重复**），fuzz 种子此前只有 10 条**尺寸**（本片为 16 条加摘要）。**维度状态如实**：版本/编译器/字节码/恢复/退化 **pinned**；打包/身份/对抗 **partial**（WAR/Boot/ZIP64 只在 builder、proptest 逐次生成）；`known_gaps` **五条**如实列出**未凑**。**父级独立复核**：校验 5 passed / 1 ignored（0.03 s、无 JDK/网络）；JSON = 93 files / 8 dims / 18 rows / 5 gaps；**翻转一个语料字节 → FAILED 并打印两个摘要，还原后复绿**。**A01–A18 逐行清单**（本片核心产出，已落进 verification.md）：**16 行现已通过**（A02 除本机未跑的 JDK 25 oracle）、**A15 未通过**（P5 自身未开始；仓库**不存在任何 cache/并行实现**）、**A18 部分通过**（P1 半已通过，缓存/并行半属 2.x）；A04/A12 的「现代恢复」增量**未发生**（与 P4 3.4 一致）。**证据**：1082 passed / 0 failed / 4 ignored（+5 passed、+1 ignored）；fmt/clippy 1.98.1 干净；**MSRV 1.88.0 check 通过**；openspec strict 16；**被修正的既有断言 0 条**；锁文件两条 exit 0。**未完成**：1.2/1.3/2.x/3.x 各节；`known_gaps` 五条未闭合）
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
