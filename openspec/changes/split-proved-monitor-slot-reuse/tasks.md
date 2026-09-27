## 1. 同步区域与槽位证明

- [ ] 1.1 将已证同步 guard 的物理拥有块交给现有局部复用计划；以 EM-20 夹具确认槽位 2/3 的引用写入与后续整数写入分别落在 guard 内外，普通 try/catch 不被误认成同步 guard。
- [ ] 1.2 扩展 `typed_split` 的异常边检查：仅准入 guard 内清理边，同时核验 SSA 定义/使用、栈上旧值、phi、完整正常/异常可达性与无回边；用真实清理 handler、后段回前段、handler 读后段值和非 guard 异常边测试验证准入/拒绝。

## 2. 声明与完整源码

- [ ] 2.1 让现有局部身份、命名、类型与声明计划消费两段复用结果；以完整 `synchronizedLoop` 类源码 `javac --release 8` 确认循环整数声明及引用均合法，`joined`、`loop`、`caught` 保持可编译。
- [ ] 2.2 验证预算耗尽/取消时不发布半份分割，拒绝报告保留 BCI 和物理来源；运行定向恢复测试确认停止状态与回退契约。

## 3. 三方验收

- [ ] 3.1 重放 EM-20 的固定测试/生产哈希和原/JADX/Jarde 全类对照；Jarde 经 Java 8 重编、`java -Xverify:all` 后七行输出须与原 class 一致，记录最终 CLI SHA、源码、日志及独立复核结果。
- [ ] 3.2 运行相关 `jarde-java`/class-source 测试、`cargo fmt --check`、`cargo check --workspace`、`git diff --check` 与 `openspec validate split-proved-monitor-slot-reuse --strict`；全部通过后才勾选任务并清理本次临时 Cargo target。
