## 1. 定义读取的身份与保留

- [ ] 1.1 定义读取身份：不可变 snapshot、完整物理位置（origin chain + ordinal）、内容 digest、声明 length、变体（multi-release 选择结果）、读取 schema 版本、已完成校验（CRC/size/span）。验证：每一维单独变异都不命中（判别性用例逐维），跨 snapshot 同坐标同 digest 不命中。
- [ ] 1.2 只发布完整读取：部分构造、被取消、被限额终止、校验未完成的读取 MUST NOT 进入可复用集合。验证：在每个终止点各有一个用例，且终止后保留集合不增长。
- [ ] 1.3 与 container/CP/Header 层共用 entry 与 retained-byte 两个上限，满则拒绝、不淘汰；报告命中、驻留权重与容量拒绝。验证：容量梯度（none / e1 / tiny / 中等 / roomy）下结果与计数；拒绝后当前请求以直读完成。

## 2. 消费点接入

- [ ] 2.1 `src/facade.rs` 的身份绑定读取与 `crates/jarde-jvm/src/providers.rs::read_definition_class`（方法驱动与 bulk class task）经快照侧复用入口；**名字搜索的候选读取不入此路径**。验证：三处各自的计数门禁 + 名字路径的读取计数不变。
- [ ] 2.2 命中不计入 `archive_entries`/`read_bytes`/`entry_bytes`/`class_bytes`，不重放旧 usage、不重置预算。验证：命中/未命中两臂的 usage 对照；已耗尽预算下命中不恢复。

## 3. 语义等价与回退

- [ ] 3.1 两个足额路径（关闭 / 启用）的逐方法 fingerprint（去 `usage` 与 `elapsed_millis`）相同；解析、准备、分析步数不变（只有被停止重复的读取消失）。验证：等价用例 + §3 实验的 24/24 指纹作为参照。
- [ ] 3.2 回退逐条演练：无 store、容量拒绝、身份不符、取消/到期耗尽、异声明条目——各自走直读且与关闭时逐项相同。验证：五条各自一个用例。
- [ ] 3.3 跨 crate 入口的分层检查：`reader` 只提供事实、`jvm` 只消费；源码守卫与既有分层门禁不变。验证：`tests/p5_benchmark.rs` 相关守卫 + clippy。

## 4. 紧预算与内存

- [ ] 4.1 紧预算冷/热对照：命中可能改变完成度，验证**不伪造 Complete**、停止原因如实（这是 §3 的未确认项 ②）。验证：受控紧预算用例 × 关闭/启用两臂。
- [ ] 4.2 驻留与容量退化：驻留增量与被复用定义的大小一致；RSS 与 retained_bytes 分开报告，不用保留权重冒充 RSS。验证：容量梯度下的驻留读数 + 5 档 RSS。

## 5. 门禁与口径

- [ ] 5.1 `cargo fmt`、`clippy -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全绿；OpenSpec strict 通过。
- [ ] 5.2 主张口径：只作**工作量**主张（窗口份额上界 0.5%，5% 需 13.8 ms）；任何时间主张必须 ≥10 次交错且写在证据里。验证：verification 里三层分列。

## Acceptance Map

| ID | 主题 | 主要外部判据 |
| --- | --- | --- |
| R01 | 身份严格 | 逐维变异与跨 snapshot 用例都不命中 |
| R02 | 只复用完整读取 | 每个终止点后保留集合不增长 |
| R03 | 语义等价 | 两臂 fingerprint 相同、准备/分析计数不变 |
| R04 | 回退完整 | 五条回退路径逐条与关闭时相同 |
| R05 | 不伪造预算与完成度 | 紧预算与已终止请求的停止原因如实 |
