## 1. 证明与现有投影衔接

- [ ] 1.1 重放 [EM-21 基线](../../evidence/java-syntax-2026-09-27/em21-this-alias/replay.py)，核对固定 JADX 三个运行测试的正反断言与 `TestRedundantThis` 的注释状态；定位 `reuse`、SSA 读写、声明布局、表达式构造的现有入口。
- [ ] 1.2 在同次方法计划中只为唯一 `this` 写入、准确变量身份且全部读取闭合的候选建立窄证明；用分支双来源、复赋值、槽复用和缺失用途负例确定拒绝边界。

## 2. 原子源码恢复

- [ ] 2.1 让证书命中的写入不发出局部声明/赋值，命中的调用/字段接收者及 `Objects.isNull` 实参读取投影为 `this`，保留现有 AST 与源 BCI；其它值流仍按旧路径构造。
- [ ] 2.2 验证不完整绑定、区域、预算/取消时整份证书作废，不出现未声明局部或半个方法体；原字节码和用途仍可报告。

## 3. 冻结验收

- [ ] 3.1 对 [EM-21 replay.py](../../evidence/java-syntax-2026-09-27/em21-this-alias/replay.py) 增加 fixed 形态断言：Jarde 消去两个简单别名，`choose` 两臂保留；原/JADX/Jarde 全类与共同 Runner 重编 Java 8 并在验证器下逐字输出一致。
- [ ] 3.2 执行定向 class-source/方法恢复测试、格式与适用 workspace check，严格验证本 OpenSpec；在验收报告中单独记录未覆盖的字段遮蔽、构造器或通用复制传播。
