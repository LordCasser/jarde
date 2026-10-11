# 审阅结论

- 已按实际实现纠正 header 类型：`build.rs:21703` 把 `next()` 节点替为呈现 `Object` 的 synthetic local；`21707` 返回 `ForEach.ty = Object`。循环体原有元素声明与 cast 保留。v1 的泛化 `for (element : iterable)` 文句现由 analysis.md 的具体形状取代。
- 当前 raw `List` 不要求预先恢复 `List<String>` 才能与 Object header 相容。JLS §14.14.2 的 enhanced-for 展开允许 raw Iterator 的目标类型为 Object；body 中原 `(String)` cast仍在。参数化 source type 被恢复时，`String` 到 Object 亦是向上赋值。
- generic refusal 来自 class source 的参数化方法声明/body proof 路径（`src/class_source.rs:7178`、`7402`），candidate 并未生成该 proof。维持 marker 是必要的，不把 Signature projection 范围混入本片。
- JADX 的 `fixIterableType` 是它自己的 ArgType/SSA 修正：已有泛型参数兼容检查或尝试设置 iterable 的 generic type；它不等于 Jarde 已证明 generic body，也不应移植成当前 candidate 的 String header。
- 保留 v1 的限制：这里静态读源码与 frozen baseline only。候选完整 SSA/effects/handlers 门、builder 最终输出、JLS 之外的整个文件可编译性均未实测。没有实现成功声明。
