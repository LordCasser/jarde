# `return a[i][j] += x` 架构近邻审计

本笔记只确认当前表达树和近邻证据，不表示新增支持或验收。唯一目标是 `NestedIntBoundaries.returned`；不把 `merged` 的 row-Phi 混进来。

## 当前形状与失败边界

源和 class fixture 在 `tests/fixtures/nested-int-array-compound-updates/NestedIntBoundaries.java:1-4` 与 `tests/fixtures/nested-int-array-compound-updates/v8/NestedIntBoundaries.class`。`returned` 的 descriptor 是 `([[IIII)I`，Java 8 class-file major 为 52，Code 长度 11。字节码按 BCI 为：`0 aload_0; 1 iload_1; 2 aaload; 3 iload_2; 4 dup2; 5 iaload; 6 iload_3; 7 iadd; 8 dup_x2; 9 iastore; 10 ireturn`。`dup2` 把同一行引用/第二维下标各复制一份供读和写；后面的 `dup_x2` 把 `iadd` 结果留一份给 `iastore`、一份给 `ireturn`。因此求值身份和顺序是明确的：行读取、下标和元素读取均先于 RHS；加法结果既写回又作为方法结果。

本片 fresh CLI 的 v8/v23 report 都把 `returned` 留作 fallback；原位置在 BCI 2、4、5、8、9、10 有来源映射，BCI 8 的 `dup_x2` 不在可证明子集中。该观察来自已冻结 CLI JSON/Java 文本，不是对候选源码的编译或 JVM 等价验收。

## 现有 AST 能表达什么

- [ast.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/ast.rs:173) 的 `ExprKind` 有 `LocalAssign`、读元素 `Index`、`PostfixUpdate`、`Binary` 等，但没有表达“写数组元素并返回写入值”的节点。`LocalAssign` 只写局部变量；`PostfixUpdate` 是 `++/--` 并产生旧值，不能代表带 RHS 的 `+=`。`ExprKind::Index` 与 `Binary` 分开只会表达读取和计算，不能表达 iastore 的副作用。
- [ast.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/ast.rs:775) 的 `StmtKind::IndexAssign` 确实有 `AssignOp`，能表达数组元素复合更新，但它是语句；[emit.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/emit.rs:832) 总会在输出后加分号。`StmtKind::Return` 只接受 `Expr`，所以不能把这个现成 statement 放进 `return`。
- [prove_array_update](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:14030) 证明的是 store 位置的语句形态：检查 `dup2` 四个互异输出与各自读/写消费者，并在 `14131-14139` 要求 `sum` 唯一被最终 store 消费。本例的 `sum` 还被紧邻 `ireturn` 消费，所以不能安全地把当前“仅 store”的证明直接放宽成允许任意多消费者。该证明还在 `14081-14109` 核对 `iaload`、原数组引用推得的一维 `int[]` 类型；这些身份和类型条件仍应保留。
- 已有 postfix 路径是表达式副作用的相邻先例，而不是可直接复用的实现：[PostfixUpdate](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:10565) 绑定完整左值读写并只在 `ireturn` 发布；`SnapshotTarget::Array` 在 [build.rs](/Users/lordcasser/workspace/projects/jarde/crates/jarde-java/src/build.rs:11192)，AST 的值语义仍明确是 postfix 旧值。Jarde 的 `tests/p3_postfix_lvalue_values.rs::returned_old_value_updates_are_single_evaluation_expressions_with_real_sources`（`259-291`）已有返回数组 postfix 的正例和真实来源映射；它不能证明复合赋值的“新值返回”。

局部临时变量从理论上能把“计算、写回、return”拆成语句；本 fixture 的 array/index/RHS 都是纯参数。但对一般左值，直接重复 `a[i][j]` 会重复执行数组/下标表达式，可能改变调用次数、空引用/越界异常相对 RHS 的顺序。若采用临时变量路径，必须先一次性保留数组引用和每一级下标，再按原顺序读元素、求 RHS、写回并返回 sum；目前 AST 没有现成的数组左值快照表达式或合成局部绑定身份可直接借用。因此它不是当前可无损复用的捷径。

最小的语义候选是一个能作为 `Return.value` 的数组赋值表达式，直接输出 Java `return a[i][j] += x;`；证明只承认本例这种 `sum` 的精确消费者集合（store 加立即的 `ireturn`），而不是放宽 `single_use_at` 为任意多消费者。需要保留的边界仍包括相同 array/index ValueId 的 store/read 副本、int 元素类型、RHS/读写顺序、无额外 effect/use，以及每个被吸收 BCI 的来源锚点。此处只是候选方向，不是实现方案；额外消费负例可沿用 `tests/p3_compound_lvalue_updates.rs::mismatched_and_shared_lvalue_copies_are_refused_with_their_source_anchors`（`282-310`）的控制习惯，不需要引入 row-Phi 形状。

## JADX 近邻

- 最接近的数组复合写测试是 `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/arith/TestPrimitivesNegate.java:13-18,25-32`：`double[]` 的 `arr[0] += -79` 断言呈现为 `dArr[0] = dArr[0] - 79.0d;`。它覆盖单层数组、语句位置和负常量化简，不覆盖嵌套数组或返回赋值结果。
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/main/java/jadx/core/dex/visitors/SimplifyVisitor.java:128-150,605-666` 的 `convertFieldArith` 明确只从 `IPUT`/`SPUT` 入口进入，并要求 `IGET`/`SGET` 同字段；该例可参考字段复合写的收窄转换，不能当成 APUT 数组表达式处理算法。
- `/Users/lordcasser/workspace/testzone/jadx/jadx-core/src/test/java/jadx/tests/integration/inline/TestInline2.java:11-30` 只是循环中 `b += a[i]` 的数组读取作为局部加法输入，不是数组元素写回。

## 下一次确认应覆盖的边界

以现有 `returned` v8/v23 fixture 做 fresh source/report 与完整类/Runner 对照；正例须保住数组引用、两级下标、元素读、`iadd`、`iastore`、`ireturn` 的一对一来源和返回新值。已有 `tests/p3_compound_lvalue_updates.rs::same_block_field_and_int_array_updates_are_single_evaluation_source_mapped_statements`（`171-243`）可保留为语句位置控制。新增的表达式证明只允许这组已闭合消费者；任意额外 sum 消费仍拒绝。不要用 row-Phi 作为本片的同一证明边界。
