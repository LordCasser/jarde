## 1. 区域后继与布尔尾路径证明

- [x] 1.1 在 `region_at` 的 one-armed 候选中检查递归 `arm_next`，不得丢失与外层 join 不同的正常可达后继；用 CF-02 BCI 11/28 正例和二义后继负例做目标测试。
- [x] 1.2 在现有区域/布尔值入口证明 guard、cast/调用、比较生产者与唯一 `ireturn` 的闭包；以 `null`、非 String、空串、`"x"` 四条路径及副作用次数测试验证。
- [x] 1.3 对无法闭合的异常边、跨区域消费及预算停止保持原子拒绝和 BCI/`unproven` 来源，不得留下可编译但语义错误的无条件早退前缀。

## 2. 固定三方对照与回归

- [x] 2.1 更新 [CF-02 重放脚本](../../evidence/java-syntax-2026-09-27/cf02-predicates/replay.py) 的 Jarde 预期，从冻结错误值改为三方十行一致；用原 class、固定 JADX 与修后 Jarde 的**完整类源码**分别 `javac --release 8 -g:none` 并 `java -Xverify:all`，记录源码哈希及诊断。
- [x] 2.2 运行相关 Region/生成测试、`cargo fmt --check`、`cargo check --workspace` 和 `openspec validate recover-proved-early-return-predicate-tail --strict`；确认浮点 NaN/无穷/负零、数组界限和原有短路测试不回退。
- [x] 2.3 更新 CF-02 验收报告，区分已修早退尾路径与仍待固定队列扩验的比较/条件变体；清理 Cargo 编译残留并提交实现，不代替 root 独立三方验收。
