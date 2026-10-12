# 私有 p3_scope 诊断断言修正候选 v1

基底是当前 `tests/p3_scope.rs`，只替换 `monitorInOuterLoop(I)I` 的旧诊断文本断言。未改共享文件，未运行构建或测试。

- `p3_scope.rs.base` SHA-256：见文件
- `p3_scope.rs` SHA-256：见文件
- unified patch：`p3_scope.patch`

新断言匹配当前 `NoType::ConflictingWrites` 的精确拒绝事实：选择的 `Object` 没有被证明可兼容 BCI 34 写入的 `int`，region 因此拒绝；同时要求首写仍是 BCI 12。它没有退化为 `@bytecode`、fallback 或泛化 refusal 检查。测试原有的 `quality != Structured`、`produced()` 和拒绝伪造 `_2 = 0;` 断言保持不变；handler 的 `crosses a protected region` 断言也保持不变。

静态全仓检索中，只有 `tests/p3_scope.rs` 仍以 `BCI 12 writes Object / BCI 34 writes int` 的旧措辞作断言。其他命中是已冻结 baseline Java 文本中的既有诊断，不是断言，故本补丁不改它们。root-v1 第 53 批实跑输出对应的拒绝仍为同一组首/后写位置和类型；本补丁未执行验收。
