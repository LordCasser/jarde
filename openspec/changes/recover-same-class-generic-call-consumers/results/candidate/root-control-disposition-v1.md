# 固定候选 v1 的控制样例裁定

## 范围

本裁定针对固定 CLI `b613c0482c8aa56bdf479321abbcc6cadd12c500dc9eec6e2efd8a62daa00900`，不替当前 overlay 源码验收。依据本片 acceptance 的 GC-04/GC-08、design 的直接消费位及 alias/phi 非目标，结合 `root-review-notes-v1.md`、root 独立矩阵核对及 `incomplete-site-audit-v1.md`。不改冻结输入、候选 v1 runner、历史 feature fail 或完整反射差异。

## 判据

正例要求完整指定 API 恢复。拒绝控制则检查拒绝范围、变量身份、物理操作和行为，不要求把不支持的语法伪装为已恢复。两个计数不能混合：通过拒绝控制不增加泛型 API 恢复数，原始反射不一致仍列示。

| 样例 | root 裁定 | 证据及后续要求 |
| --- | --- | --- |
| VoidDirect | 明确正例失败，必须修复 | 原/JADX/已接受基线四腿均保留 sink(T) 与 relay(T)；候选同时擦除为 Object。编译、调用次数通过不能抵消既有 API 回归。v16 新增集成断言仍失败。 |
| SameErasureBinder | 不兼容 binder 拒绝控制通过；完整 API 未恢复 | 原 method U 的 Number & Runnable bounds 不与 class T 的 Number bounds等价。四腿候选保持基线 raw Number sink，不把 U 合并成 T，独立 independent(T) 保留；null 调用计数通过。不是 U API 恢复正例。 |
| MultiUseResult | 多 use/局部 alias 拒绝控制通过；完整 API 未恢复 | identity 结果先存局部，再传给 observe 并返回，超出本片直接消费位。四腿 identity/relay 共同回退 Object；observe(Object)、操作顺序、返回和 observed 的 marker 身份保留。基线四腿编译失败，候选四腿完整编译、验证、行为通过。不能据此声称支持局部泛型传播。 |
| IncompleteSite | 合法 conditional 单调用正例通过；不构成 incomplete census 控制 | 四腿完整 javap 各只有 BCI 6 的 identity 调用。候选两个方法 API 与原一致，Probe 走 true/false 两条路径。缺失 site/物理 census 的负例必须独立补充。 |
| UnknownIncoming | 关联拒绝通过；独立 leaf 保留 | 候选 identity/safe/unsafe 同组擦除、untouched(T) 保留，四腿完整行为通过。原 unchecked T cast 的擦除字节码无运行时 checkcast；不要求虚构该物理操作或宣称恢复了原 cast 语法。实际输出保留物理方法来源和拒绝 marker。 |
| CycleRelay / MethodHandleUse / VarargsCall / InheritedUnknown | 边界拒绝通过；API 损失仍列示 | 固定候选四腿完整编译、验证、实际调用路径通过，API fallback 与已接受基线一致。分别不宣称环推断、bootstrap 泛型代换、varargs 推断或继承 member 类型恢复。 |
| BridgeUnknown | 已有正向控制通过 | 四腿完整 API 和行为与原一致，Probe 额外检查 bridge 存在。不得因为族名含 Unknown 把它归为拒绝。 |

## 未闭合门禁

上述判定不完成 GC-08/GC-10：还缺带完整物理调用事实的 partial census 负例，以及当前窄 overlay 的全部停止/取消与最终完整矩阵验证。GC-01 的 VoidDirect、GC-07 的相关 Comparable<T> overload 头、GC-09 的 SCGA setter 是 v16 明确失败项；constructor-only 回归在 v16 局部断言已通过，仍须四腿旧片重放确认。任务不提前勾选，candidate-v1 不作为交付 CLI。
