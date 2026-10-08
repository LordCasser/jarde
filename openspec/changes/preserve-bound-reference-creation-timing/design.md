## Context

见proposal。lambda::plan的receiver_nonnull已约束带parameter_adaptation的绑定引用，但直接引用绕过该门。build::lambda_expr当前只用init receiver_tail.is_some()传该事实。root冻结NoCheck原class与错误重编输出，完整-Xverify:all证明确有创建/调用时机偏离。

## Goals / Non-Goals

**Goals:** 同一事实门覆盖直接与适配引用；给已有语义证明补充真实this/分配/明确常量的来源，保持完整refusal/producer retention。

**Non-Goals:** 不增加nullable引用生命周期机制，不推断bootstrap隐含check，不自动转成lambda而将任意capture表达式移到函数体，不改泛型或ctor形状证明。

## Decisions

1. **lambda planner统一gate。** reference_shape、Reach::Receiver、captures=1的引用候选同样消费receiver_nonnull，直接形也不豁免。缺失时给出明确创建时机refusal；既有adapter拒绝原因能保持则保持。不以source::reference会检查null为原bytecode已经检查的证据。
2. **复用同轮事实，不复制算法。** builder在现有bool实参中传事实：receiver_tail，has_receiver且既有receiver_is_entry_this核对SSA entry/load，完成allocation/stable move链的既有证明（可复用init::proved_receiver_class）。直接String/Class等非null常量只依据解码原operation，不根据rendered字符串或owner猜测。函数值仍过capture replayability门。
3. **保守回退而非猜测闭包。** `() -> factory().start()`会把capture创建延后，故不自动替代未获证引用；更广泛可空receiver闭包适配留独立后续。复用现有整个producer/constructor依赖fallback与原始origin。
4. **分层与依赖。** raw parse/frame/SSA不改；build提供源值事实，lambda决定语义形式。无新crate/library/IR/pass或字段，现有bool够用。构造实参主片在本片通过后才能合入；泛型字段片独立。

## Risks / Trade-offs

- 用local0冒充this → 显式has_receiver门+静态参数反例。
- 放宽分配链却忽略later local write → 重用既有完整move/stability proof及capture replayability，拒绝不完整链。
- blanket拒绝伤及已证明非null → this、allocation-tail、直接常量正例和既有bound/typed-functional族回归强制验收；更广泛source checks不在本片承诺。
- 只测反例不能发现效果时机 → 原始NoCheck在真实javac8/23运行验证并比较错误输出；正式产物必须拒绝或匹配，quote内含capture/consumer BCI。

## Migration Plan

本片独立spec/commit，随构造主片root共同门禁后合并。无公开API/依赖迁移；没有证据就保守拒绝，回滚不保留不安全引用输出。
