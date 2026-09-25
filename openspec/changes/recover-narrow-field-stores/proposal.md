## Why

JVM 的 `putfield` / `putstatic` 会把栈上的 int 值按目标 byte、char、short 字段窄化。当前 jarde 用普通 Java 赋值规则处理已呈现的 int，随后拒绝该写入；完整类文本却仍可能编译成空方法，漏掉字段变化、生产者调用与异常。该缺口既不是字段识别失败，也不需要通用类型推理。

## What Changes

- 只在已验证的真实窄字段写入位置，依据字段描述符与已呈现整数操作数，使用现有 Cast 表达 JVM 窄化。
- 沿用 `field_write` / `field_value`、verified accessor 写入、现有来源和延迟值绑定；保持接收者、值生产者、null 检查的先后及调用次数。
- 拒绝仍未获证明的 boolean 低位规则和其他引用/数值转换；不放宽通用赋值与调用位置，也不引入新类型 pass。
- 本案按上述边界完成字段写入消费点实现；完整 267/285 项运行证据见 `verification.md`。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：保留 byte/char/short 字段写入指令已经陈述的窄化及运行副作用。

## Impact

主要改 jarde-java 字段写入值的消费位置与相邻测试，不改 reader/SSA 的字段识别。证据见 `../../evidence/java-syntax-2026-09-22/numeric-conversions/narrow-field-stores/`：完整 380 项含 Z 边界，输入阶段裁剪的 B/C/S 核心 285 项。JADX 完整源码 javac 失败，因此不能充当此例的运行 oracle；原 class 是运行判据。
