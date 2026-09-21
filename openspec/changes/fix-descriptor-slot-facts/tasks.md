## 1. 反例与统一事实

- [x] 1.1 先加失败用例：`static int f(long[] xs, int n)`、`static long g(double[][] m, long k)`、实例 `int h(long[] xs, int n)`（`javac --release 8 -g:none` 自造或用测试内构造器），断言**签名里的参数名与正文引用指向同一槽**。验证：修前必红，并给出真实失败输出（D01）。
- [x] 1.2 在 `jarde-reader` 提供唯一的描述符事实（基本类型/原始类名/数组维数/参数序号/槽占用），并加单元测试：数组一槽、`long`/`double` 两槽、`void` 位置合法性、畸形描述符拒绝沿用既有错误码。验证：新单元测试 + 反例 1.1 转绿（D01）。
。证据：实现提交 `138524a`；详见 [verification](verification.md)。

## 2. 消费点切换

- [x] 2.1 把 `jarde-jvm` 的 frame/lambda/声明槽位计算切到同一事实；把 `src/class_source.rs` 与 `jarde-java` 的拼写切到同一事实，删除分叉实现。验证：`cargo test -p jarde-jvm -p jarde-java`、`tests/p3_prepared_input.rs`、`tests/p3_instance_receiver.rs`、`tests/p3_content.rs` 全绿（D01/D02）。
- [x] 2.2 逐条核对受影响的既有断言（含 golden），确认差异只来自"数组槽宽修正"与随之左移的槽位编号；不得放宽或删除断言。验证：可逆差异检查 + 受影响清单（D01）。
。证据：见 [verification](verification.md)。

## 3. 行为与回归

- [x] 3.1 受控 JDK 编译执行对照覆盖数组与 `long`/`double` 混排、实例 receiver、无调试信息命名（至少一个入口；另一入口说明为何未覆盖）。验证：`cargo test --test p3_execution_comparison --locked -- --ignored` 真实通过（D12）。
- [x] 3.2 反例：把数组槽宽改回"沿元素宽度" → 1.1 与 2.x 的断言必须变红；还原后全绿。验证：真实失败输出 + 还原方式（D01）。
- [x] 3.3 全量门禁：`cargo fmt --all`、`cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`、`cargo test --workspace --all-targets --all-features --locked` 全绿；OpenSpec strict 保持通过（D01/D12）。

## Acceptance Map

| ID | 主题 | 主要外部判据 |
| --- | --- | --- |
| F01 | 数组/宽度槽事实 | `long[]`/`double[][]` 与 `long`/`double` 混排的签名与正文一致 |
| F02 | 单一解析者 | frame/lambda/声明/拼写都消费同一事实，无第二套解析 |
| F03 | 行为不变 | 受控编译执行对照通过，prepared/直接等价与 receiver 规则未回退 |
