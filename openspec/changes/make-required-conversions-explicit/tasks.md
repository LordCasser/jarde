## 1. concat 反例与转换节点

- [ ] 1.1 先加失败用例并复跑 T5：`append((int) c)`（`c` 为 `char`，输入 `'A'`），断言原类与恢复文本在同一输入上执行结果相同（原 `"65!"`）。验证：修前必红，附真实对照输出（C01）。
- [ ] 1.2 在表达式构建期引入显式转换节点，并在 concat 路径上对齐"片段呈现类型"与"append 参数类型"；`char → int`、`byte/short → int` 与必要的加宽转换按证据插入，不需要时不插入。验证：1.1 转绿 + 既有 `tests/p3_concat_conversion.rs` 全绿（C01/C03）。
- [ ] 1.3 反例：去掉转换节点生成（回到只按上下文拼写）→ 1.1 与至少一条既有对照变红；还原后全绿。验证：真实失败输出 + 还原（C01）。

## 2. 复用到其它消费位置

- [ ] 2.1 把同一机制复用到调用实参与构造实参：实参呈现类型与形参类型不一致时显式转换，无法证明时拒绝。验证：新增执行对照样本 + 既有 `p3_accessor_edges`/`p3_content` 全绿（C02/C03）。
- [ ] 2.2 复用到返回与赋值/字段写入位置（含 `char`/`short`/`byte` 与 `long`/`double` 混排）。验证：执行对照样本（C02/C03）。
- [ ] 2.3 逐条核对受影响的既有文本断言（含 golden），确认差异只来自"新增的显式转换"；不得放宽断言。验证：受影响清单 + 可逆差异检查（C03）。

## 3. 回归与口径

- [ ] 3.1 计费对照：转换节点的 `IrItems`/`NormalizationClones` 前后计数，确认没有用"少构造"冒充正确性。验证：计数表（C03）。
- [ ] 3.2 全量门禁：`cargo fmt --all`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked`、`cargo test --test p3_execution_comparison --locked -- --ignored` 全绿；OpenSpec strict 通过（C01–C03）。

## Acceptance Map

| ID | 主题 | 主要外部判据 |
| --- | --- | --- |
| C01 | T5 反例 | `append((int) c)` 的执行结果与原类一致 |
| C02 | 转换机制 | 调用/返回/赋值位置同规则，无法证明时拒绝而非猜 |
| C03 | 无回退 | 既有拼接/布尔/拒绝断言与执行对照全绿，差异只来自新增转换 |
