# 枚举数组路径证明：独立复核

2026-09-24 使用当前工作树构建的 `jarde-cli`（SHA-256 `fda0dd30d638d8b80e5bde9fd430cf0a1f7fd92a0fecec8d91263868af6e8556`）。本次只验收枚举数组工厂/公开 `values()` 证明与 helper 表读取身份，不宣称 `project-proved-enum-switch-labels` 的全部预算、来源和语料任务完成。

- `negative/enum-values-array/replay.py` 在独立目录重放两个等长 Code 补丁。两份 `Hue.class` 均通过 `java -Xverify:all`：工厂 null 元素给出 `RED,BLUE,null` 与 `1,2,3`；公开 `values()` 返回 null 给出 `null` 与 `ExceptionInInitializerError`。直接枚举源码却给出 `RED,BLUE,GREEN` 与 `1,2,3`，所以仅凭字段名/ordinal 投影会改变可观察结果。
- `cargo test -p jarde-java --test class_initializer_candidates enum_identity_proof --locked`：2/2；`cargo test -p jarde --test class_source enum_switch_projection --locked`：2/2。普通映射、合法交换映射通过；字段 alias、数组工厂 null 元素和公开 `values()` 返回 null 被拒绝，保留整数表分派。
- `replay.py --cli <本次 CLI> --out <独立目录>` 重编原始 Java 8 class 并核对冻结哈希。普通和交换映射两组的原 class、JADX 与当前 Jarde 均成功重编并以 `-Xverify:all` 执行，逐行分别为 `1|1, 2|2, 3|3, null|0` 和 `2|2, 1|1, 3|3, null|0`；本次 Jarde 源码 SHA 分别为 `0bd05f46f5c8027cc6dcc911c7c97995758cb0b8ff6395fa1fc759b6296dd822`、`c96db4645035d966f8b529879a10d7ec0a42afe3d466f7a8a6713fb1f3ee62e0`。

代码核对：门面从 enum `<clinit>` 尾部实际 Callref 解析私有静态数组工厂，并分别读取其 IR 与公开 `values()` 的 IR；证明逐条要求工厂长度为 N、各 ordinal 位置装入对应常量、返回同一数组，以及公开方法仅读取已发布字段、克隆、同型转换并返回。helper 表候选前置检查现在以 `getstatic` 的真实 BCI 与候选比对，不再误用 `iaload` BCI。仍须单独收敛任务 1.2/1.3/2.2/3.3 余下的多写、依赖/预算、语料与全套静态检查；严格 Clippy 当前在 enum 旧表证明、region、report 和 facade 的其它告警处失败，不能记录为全绿。
