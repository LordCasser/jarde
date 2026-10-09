# 无界 wildcard 数组返回声明投影实现记录 v1

本记录只说明 class-source 投影一侧的代码改动，不代表本片验收完成。`report.rs` 的同次数组创建候选由 EM18 按已确认的合同单独提供；本次未编辑该文件。

## 改动

- `src/class_source.rs::ordinary_parameterized_declaration` 新增 `GenericReturnValue::ArrayCreation` 分支。仅当方法无方法类型参数、无物理/Signature 参数、无 `throws`、候选参数列表为空，且候选数组名与 reader 从物理方法 descriptor 拼出的返回类型相同，才尝试投影。
- `wildcard_array_return_matches` 对物理返回拼写、Signature 数组 rank 和叶类型实参分别作共享预算计费/poll。它要求 rank 相同、叶类为单一 source segment、叶类名与物理返回一致，且叶类实参非空并且全部为无界 `?`。因此也接受 `Map<?,?>[]` 形状；exact、extends、super、type variable、rank/leaf 不同和缺失数组 Signature 均拒绝。Signature 到物理 descriptor 的擦除有效性仍由既有 reader proof 保证；此分支不重新推导擦除关系。
- 返回声明沿用既有 Signature spelling；正文仍按恢复 AST 输出原始 `new java.util.Collection[][]`，不加入 cast。
- 新增 `src/class_source.rs` 私有 helper 形状测试，覆盖 `Collection<?>[][]` 与 `Map<?,?>[]` 成功以及 exact、extends、super、type variable、rank、leaf、非数组结果和候选/物理数组名不一致拒绝。
- 新增真实 same-run array candidate 驱动的 `ordinary_parameterized_declaration` 测试，确认 wildcard 成功，并确认 same-erasure exact/extends/super/type-variable、rank/leaf 不同及缺失 candidate 在生产声明路径拒绝。另有 helper 的预取消断言，以及 `AnalysisSteps=5` 在物理 rank、Signature rank 和叶节点计满后、扫描 wildcard 实参时停止的断言；focused 测试尚未运行。
- `tests/p3_heterogeneous_array_initializers.rs` 在原 direct 双腿验收中锁定 `collectionGridDirect` 声明为 `public static java.util.Collection<?>[][] collectionGridDirect()`，要求无 generic Signature refusal，且方法正文仍包含 `new java.util.Collection[][]`。

Root focused v1 首次编译发现私有 probe 混用了 `ClassSourceRecovery` 与内部 `RecoveryReport`。已改为从 `recovered.report` 读取 execution/diagnostics，并将该 report 传给 `ClassSourceMethod::recovered`。这是按保存的 focused 编译日志修复；没有重跑 Cargo，故后续验证状态仍未确定。

## 证据边界

`baseline-root-v1.json` 是历史基线，不是本片新执行结果。它记录 javac8/javac23 的原始 Signature `()[[Ljava/util/Collection<*>;`、物理 descriptor `()[[Ljava/util/Collection;`，以及当时完整成员/原始运行流核验继承自前片证据；状态明确为 `baseline_frozen_signature_pending` 且 `fresh_execution=false`。本片投影测试、focused 构建、完整双 JDK 源码重编运行和全仓门禁均尚待 root 执行，不能据此宣称通过。

## 限制

未扩大到参数化 method formal、方法类型变量、`throws`、有界 wildcard、具体类型实参、多个叶 segment 或不同数组 rank/leaf；没有编辑规划文件或任务勾选。共享预算覆盖新增的物理 rank、Signature rank 和 wildcard 实参走访；其他 Signature spelling/Reader 费用保持既有路径。
