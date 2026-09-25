# Root 独立验收记录

## 核心 B/C/S 结果

2026-09-25，root 检查了 `array_write` 的最终准入：实际 store opcode 与数组已证明元素类型先相符，`bastore` 的 B/Z 不由 opcode 猜测；只有 B/C/S 消费整数表达式且普通赋值不接受时，才以现有 Cast 包裹值。数组、下标、值仍在写入 BCI 呈现，数字拒绝走 `quoted_bcis`，通用 `meeting_position` 未放宽。相邻字段写入和返回不在本项内。

root 用当前私有 CLI `/tmp/jarde-narrow-array-target/debug/jarde-cli`（SHA-256 `cb0fc223553b7c82b023632fc67513412a48cbb8af31a3bc10d1fa561e41c315`）**独立**对永久 class `tests/fixtures/p3-narrow-array-stores/v8/NarrowArrayStores.class`（760 B，SHA-256 `a3464f4b62da257e4cd4ff70970a05360475e51502a938c675392b334b359803`）运行 `class-source --evidence all`，在隔离临时目录用 `javac --release 8 -g:none` 编译完整输出与 source-only helper/runner，随后 `java -Xverify:all` 执行。结果：零 `@bytecode`，恢复源码 SHA-256 `10d00c05965aec90d1eeefa45fba694e4d3e0eeef8333a78ffb0c008d414aada`，147 行与先前冻结的 patched JVM 输出逐字相同，两者运行输出 SHA-256 均为 `dbd5b1d08239b72ff59a50fad806a4ce55183d7a441a1f64e8a642dab0bb47fe`。该复放不复用 worker 的运行目录。

root 另用 `javac --release 8 -g:none` 独立生成 `NarrowArrayStoreUnknownElement.unknown()V`，其 class SHA-256 `c575d237ba8e5a238075f12919fd11760d88db4a6bba48b3965fd1bacd05e1fc`；`java -Xverify:all` 执行原 class 为 `NullPointerException`，当前 Jarde 保留 `// @bytecode 5`，未把未知 `bastore` 元素猜成 byte/boolean。拒绝后的可编译占位源码不能视为执行等价，故此项只验收诚实拒绝和来源。Z 低位与 boolean 操作数的临时合法变体由 `verification-fixture.md` 单列输入及阶段结果。

对数组、下标和值本身都带调用的三种窄写入，root 另从 `tests/fixtures/p3-narrow-array-store-order-e2e/` 独立编译 int[] 源类，再以精确补丁得到 SHA-256 `eaf27ba0badd9cf6649a829a6fccf17ad06081f6523395f0dff7179fc9d51470` 的 B/C/S class。patched JVM 与当前 Jarde 完整恢复类都经过 `java -Xverify:all`；后者由 `javac --release 8 -g:none` 原样编译，零 `@bytecode`。24 行的数组/下标/值调用顺序、每次调用数、生产者异常优先级、null/OOB、失败时原数组值及前后独立语句逐字相同，输出 SHA-256 `f6b57664f4b875a33e657275e21cbfd1cd581bd9d3122cd15cc47943347c6de5`。root 自行复跑 ignored Rust 端到端测试并以隔离临时目录单独完成第二次 CLI/JVM 重放；详细输入见 `verification-order.md`。

root 还独立运行永久 fixture 的 `run_fixture.py --jarde-cli` 完整三方阶段：Jarde 核心零拒绝、Java 8 javac 与 JVM 均返回 0，147 行零差异；JADX 1.5.6 完整源码 javac 返回 1，因此没有把它列作执行通过。Z 低位输入 17 项和直接 boolean 操数输入 2 项均通过原 JVM 验证；当前 Jarde 对它们及 unknown 输入明确拒绝，拒绝后占位源码的运行差异没有混入 B/C/S 正例统计。

## 定向回归与待关闭范围

root 在同一私有 target 上运行 `p3_array_access` 11/11、`p3_boolean_contexts` 13/13、`p3_deferred_value_order` 2/2、`p3_narrow_array_stores` 3/3、`p3_narrow_field_stores` 2/2；另外三个 ignored 的窄数组/延期求值/窄字段 Java 完整执行对照分别通过。`p3_required_conversions` 除既存 `ir_items` 冻结值外 7/7；其 224→234 来自共享 build 的 guard region 1 项和 compound planner 的 9 条指令扫描，已拆至 roadmap 独立债务。`p3_meeting` 除仍要求 `i2l` 拒绝的旧断言外 5/5；该指令已由 `recover-primitive-conversions` 恢复为 `(long) arg0`，也已独立登记。两项红灯均不是本次数组路径所致，不以修改本案范围掩盖。

返回相邻测试 `p3_narrow_integer_returns` 的两个非 ignored 正例仍在 `directByte` 得到 Mixed，属于已规划但未实施的 [recover-narrow-integer-returns](../recover-narrow-integer-returns/tasks.md)；其 Java 执行测试尚为 ignored。该结果与数组写入分开，不把返回消费位置的缺口并入本项。

严格 Clippy 只命中当前工作树已有的 `useless_conversion`、`too_many_arguments`、`type_complexity`、`needless_option_as_deref`、`collapsible_if` 五类；仅对这五类作命令行例外后 `jarde-java --lib` 与两个新测试 target 均通过。新测试的一处 `single_element_loop` lint 已直接修正。`tests/fixtures/README.md` 已登记两种夹具，根代理重新生成当前工作树的 corpus fingerprint，并运行 `p5_corpus_fingerprint` 5/5；manifest 全量包含其它在途夹具，不能把其大 diff 归因于本项。`cargo fmt --all -- --check`、`git diff --check` 与 `openspec validate recover-narrow-array-stores --strict` 均通过。所有行为与门禁已验收；随后 `cargo clean --target-dir /tmp/jarde-narrow-array-target` 清理 3.4 GiB，项目目录占用 218 MiB，数据卷可用 96 GiB。
