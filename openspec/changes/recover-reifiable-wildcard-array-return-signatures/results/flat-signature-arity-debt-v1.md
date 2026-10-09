# 普通泛型 Signature 的类型元数缺口

## 范围

这是本次组合审计发现的共享 Signature 拼写债务，独立于当前 wildcard-array slice。此处只读源代码，没有构造或运行对照 classfile，也没有证明具体变体已通过整条 façade；因此记录为“未证明/可能生成不可编译源码”，不写成已复现的产品错误或已拒绝情形。

## 证据

Reader 的 `prove_method_signature_erasure_with_class_scope` 对方法参数和返回值做擦除与物理 descriptor 精确对照，但 `erase_type` 处理 `SignatureType::Class` 时取最后 segment 的 binary name 生成 descriptor，不读取这个类的 class header，也不比较 Signature 实参数与类声明形参数量，见 [signature.rs:595](/Users/lordcasser/workspace/projects/jarde/crates/jarde-reader/src/signature.rs:595)。此 proof 的文档也把职责限定为 reader fact proof，而不是 source spelling。

普通类型拼写器 `spell_signature_type_with_spelling` 会逐项拼写 `SignatureType::Class` 的实参；其元数核对只在已由 `ProvedMemberInnerSourceSegment` 选中的 member mapping 上执行，见 [class_source.rs:7916](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:7916)。普通外部类路径没有对应 mapping 时，函数仅要求单段类名可拼写，再生成尖括号参数；当前没有该类真实类型参数个数的事实可用于一致性校验。新数组分支的 [wildcard_array_return_matches](/Users/lordcasser/workspace/projects/jarde/src/class_source.rs:7675) 对 leaf 更窄，但同样只要求存在至少一个类型实参且全部是 `?`，不核对它与 leaf 声明的 arity。

因此可作为未来 negative control 设计的同擦除输入包括：物理 `()[Ljava/lang/String;` 配 `()[Ljava/lang/String<*>;`（Java `String<?>[]` 不合法），以及物理 `()[Ljava/util/Collection;` 配 `()[Ljava/util/Collection<**>;`（`Collection` 是单类型形参，而源码实参数为二）。后一种形状满足当前数组 gate 的“非空全 Any”局部谓词；但本报告没有将 class bytes 注入真实完整路径，不能宣称现有 API 一定会将它投影成功。

## 后续最小方向

未来独立 change 应复用当前 class-source/snapshot/runtime 已有的 class header 与来源选择事实，证明 Signature 中每个被拼写类 segment 的声明 arity，再决定是否允许普通拼写。该证明必须对缺失、冲突或不完整 class header 保守拒绝，并沿同一 `Budget` 停止；不要仅按 JDK 类型名加局部 arity 表，因为外部用户类、加载器和版本都会改变事实。数组路径只应消费同一共享拼写证明，不另建一套只针对 wildcard arrays 的元数机制。

本片目标只比较合法 javac 家族的全 `?` Signature 与 same-run array type/rank/erasure，并排除具体/有界实参、形式参数和 throws。Root 正在执行的五源 fresh CLI 和 facade 门禁用于本片验收，不会替代未来的非法元数负控；本债务不改变当前任务的实现或验收分母。
