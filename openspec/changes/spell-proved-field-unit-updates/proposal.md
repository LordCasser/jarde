## Why

固定 JADX `TestFieldIncrement` 要求实例整数字段 `++` 与静态整数字段 `--`。EM-23 的完整 Java 8 对照中，Jarde 能正确重编运行，但分别输出 `+= 1` 和 `field = field - 1`。现有字段计划与 SSA 已能证明前者的读写，缺的是这两个无返回值单位更新在末端的源码形式；字符串 `+=` 属另一条拼接构造链，不应混入。

## What Changes

- 对准确、无 handler 的本类 `int` 字段单位更新链，证明一次字段读取、一个常量 `1` 的加/减、一次同字段写入，且更新值不再被消费。
- 复用现有字段/SSA 计划并使已有后缀自增 AST 能表达增一和减一，原子输出 `this.field++` 或 `Owner.field--`；保留真实接收者只求值一次。
- 对目标/接收者/宽度/效果/返回值使用/异常路径不完整的形态保留原投影或拒绝；增加三方完整源码重编与验证运行及定向负例。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 将获证的语句级 `int` 字段单位更新恢复为 Java 后缀 `++`/`--`。

## Impact

影响 `jarde-java` 既有字段/SSA 更新证明和后缀表达式表示、emitter 与定向测试；不增加反编译 pass、JVM IR、公开 CLI/schema 或依赖。冻结对照见 [EM-23 审计](../../evidence/java-syntax-2026-09-27/em23-field-updates/report.md)。
