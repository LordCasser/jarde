# 任务 2.3 验收：保留已证枚举前缀后的单项静态赋值

投影继续使用 2.1 的同次 Code/use 事实和 `<clinit>` AST 侧车。新增门槛只接受 `LOW(2), HIGH(5)` 且前缀结束 BCI 为 34 的形状：BCI 34 必须是对本类唯一 `sumUnits()I` 的 `invokestatic`，BCI 37 必须是写入唯一、无 `ConstantValue`、`ACC_STATIC`、`I` 描述符 `totalUnits` 字段的 `putstatic`，BCI 40 必须是唯一正常 `return`。Code/use 身份与顺序、异常表、侧车五个步骤、FieldWrite 的字段身份/操作、BCI 37 语句来源和 BCI 34 RHS 来源都需一致；不接受额外指令或跨边界来源。

只有该门通过且现有 enum 源构造器计划、RHS 表达式发射和完整输出预算都成功时，源码文本才省略物理 `<clinit>`，并在普通字段之后写 `static { totalUnits = sumUnits(); }`。失败或停止保留原字段式常量、完整 `<clinit>` 与用户效果。RHS 只由现有 `emit_class_initializer_value` 发射；物理字段和方法记录及其 outcome 不做删改。

本地 `cargo test -p jarde --lib enum_constants::tests --target-dir /tmp/jarde-enum-proof-target` 为 15/15；覆盖 Stage、Measure、Counted、四类 1.2 负例、额外 `$VALUES` 读取、额外调用/字段写/异常边、默认/all evidence 正文和物理成员、预算回退及取消。`class_source` 为 47/47，`interface_initializer_proof` 为 4/4，`interface_initializer_projection` 为 6/6。完整 Measure 与 MeasureRunner 的原始、JADX、Jarde 源码分别用 `javac --release 8 -g` 和 `-g:none` 编译，并以 `java -Xverify:all` 运行；两种调试模式下均逐行得到 `user static boundary: PASS`。Counted 的原始和 Jarde 完整类也在两种模式下重编、验证，runner 均得到 `calls=1,total=7`，证明 `sumUnits()` 的副作用恰执行一次。Root 独立确认 Counted 原始与 JADX 重编类在两种模式下也得到同一结果。

Root 独立重建 CLI 后确认 Stage、Measure 在 `-g`/`-g:none` 下的原始/Jarde verifier 运行结果一致，默认和完整 evidence 正文一致，Stage 的 4 fields/6 methods 与 Measure 的 5 fields/6 methods 均保留；六个既有拒绝样本（name/ordinal、values 顺序、`valueOf` 空参数、构造器形状、常量前缀效果）和额外 `$VALUES` 读取样本未误投影。

验证命令还包括 `cargo fmt --all -- --check`、`cargo clippy -p jarde --lib --no-deps --target-dir /tmp/jarde-enum-proof-target` 与 `openspec validate --strict recover-proved-enum-constants`。Clippy 为既有 10 条告警，没有 2.3 新增项。额外尝试 `cargo test -p jarde-java --test class_initializer_candidates --target-dir /tmp/jarde-enum-proof-target` 时，当前共享工作树的该测试文件有 7 处 `recover_for_class_source` 调用缺少新必需的布尔参数，未通过编译；该文件不在本任务修改范围内。私有 Cargo target 保留供 Root 独立复核。
