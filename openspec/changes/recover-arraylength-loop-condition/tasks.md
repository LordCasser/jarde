## 1. 准入与证据

- [ ] 1.1 在既有循环测试块证明中，仅对终端分支同块 SSA 消费链内、可一次呈现的 `ArrayLength` 允许通过；用定向 Region 正反例验证未消费、复用及额外效果继续拒绝。
- [ ] 1.2 确认现有 Builder 与声明路径在 `StepIndex.everyOther` 上生成带原 BCI 来源的完整非 foreach 索引循环和返回；以定向恢复测试断言 `i += 2`、数组长度、返回值及无引用回退。

## 2. 完整类验收

- [ ] 2.1 对隔离 `StepIndex` 和组合 `ForeachCases` 重放原 class、固定 JADX、修后 Jarde 的完整 Java 8 重编与 `java -Xverify:all`；逐行核对 `4` 与 `10/abc/4`，补充 null 数组异常位置/次数对照并记录来源哈希。
- [ ] 2.2 运行相邻 foreach/循环回归、预算/取消门、`cargo fmt --all -- --check`、相关 crate 测试、workspace check 与 `openspec validate recover-arraylength-loop-condition --strict`；将结果写入 CF-10 证据并保持 CF-10 单元状态为部分已测。
