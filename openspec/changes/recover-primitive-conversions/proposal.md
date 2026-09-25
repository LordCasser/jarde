## Why

15 个 JVM 基本数值转换 opcode 目前全部落到 Other，三个完整类的 180 项基线均无法从 jarde 输出重编译。root 又确认转换链的中间舍入不可删除：73 项链式对照中 JADX 有7项错值，另有13项重载目标错误；恢复必须保留转换过程，不能直接仿照其简化。

## What Changes

- 忠实恢复 `i2l/i2f/i2d/l2i/l2f/l2d/f2i/f2l/f2d/d2i/d2l/d2f/i2b/i2c/i2s`，复用现有 Cast 及类型呈现。
- 保留转换链、中间精度损失、截断/饱和、符号、窄整数类型，以及后续调用的原 descriptor 目标。
- 接入已有消费、求值位置、来源、预算与取消路径；不因转换本身无副作用就重算或丢失操作数效果。
- 独立于移位、隐式返回窄化和局部声明类型扩展；不新建数值求值器、优化 pass 或一般类型推导框架。生产在共享求值顺序完成后串行进入。

## Capabilities

### New Capabilities

无。

### Modified Capabilities

- `java8-recovery`：恢复明确数值转换指令，保持原值与转换后类型，沿用有界产物和拒绝来源契约。

## Impact

涉及 jarde-java 的 facts/decode 与 build 消费路径，AST/统一 emitter 的 Cast 已存在。保留调用参数静态类型修复，不改 reader/JVM 类型模型或引入依赖。证据在 `../../evidence/java-syntax-2026-09-22/numeric-conversions/`、`root/` 与 `chains/`；三个组的同一180行日志不能重复累加。位运算、boxing、泛型适配、ireturn 隐式窄化及 byte/char/short 局部声明精化不混入本项。
