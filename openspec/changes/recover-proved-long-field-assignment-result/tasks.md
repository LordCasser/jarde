## 1. 同次形态证明

- [ ] 1.1 重放 EM-07 冻结基线并核对 `javap` 的五步 `dup2_x1` 路径、现有字段证据及 SSA 栈读写；以 `replay.py baseline` 摘要和原始正例测试固定实现输入。
- [ ] 1.2 在现有 `jarde-java` 方法 builder 中证明唯一 `long` 参数、准确本类实例 `J` 字段、category-2 复制的两个指定消费者、完整单块/无 handler 的范围；用正例及错误宽度、owner、descriptor、第二消费者、插入操作负例验证只命中准确形态。

## 2. 结构化恢复与拒绝

- [ ] 2.1 从已证明的值和字段身份写出既有 `FieldAssign` 与 `Return` 两句，BCI 2 保留来源锚点；以源码断言和来源测试确认字段只写一次、参数值只返回一次，且不引入通用赋值表达式 AST。
- [ ] 2.2 对不完整 SSA/效果、异常 handler、预算耗尽和取消保留原子拒绝及物理报告；用定向测试确认不发布半个方法体、停止原因和执行状态准确。

## 3. 冻结验收

- [ ] 3.1 执行 [EM-07 replay.py](../../evidence/java-syntax-2026-09-27/em07-long-assignment/replay.py) 的 `fixed` 模式；原/JADX/Jarde 完整 Java 8 源码重编、`java -Xverify:all` 在跨 32 位正数和负数下返回值/字段值逐字一致。
- [ ] 3.2 跑字段恢复及 class-source 定向测试、格式检查、适用 workspace check 与 `openspec validate recover-proved-long-field-assignment-result --strict`；记录检查结果，不把 EM-19 数组或其它栈重排债务混进实现。
