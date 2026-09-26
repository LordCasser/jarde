# 独立债务：`Outer.super` 方法桥测试与当前投影不一致

这项债务在 DT-02 静态成员类实现的回归巡查中发现，但在未修改的基线提交 `36b57495` 上已经存在。它属于具名非静态成员类的 `Outer.super` 桥闭合，不属于静态成员无捕获证明。

复现命令：

```sh
cargo test -p jarde --test member_family_identity outer_super_method_bridge_is_not_projected -- --nocapture
```

测试 `tests/member_family_identity.rs::outer_super_method_bridge_is_not_projected` 预期 `ClassSourceMemberProjection::Refused`，理由包含 `Outer.super method bridge`；基线实际给出 `Projected`，其派生范围包含 `HiddenOuterSuperBridge`。在干净的 `git archive 36b57495` 中和 DT-02 工作树中均可复现，root 又独立运行了基线测试二进制确认同一失败。

后续应作为 EM-12/DT-03 的独立核验：先比较原始 class、JADX、Jarde 的完整 Java 8 源码和运行行为，再判断测试预期已经过时，还是桥闭合证明放宽过度。未完成该三方核验前，不修改投影规则或把当前 `Projected` 当成正确结果；DT-02 的验收单独报告这条基线失败。
