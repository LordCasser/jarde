## 1. 最小修复

- [x] 1.1 复用receiver_nonnull对直接绑定引用也启用创建时机门；NoCheck/NoStand合法无check反例拒绝且factory/capture/constructor/consumer BCI完整保留，不再输出arg0::start。
- [x] 1.2 builder提供同轮this/完成分配/直接String常量证明，显式has_receiver避免static local0误判；已证明非null/typed-functional/array/static/constructor-ref正例保持。KnownBound.class 的 javac `dup; getClass; pop` receiver仍保守拒绝，已由测试固定；不扩大dup形状所有权。

## 2. 独立验收

- [ ] 2.1 双JDK -Xverify:all确认冻结原class创建成功/调用NPE；正式产物拒绝或行为一致，真实javac带check负例保持保守，root独立审查差异。
- [ ] 2.2 与functional-constructor主片共同完成fmt/clippy/两seed workspace、ignored整类oracle和strict spec，root验收后合并推送，清理Rust/worktree。
