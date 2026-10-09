# `collectionGridDirect` 泛型 Signature 拒绝定位

## 结论

`collectionGridDirect` 的 array body 已被恢复为完整结构，但候选源码仍是擦除类型 `Collection[][]`，并带有 `ordinary_generic_source_unproved` 注释。当前精确拒绝发生在泛型 Signature 投影的 same-run body candidate 门：`generic_return_candidate` 没有 `NewArray` 返回候选形状，所以返回 `None`；`ordinary_parameterized_declaration` 随后以 “same-run Program/SSA cannot prove the body under parameterized types” 拒绝投影。

这不是数组 Signature 语法无法解析，也不是本片 Builder 的 erased `ArrayStore` 赋值门失败。更准确地说，是缺少把完整 `NewArray` 返回 AST/SSA 与泛型 method return Signature 对齐的证明形状。泛型 Signature 中的数组递归拼写已有支持；generic array 源类型之间的参数化继承/通配符兼容没有被这个返回候选路径证明。当前没有 cast 规则或可直接复用的完整 generic-array widening 规则可以解决它。

## 永久结果与原始源码

本报告依据 `results/candidate-root-v4/manifest.json`，SHA-256 为 `a073bc6a41a057f81c1bcb3df1a00dfd2a53093e55f35aa05f59a8d7e5f8c440`。manifest 中 fixture/direct 的 javac8、javac23 两条记录均为 `candidate_success=true`，全源集编译和 candidate runtime exit 均为 0，raw streams 与原输入相同；`collectionGridDirect` 的 method record 为 `quality=structured`、`representation=java`、10 个 source-map segments。

但这表示擦除形式 body 被成功恢复和验证，不表示泛型 Signature 已投影。两条输出源都在方法前留有：

```text
// jarde: generic Signature projection refused for `collectionGridDirect()[[Ljava/util/Collection;`: unsupported (ordinary_generic_source_unproved): same-run Program/SSA cannot prove the body under parameterized types
public static java.util.Collection[][] collectionGridDirect() {
```

`fixture/direct/javac8/sources/Main.java` 与 `javac23/sources/Main.java` 内容相同，SHA-256 均为 `07b8aa3351bbe2f6875b327f4f9c687c3779975bb578e1300f142acf67ae3695`。正文是完整 raw initializer：`new Collection[][] { new ArrayList[] { listValue(...) }, new HashSet[] { setValue(...) } }`。canonical 输入 `tests/fixtures/p3-heterogeneous-array-initializers-v3/direct/Main.java` SHA-256 为 `e4e9189086f9417c523c4db0cadce69739c66f9e6c297ccc4a7914768f29a017`；其方法签名与正文在 102–106 行声明 `Collection<?>[][]`、`ArrayList<?>[]`、`HashSet<?>[]`。因此候选原流等价和 Java 编译通过不能算作泛型类型保真通过。

## 拒绝路径

1. `report.rs::generic_return_candidate` 以同一 `Program`、SSA、operations 和 construction Sites 为输入。它要求非 ragged、非空的完整 Program，再从根 return value 建立有限的 `GenericReturnValue`（`crates/jarde-java/src/report.rs:7650-7673`）。目前根表达式支持 Local/Parameter、Conditional、受证明的成员创建、静态成员创建和 TypedFunctional。`ExprKind::NewArray` 没有专门分支，最终落到 `_ => return Ok(None)`（`report.rs:8192-8235`）。该 enum 也没有数组 initializer 变体（`report.rs:5117` 起）。

2. `class_source::ordinary_parameterized_declaration` 用该 candidate 与实际 method Signature 校验 body。Recovered body 没有 candidate 时，精确在 `src/class_source.rs:7400-7403` 抛出上述错误；此时还没有进入任何 “ArrayList<?>[] 是否能赋给 Collection<?>[]” 判断。

3. `spell_signature_type_with_spelling` 已递归处理 `SignatureType::Array`，可以拼出 `Collection<?>[][]`（`src/class_source.rs:7805-7815`）。所以拒绝不来自多维数组 Signature 的解析/打印。

4. 已有的泛型源类型判定 `overload_source_type_assignable` 会递归处理 `SignatureType::Array`，但它属于 generic overload applicability proof，不是 initializer return proof。其类到类路径对带 type arguments 的非同类型关系保守返回 unknown；注释也明确 wildcard conversions remain unknown（`src/class_source.rs:4113-4155`）。它既不接收当前 `Program` 中 child-array 的准确 store 节点，也不单独证明 `ArrayList<?>[]` 与 `Collection<?>[]` 这类通配符数组层级关系，不能据其存在就声称此缺口已有答案。

5. 本片 Builder 的 `array_initializer_element` 只消费擦除后的 `Type`、SSA expression 和确切 store BCI；它通过 exact component、闭集 array/platform 关系或 facade 提供的 exact snapshot hierarchy widening 决定 erased reference 是否赋值兼容（`crates/jarde-java/src/build.rs:26673-26735`, `29434-29459`）。Facade 的 `aastore` facts 来源是 arrayref descriptor component 与 stored SSA value type（`src/facade.rs:25108-25159`），snapshot header walk 证明的是 leaf class hierarchy 及完整擦除 type spelling（`25169-25213`），不携带 generic type arguments。这些机制证明 `ArrayList[] → Collection[]` 等字节码擦除关系，不能替泛型 Signature 投影证明 `ArrayList<?>[] → Collection<?>[]`。

## 可复用边界

最贴近现有架构的入口是 `GenericReturnCandidate`：producer 已从当前恢复的完整 `Program`/SSA 形成同 run 的 body facts，`ordinary_parameterized_declaration` 已统一负责 Signature erasure 和返回源校验。`ExprKind::NewArray` 与其 origins 能定位数组结构和 store；`ArrayInitializers`/`Builder` 已保存每个 initializer element 与准确 `store_bci` 的对应关系；Snapshot hierarchy facts 可继续提供 class 继承的 erased 部分。Signature parser/speller 已支持任意有限数组层级。

缺口是把以上事实组装成一个针对 generic return 的完整数组值证明，并对目标 Signature 与各层真实运行时数组类型/元素类型做来源一致的参数化赋值核验。generic call overload helper 虽能作为局部参考，不能当作通用 generic type service。当前任务没有增加这项范围；本报告不建议改写该 API，也不把 raw candidate 的成功解释为 generic source acceptance。

## 证据边界

本次只读取当前 facade、class-source、Builder 和 manifest/source 证据；未运行 Cargo、Git 或 Java，也没有修改产品、fixture 或旧结果。manifest 中的 direct javac8/javac23 candidate 执行成功是已冻结的既有证据；本报告的根因定位是由其 marker 与当前源码控制流相互印证的静态结论，不是新增 candidate 运行。
