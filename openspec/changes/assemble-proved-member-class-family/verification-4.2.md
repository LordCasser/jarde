# 4.2 派生来源与物理身份复核

家族投影现在在根报告的 `member_family.projection.derived` 中记录根源码文本的字节区间及完整物理锚点。已证明的 `new Member` 绑定 caller 的分配与构造调用 BCI、child 构造器隐式首参；`Outer.this` 绑定 child 读取 BCI、捕获字段及构造写入 BCI。被隐藏的字段、隐式构造参数和写入也各自有派生条目，指向成员声明或构造器头。每个锚点携带物理定义/成员身份，根和 child 的相同 BCI 不会合并。原根/child 方法的恢复报告、覆盖和各自 source map 未被替换。

投影重新请求方法 source map，先要求目标分配 BCI 与构造调用 BCI、或目标捕获读取 BCI 精确落在同一已证恢复节点，再取唯一最短的源码词元。writer 按方法 artifact 和内层类缩进逐层换算位置，并逐字节核对最终根文本。任何歧义或换算失败都拒绝整次家族投影；文本与派生表一起发布，不留下无来源的局部改写。

Root 用独立 Cargo target 复跑 `cargo test --locked -p jarde --test member_family_identity`（14/14）、`cargo test --locked -p jarde --lib class_source`（22/22）、`cargo test --locked -p jarde --test class_source`（47/47）和 `cargo test --locked -p jarde-cli --test class_source_cli`（17/17），全部通过。测试检查每条区间的 UTF-8 边界、原文切片、完整 owner/BCI，默认和 all 两种证据模式的文本/派生区间一致；根和 child 各自的构造器 BCI 0 仍映射到不同方法，序列化 JSON 中各有自己的 source map。同一 caller 连续两次 `new Member` 的 Java 8 编译样本产生不同区间。拒绝与预算停止样本的 JSON 中没有伪造的 `derived`。`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate assemble-proved-member-class-family --strict` 通过。

本记录只验收 4.2。外部物理消费者闭包（4.3）和最终整体验收（5.1）仍待完成。
