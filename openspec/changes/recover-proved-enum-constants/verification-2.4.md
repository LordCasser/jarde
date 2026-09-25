# 任务 2.4 验收：类文本投影与物理成员记录并存

`ClassSourceReport`、`ClassSourceField`、`ClassSourceMethod` 的 RustDoc 说明：枚举投影只改变组装后的类文本；物理字段/方法表项、索引、身份和成员 outcome 留在报告 JSON 中，方法级恢复报告仍带原 BCI 来源。成功投影时编译器隐式成员不在类文本重复声明；证明拒绝或停止则保留物理形式。`docs/support-matrix.md` 的 class-source 行已补充这个唯一例外及物理事实仍可查询的边界。

Measure 的定向测试对 default 与 all evidence 检查相同正文，同时逐项核对 5 个字段、6 个方法的名字、连续物理索引、物理身份和每个方法 outcome。它检查隐式 `$VALUES`、`$values()`、`values()`、`valueOf(String)` 不重复写入组装文本，并从保留的 `<clinit>` 恢复报告/source map 确认 BCI 34 仍映射到 `sumUnits()`、BCI 37 仍映射到 `totalUnits` 写入。预算停止的同一测试路径确认正文完整保留原字段式常量和用户 `<clinit>`，JSON 仍含 5/6 个物理成员及 `<clinit>` 原 outcome。既有预取消测试仍确认请求沿 `Incomplete/Cancelled` 停止，未生成部分 enum 报告。

另加 `OrdinaryInit` 普通类回归：enum proof 为 `NotApplicable`，两个普通静态字段和原静态初始化块保留，default/all 正文相同。普通 class-source 测试及接口字段初始化证明/投影测试继续覆盖已有声明、字段和类初始化路径。

验证结果：

- `cargo test -p jarde --lib enum_constants::tests --target-dir /tmp/jarde-enum-proof-target`：16/16。
- `cargo test -p jarde --test class_source --target-dir /tmp/jarde-enum-proof-target`：47/47。
- `cargo test -p jarde --test interface_initializer_proof --target-dir /tmp/jarde-enum-proof-target`：4/4。
- `cargo test -p jarde --test interface_initializer_projection --target-dir /tmp/jarde-enum-proof-target`：6/6。
- `cargo fmt --all` 已执行；`cargo clippy -p jarde --lib --no-deps --target-dir /tmp/jarde-enum-proof-target` 只有既存 10 条 warning，没有新增 warning。
- `openspec validate --strict recover-proved-enum-constants` 通过。

Root 在 2.3 独立确认完整 CLI 的 Measure 物理成员与原始恢复正文/source-map 保持不变；该实现没有改 CLI 或公开 JSON schema。`docs/support-matrix.md` 同一行关于 package、throws、注解和泛型的其余概括早于现有实现，属于另案修订的历史文案债务，本任务只补枚举投影边界。2.3 验收记录保留共享工作树中 `class_initializer_candidates` 的独立 Rust 测试签名编译债务。共享 Cargo target 保留供 Root 独立复核。
