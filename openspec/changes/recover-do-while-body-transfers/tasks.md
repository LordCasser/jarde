## 1. 冻结循环体分支与边界

- [x] 1.1 保存 Java 8 源码、498 B/7项核心、702 B/9项混合及532 B正面对照的原 class、JADX、冻结 jarde 整类编译/执行日志和 SHA；root 复跑 `do-while/run_audit.py`，确认两处失败仅在体内转移。
- [x] 1.2 从核心建立永久 class/runner 与 RED 测试，钉住 `withContinue`、`withBreak` 的 javap Code、源码映射和原 class 的 trace/返回值；补一个非本层出口或嵌套 `switch` 边界，确认它不被写成错误的无标记 `break`。**验收**：[verification-1.2.md](verification-1.2.md) 记录永久 class/runner、核心 class SHA、BCI 转移、原 class 九行输出、RED gate 的来源缺口及嵌套 switch 保守拒绝。原正例源码复编生成的 `DoWhileCore.class` 与冻结 class 字节一致；定向套件 5 passed、1 ignored，switch 负例通过；显式运行 ignored RED gate 在已固定 fixture/Code/执行后按预期于来源覆盖断言失败。

## 2. 只证明本层 do-while 的体内转移

- [x] 2.1 按头块分支的真实后继/闩锁角色修正头测与闩锁测试的区分；`withContinue` 的两臂在闩锁汇合后只呈现一次体语句与条件，正面 `basic` 和带调用闩锁测试保持通过。
- [x] 2.2 对独占且无未认领效果的普通转移桥证明目标恰是本层唯一出口，并在循环臂中写出 `break`；用 `withBreak` 的早退 trace、闩锁调用次数、出口后的单次返回和未证明目标拒绝验证。
- [x] 2.3 保持条件/体/转移/闩锁/出口的真实 BCI 来源、默认/all 正文一致和预算/取消停止；用永久来源断言与低限测试验证每个活块被认领或完整引用。

## 3. 整类执行与 root 验收

- [x] 3.1 原样重编译执行永久核心、完整混合类和健康正面类，对照原 class/JADX/jarde 全部7/9/4项输出与异常；已证实方法零相关引用、整类 `javac --release 8` 和 `-Xverify:all` 成功。见 [Root 独立验收](verification-root.md)。
- [x] 3.2 root 独立审读头测/闩锁判定与转移桥所有权，重放三个完整类及未证明边界；复跑循环、if、switch、guard、来源和停止的相邻回归，记录多层/`switch` 转移债务。见 [Root 独立验收](verification-root.md)。
- [x] 3.3 root 统一 reader census/fingerprint、fmt、适当 Cargo 回归和 OpenSpec strict；只有证据完成后才勾选。**验收**：[最终门禁与相邻回归](verification-root.md#33-最终门禁与相邻回归)；全仓旧失败另见 [基线审计](../../evidence/java-syntax-2026-09-26/test-baseline/analysis.md)。
