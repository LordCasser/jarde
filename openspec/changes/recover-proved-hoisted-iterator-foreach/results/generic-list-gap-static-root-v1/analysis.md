# 同一个 CF11 样例的 List<String> 差距

仅静态定位后续工作，没有新的实现或泛型运行验收。本片仍只恢复内层 Object foreach，整个 CF11 尚未关闭。

固定完整类基线的真实拒绝是 `ordinary_generic_source_unproved: same-run Program/SSA cannot prove the body under parameterized types`。`src/class_source.rs` 的 ordinary_parameterized_declaration（7178）已有 Signature/erasure 拼写，7350 后要求同 run body proof 或 GenericReturnCandidate；缺候选在 7420 附近准确拒绝。因此不需要另一套 Signature parser。

仅替换参数头不成立：当前正文保留 `l2.add((java.lang.Object) s1)`，改为 List<String> 后这个调用不能编译。`remove((Object) s2)` 则有保留原重载绑定的意义，不能统一删 cast；Integer/索引重载尤其需要反例。已有 GenericReturnValue::VoidBody 对应完整直线 void，不可冒用为循环证明。

`src/facade.rs::prove_generic_method_body`（16950）已有物理 formal/AST/SSA/Methodref consumer 事实，但 invocation_statements 要求每个调用有唯一 validated_overload BCI 且 owner 为当前类；目标含平台 List 调用，不能把同类证明当平台泛型证明。下一片须先确认现有 projection/consumer 能否消费有限平台契约：iterator、add(E)、remove(Object)、size、clear；只在准确 receiver/formal/元素来源和完整调用绑定证明成立时允许泛型拼写及 add 的 Object 上转型适配。优先复用既有 projection，不预设新增机制。

下一项分析按固定样例推进：逐调用列出 Memberref/SSA 来源/源 AST consumer 与当前拒绝；用 String 正例和 List<Integer> remove、raw alias/escape、wildcard/control 反例确认界限，再写独立 OpenSpec。不得混入当前 foreach 改动或增加已完成单元数。
