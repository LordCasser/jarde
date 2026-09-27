## 1. 固定物理形态与证明

- [x] 1.1 重放 [EM-23 基线](../../evidence/java-syntax-2026-09-27/em23-field-updates/replay.py)，核对 JADX 活动字段断言、`TestArith` 的未完成精确局部断言，定位 Jarde `CompoundAssignments`/`FieldIncrements`、`field@1` 与后缀 AST。
- [x] 1.2 用同次 Code/SSA/field 事实证明两个无 handler 单块闭合形态及所有读取唯一用途；定向负例覆盖错 owner/descriptor、复杂接收者、volatile、额外消费/效果和异常范围。

## 2. 表达式与原子投影

- [x] 2.1 使既有后缀更新节点表达增/减两个方向，保留旧返回值/数组自增的行为及来源；对获证语句链输出 `this.instanceField++`、`Updates.staticField--` 一次。
- [x] 2.2 证明失败或预算/取消时保留既有字段赋值或拒绝，不产生半条更新或失去字段效果；字符串构造链不在此变更中处理。

## 3. 冻结验收

- [x] 3.1 将 [EM-23 replay.py](../../evidence/java-syntax-2026-09-27/em23-field-updates/replay.py) 加 fixed 字段拼写断言，原/JADX/Jarde 完整 Java 8 类重编、`java -Xverify:all` 输出一致。
- [x] 3.2 执行字段更新/后缀表达式定向测试、格式与适用 workspace check，严格验证本 OpenSpec；单独记录 EM-27 字符串、嵌套字段复合赋值和循环局部的剩余项。

本切片的未覆盖项：字符串 `+=`/StringBuilder 链归 EM-27；嵌套接收者字段复合赋值（如 `this.a.f += n`）仍须证明接收者身份与求值次数；循环局部变量更新不在字段后缀恢复内，沿 EM-20/控制流审计处理。
