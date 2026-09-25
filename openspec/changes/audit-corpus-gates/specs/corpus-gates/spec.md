## Purpose

为仓库 class 夹具和 P5 输入建立可复核的维护契约，使 reader census 与内容指纹都对应经审查的实际语料，而不是未解释的目录变化。

## ADDED Requirements

### Requirement: 审核后才能更新语料门禁

维护者 MUST 在改变 class fixture census 或 P5 fingerprint 前，逐项审查新增、缺失或变更的语料文件，并能说明其来源、用途和保留/移除决定。门禁中的数量与指纹文件 MUST 描述审查后的语料集合；单独改计数或无审查地重生成 fingerprint 不满足本要求。

#### Scenario: 发现未登记输入
- **WHEN** fingerprint 检查报告 corpus 中有未登记文件
- **THEN** 维护者先将文件按意图保留的夹具、可证明的临时产物或待判定项分类，并记录证据，再决定更新或清理

#### Scenario: class census 发生变化
- **WHEN** reader 夹具扫描结果与已记录 census 不同
- **THEN** 维护者先确认新增 class 的来源及其是否属于永久输入，再更新并验证 census 与相关结构计数

#### Scenario: 完成语料审核
- **WHEN** 所有差异均有明确保留或移除决定
- **THEN** 更新后的 fingerprint 必须与源码分类表一致，reader class census 必须反映最终 class 集合，且两项门禁的定向测试均通过

### Requirement: 保留门禁失败的可重放入口

维护记录 MUST 给出能重现 census 和 fingerprint 差异的定向命令，并说明哪一项门禁失败；维护者 MUST 在最终修改后重新运行这些命令验证。

#### Scenario: 复现当前差异
- **WHEN** 维护者开始处理本次 164/310 与 fingerprint 差异
- **THEN** 可运行 `cargo test -p jarde-reader --lib` 与 `cargo test --test p5_corpus_fingerprint --locked -- --nocapture` 定位失败
