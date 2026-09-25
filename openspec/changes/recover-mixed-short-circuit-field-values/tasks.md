## 1. 前置证据与设计收口

- [x] 1.1 先验收 `preserve-region-fallback-instruction-origins` 和 `refuse-overlapping-region-ownership` 的额外入口联合负例，再开放混合图正例；冻结 MixedBooleanField source/class byte equality、原/JADX/Jarde 完整类 Java 8 重编和 16 路径 JVM 对照。验证：基线原/JADX 16 行一致、Jarde fallback 8 行字段值错误，且所有缺口 BCI 已记录。
- [x] 1.2 审计现有 ShortCircuitValue 两/三测试字段、proof 和 emitter 的全部调用者、预算/取消与异常 frame；确定私有测试节点的两个正常后继及拓扑集合，不另建公开 AST/Region。

## 2. 闭合决策图的证明与发射

- [x] 2.1 将局部候选扩成有预算的无环测试图，允许精确证明的内部测试多前驱，禁止外部入口、transfer-only gateway、回边和异常边；只在全部节点/producer/consumer 闭合时一次提交 owner。验证：冻结 `andOr/orAnd` 各有一个 owner，无消费点重访；三元 OR 额外入口完整拒绝。
- [x] 2.2 将短路 SSA/字段证明逐图校验 decoded taken/fallthrough、测试依赖、两个 1/0 producer、唯一 Phi 与唯一 `putstatic Z`，严格处理第二消费者、独立效果及预算/取消。验证：两个混合方法 proof 通过；负例全段 quote，无部分结果。
- [x] 2.3 从外层测试以已有 `Conditional` 递归构造惰性表达式，预算化共享子图复制并在消费点只写一次字段；source map 覆盖全部测试、producer、转移、消费与后缀。验证：原/Jarde 完整类 16 行字段值和两个 calls 逐字一致，方法 `structured/java` 无引用。

## 3. 独立验收

- [x] 3.1 复跑现有两测试 `&&`/`||`、纯三测试 OR、真实异常边与三元 OR 额外入口控制，确认正向放宽没有改变拒绝边界；JADX 只作对照，不是正确性判据。
- [x] 3.2 运行定向/邻近测试、格式、适用 Clippy、`openspec validate recover-mixed-short-circuit-field-values --strict`；记录结果、剩余边界和 Cargo target 清理。仅证据支持的任务勾选，未完成不归档。
