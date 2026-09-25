# 2.2 同轮终端构造器正文验证

2.2 在既有 `recover_for_class_source` 运行中增加最小构造器 AST sidecar。只有类级已识别出 Java 8 双构造器物理对时，facade 才从已读的成员表请求 sidecar；单构造器 `Stage`/`Measure` 不收集此候选。候选保留物理 member identity、完整性、异常表状态、顶层语句顺序、BCI、AST 形状及来源。非 Produced、分析或恢复执行未完成时，facade 会清除候选；Code/AST 捕获与证明沿同一预算和取消通道收费。此交接不读取或解析 `RecoveryReport.text`，也不另跑方法分析。

私有终端正文证明先验证两条物理 `Signature`：唯一属性在通用擦除仍拒绝且仍保留单个 refusal marker 的上下文下，被结构化解析为 `(String,int)V` 对应 `()V`、`(String,int,int)V` 对应 `(I)V`。通用 Signature 擦除规则和物理报告没有放宽。随后终端 Code 必须完整、无 handler，按精确 BCI 顺序只含 `Enum.<init>`、外部类 owner 的静态 `(I)V` 调用、对本类单个精确 `private final int` 实例字段的保存和 `return`。helper owner 必须是合法 Java 路径，helper 方法名及字段 raw/AST 拼写必须各自是单个合法 Java 标识符；字段 source record 不得带 alias 或 refusal marker。该首片不接受同枚举实例 helper 或接口调用。

同轮 AST 的四个顶层语句必须依次对应 BCI `[3,7,12,15]`。BCI 3 的 `super(name, ordinal)` 由入口 slot 1/2 的 Code loads 与 AST operand origins 证明；BCI 7 的 helper Methodref 由相同 BCI 的 Code 引用和 AST 调用名/owner 核对，其实参由 BCI 6 的 `iload_3` 证明；BCI 12 的字段赋值由 `field@1` claim 与同 BCI Fieldref 核对，receiver/RHS origins 分别是 BCI 10 的 `aload_0` 和 BCI 11 的 `iload_3`。不比较调试局部变量名称，因此 `-g:none` 的 `arg3` 与 `-g` 的源参数名走同一结构证明。成员引用清单、AST 步骤数、顺序和终结 `return` 均完整消费；漏用、额外引用/动作、错槽、错目标、错序或异常边均拒绝。

即便上述私有正文事实成立，类级结果仍是 `Refused`，原物理字段、两条构造器与报告结果保持原状。没有实现 2.3 的正文/常量投影，也没有宣称 2.4 来源与反射验收完成；2.2 任务复选框留待 Root 独立验收。

定向拒绝同步改写了非法 helper 名的 Code Methodref、member-use 与 AST 名；非法字段名控制同步改写 Fieldref、member-use、物理字段头/身份和 AST claim，并单独验证字段 alias/refusal marker 会拒绝。聚焦验证（均使用私有 Cargo target）：

```text
cargo test --target-dir /tmp/jarde-enum-delegate-body-target -p jarde --lib enum_constants::tests
cargo test --target-dir /tmp/jarde-enum-delegate-body-target -p jarde --test class_source
cargo test --target-dir /tmp/jarde-enum-delegate-body-target -p jarde-java --test anonymous_allocation_candidates
cargo test --target-dir /tmp/jarde-enum-delegate-body-target -p jarde-java --test class_initializer_candidates
cargo test --target-dir /tmp/jarde-enum-delegate-body-target -p jarde-java --test p3_patterns
cargo clippy --target-dir /tmp/jarde-enum-delegate-body-target -p jarde --lib --no-deps
cargo fmt --all -- --check
git diff --check
openspec validate recover-proved-enum-constructor-delegation --strict
```

最终相关结果：枚举模块 22/22、`class_source` 47/47、`anonymous_allocation_candidates` 4/4 通过；Clippy 退出 0，10 条警告与 2.1 Root 基线一致，无新增警告；fmt、diff check 和 OpenSpec strict 通过。`class_initializer_candidates` 为 8/10，`p3_patterns` 为 52/53；Root 确认其失败与共享工作树既有 [class-source caller 基线记录](../align-class-source-test-callers/verification.md) 中同名失败一致，且这些直接调用均关闭构造器 sidecar，故未改动该范围。九组 verifier-valid 反例的独立重放及哈希验收属于 2.1 Root 已记录的证据；本轮未重复执行。私有 Cargo target 保留给 Root 独立验收后清理。
