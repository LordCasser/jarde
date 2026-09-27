## Why

固定 JADX EM-06 测试的连续静态字段链中，JADX 在字段声明处保留依赖顺序，Jarde 只输出完整 `static` 块。两者和原 class 的 Java 8 运行相同，但 Jarde 缺少已可证明的源码级字段初值位置。[冻结对照](../../evidence/java-syntax-2026-09-27/em06-field-init/report.md)也确认依赖实例状态的方法调用不能被移出构造器。

## What Changes

- 将既有同轮 `<clinit>` 候选及接口字段初值证明复用于一个普通类的完整静态运行时字段组：每个字段唯一赋值、严格原始顺序、无穿插效果或异常边，所有 RHS 字段读与来源可核验，才原子投影为声明初值。
- 普通类实例字段、构造器及其写入不参与这组投影；ConstantValue、前向读、缺失/重复写入和不完整证明保持原始 `static` 块。
- 使用固定 EM-06 原/JADX/Jarde 完整 Java 8 重编、验证运行及定向拒绝测试验收。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`: 为获证普通类静态初始化链输出有序字段声明初值。

## Impact

限于现有类源码装配、证明报告、writer 和定向测试；不新增 crate、CLI 参数、JVM IR 或依赖。继承字段、异常初始化、实例字段共有初始化及数组特例仍按 EM-06 后续切片审计。
