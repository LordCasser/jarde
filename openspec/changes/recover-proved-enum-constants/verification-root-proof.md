# Root 独立验收：任务 2.1 的枚举类级证明

本验收只确认**同一次 class-source 成员运行中的私有证明侧车**；2.2–2.4 的常量列表、构造器及静态后缀源码投影仍未实施，不能据此宣称 Jarde 完整枚举类已经可重编。

`src/enum_constants.rs` 从已分析的 `MethodIr` 拷贝候选 Code 指令、常量池身份和所有物理方法对拟消隐成员的直接/Bootstrap 引用；不另读类、不解析 Java 文本。只在已准备的 Java 8、完整成员表、恰两枚举常量字段的类上收集额外事实。证明要求字段/方法表身份、构造调用的 name/ordinal/int 实参、`$VALUES` 数组顺序、标准 `values`/`valueOf` 和 `<clinit>` 结构化前缀一一匹配；用户方法可读取源码可表达的常量/标准 API，但额外读取 `$VALUES` 本体或调用 `$values` 会整组拒绝。MethodHandle 只解一层，Bootstrap 关联项逐项计费并轮询。私有结果不进入公开 JSON；原始成员报告保持不变。

Root 在最终代码上独立运行：

- `cargo test -p jarde --lib --test class_source --locked --target-dir /tmp/jarde-enum-proof-target`：39 个库测试和 47 个类源码测试全过；其中 `enum_constants::tests` 8/8 覆盖 Stage/Measure、方法/字段索引、四个 helper/构造器/前缀变体、额外 `$VALUES` 读取保留原方法文本、预算停止与预取消。
- `cargo test -p jarde-java --lib --locked --target-dir /tmp/jarde-enum-proof-target`：180/180，通过于这次仅改 adapter 的重构前；`jarde-java` 自身在随后未变。
- 最终 CLI SHA-256 `1c68b1b8a1a8e5ef37c8003714d5c2083a3ed83bb5b6008daf64591fabb3a019`。用 `Stage`、`Measure` 完整原 class 的 `--evidence all --format json` 对比冻结旧 CLI `ca04265a4f412d59c29d6bd4a26b7d9cb961f72ae13e77684831c0b9e57b5145`：两组 `text` 字节相同、顶层 JSON 字段集合相同、无 `enum_constant_proof` 字段、执行均 complete。
- [额外数组读取三方反例](../../evidence/java-syntax-2026-09-25/enum-values-access/analysis.md)在 `-g`/`-g:none` 下两次重放：原 class 经 verifier 输出 `B`，JADX 重编输出 `A`；Jarde 证明测试拒绝，原 `return E.$VALUES;` 留在类正文。
- `cargo fmt --all -- --check`、`git diff --check`、`openspec validate recover-proved-enum-constants --strict` 通过。`cargo clippy -p jarde --lib --no-deps --locked` 成功，只有既存 11 条 `jarde` lint；全依赖 `-D warnings` 仍先被既存的 17 条 `jarde-java` lint 阻断。本轮新增的三条复杂度 lint 已消除，旧债不混入本变更。

仍需 2.2/2.3 将已证事实投影成可编译 enum 常量列表、源码构造器与静态用户后缀；2.4 明确物理成员报告与类正文的关系，3.1/3.2 再做完整运行验收。Bootstrap MethodHandle 的专门 verifier-valid 变体未单独冻结，本轮基于结构化常量池解码与有界单层引用审查；若后续扩展引用变体，应另加拒绝证据，不放宽当前门。
