# Array element interval closure regression

`prove_array_initializer` 的旧实现把 element interval closure 放在三种元素路径共同的 postlude 中：child-array、postfix 特例和普通 generic expression 都必须覆盖 value 起点到 store 前的完整物理区间。组合实现调整分支后，闭合检查被移出共同 postlude；普通 generic 与 child-array 路径漏检，只有新增 constructed Site 分支保留完整interval检查。child-array原有的“sources都在区间内”只是单向包含，不能证明“区间内所有指令都属于sources”。

这会让不属于元素值依赖的中间指令被静默略过。`DifferentSlotArrayElement` 的目标表达式读取一个槽，而区间内还写了另一个槽；generic visitor 仍据值依赖构造表达式文本，导致 source side effect 被挪到 array initializer 外表中。

修复恢复共同不变量：各分支完成自己的依赖收集后，统一先拒绝空依赖，再由共同 postlude 调用 `interval_is_expression(block, cursor + 2, store_pos, &element_dependencies, budget)`。constructed Site 分支仅保留其本地 `interval_dependencies.is_empty()` 检查，并移除重复的区间扫描；postfix 特例的识别条件保持不变。这样三条路径都核验完整区间，且每个元素只扫描、计费一次。

本修复仅编辑 `crates/jarde-java/src/build.rs`。未运行 Cargo、Git、rustfmt 或 Java；回归结果待 root 串行验收。
